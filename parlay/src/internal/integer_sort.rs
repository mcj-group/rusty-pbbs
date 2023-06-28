// ============================================================================
// This code is part of Rusty-PBBS.
// ----------------------------------------------------------------------------
// MIT License
// 
// Copyright (c) 2023-present Javad Abdi, Mark C. Jeffrey
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
// ============================================================================

use std::mem::size_of;
use std::slice::from_raw_parts;
use rayon::prelude::*;
use enhanced_rayon::prelude::*;

use crate::{DefInt, maybe_uninit_vec};
use crate::utilities::log2_up;
use crate::internal::counting_sort::{count_sort, seq_count_sort_};

const RADIX: usize = 8;
const MAX_BUCKETS: usize = 1 << RADIX;


fn seq_radix_sort_<T: Copy, F: Fn(T) -> DefInt>(
    inp: &mut [T],
    out: &mut [T],
    g: &F,
    bits: usize,
    inplace: bool
) {
    let n = inp.len();
    if n == 0 { return; }
    let mut counts = [0; MAX_BUCKETS + 1];
    let mut swapped = false;
    let mut bit_offset = 0;
    let mut bits = bits;
    while bits > 0 {
        let round_bits = RADIX.min(bits);
        let num_buckets = 1 << round_bits;
        let mask = num_buckets - 1;

        if swapped {
            let keys: Vec<_> = (0..n)
                .map(|i| (g(out[i]) >> bit_offset) & mask)
                .collect();
            seq_count_sort_(out, inp, &keys, &mut counts, num_buckets as usize);
        }
        else {
            let keys: Vec<_> = (0..n)
                .map(|i| (g(inp[i]) >> bit_offset) & mask)
                .collect();
            seq_count_sort_(inp, out, &keys, &mut counts, num_buckets as usize);
        }

        bits = bits - round_bits;
        bit_offset = bit_offset + round_bits;
        swapped = !swapped;
    }

    if swapped && inplace { inp.copy_from_slice(out); }
    else if !swapped && !inplace { out.copy_from_slice(inp); }
}


pub fn seq_radix_sort<T: Copy, F: Fn(T) -> DefInt>(
    inp: &[T],
    out: &mut [T],
    tmp: &mut [T],
    g: &F,
    key_bits: usize
) {
    let odd = ((key_bits - 1) / RADIX) & 1 == 1;
    if odd {
        if tmp.as_ptr() != inp.as_ptr() { tmp.copy_from_slice(inp); }
        seq_radix_sort_(tmp, out, g, key_bits, false)
    } else {
        if out.as_ptr() != inp.as_ptr() { out.copy_from_slice(inp); }
        seq_radix_sort_(out, tmp, g, key_bits, true)
    }
}

pub fn integer_sort_r<T, F>(
    inp: &[T],
    out: &mut [T],
    tmp: &mut [T],
    g: &F,
    key_bits: usize,
    num_buckets: usize,
    parallelism: f32
) -> Vec<DefInt> where
    F: Fn(T) -> DefInt + Sync + Send,
    T: Copy + Send + Sync,
{
    let n = inp.len();
    let cache_per_thread = 1000000usize;
    let sz = 2 * size_of::<T>() * n / cache_per_thread;
    let base_bits = if sz > 0 { log2_up(sz) } else { 0 };
    // keep between 8 and 13
    let base_bits = 8.max(13.min(base_bits));
    let return_offsets = num_buckets > 0;

    if key_bits == 0 {
        out.copy_from_slice(inp);
        return vec![];
    }
    // sequential sort for small inputs or small parallelism
    else if (n < 1 << 17 || parallelism < 0.0001) && !return_offsets {
        seq_radix_sort(inp, out, tmp, g, key_bits);
        return vec![];
    }
    // single parallel count sort for few bits
    else if key_bits <= base_bits {
        let mask = (1 << key_bits) - 1;
        let get_bits: Vec<_> = inp
            .into_par_iter()
            .map(|&i| g(i) & mask)
            .collect();
        let num_bkts = if num_buckets == 0 {1 << key_bits} else {num_buckets};

        let (offsets, _) = count_sort(inp, out, &get_bits, num_bkts, parallelism);

        if return_offsets {return offsets;} else {return vec![]};
    }
    else { // recursive case:
        let bits = 8;
        let shift_bits = key_bits - bits;
        let num_outer_buckets = 1usize << bits;
        let num_inner_buckets =
            if return_offsets { 1usize << shift_bits } else { 0 };
        let mask = num_outer_buckets as DefInt - 1;
        let f = |i: usize| { (g(inp[i]) >> shift_bits) & mask };
        let get_bits = (0..n).into_par_iter().map(f).collect::<Vec<_>>();

        let (offsets, one_bucket) = count_sort(
            inp, out, &get_bits, num_outer_buckets, 1.0);

        // if all but one bucket are empty, try again on lower bits
        if one_bucket {
            return integer_sort_r(inp, out, tmp, g, shift_bits, 0, parallelism);
        }

        let mut inner_offsets =
            vec![0; if return_offsets { num_buckets + 1 } else { 0 }];
        if return_offsets { inner_offsets[num_buckets] = n as DefInt; }

        let mut helper_vec = vec![0; offsets.len()];
        let iter = out.par_ind_chunks_mut(&offsets);
        let iter = if num_inner_buckets > 0 {
            iter.zip(inner_offsets.par_chunks_mut(num_inner_buckets))
        } else {
            iter.zip(helper_vec.par_chunks_mut(1))
        };
        iter
            .zip(tmp.par_ind_chunks_mut(&offsets))
            .zip(offsets.par_iter())
            .with_gran(1)
            .for_each(|(((oc, ioc), tc), oi)| {
                let r = integer_sort_r(
                    unsafe { from_raw_parts(oc.as_ptr(), oc.len()) },
                    tc,
                    oc,
                    g,
                    shift_bits,
                    num_inner_buckets,
                    (parallelism * oc.len() as f32) / (n+1) as f32
                );

                if return_offsets {
                    ioc
                        .iter_mut()
                        .zip(r.iter())
                        .for_each(|(io, ri)| *io = oi + ri );
                }
            });

        return inner_offsets;
    }
}

pub fn integer_sort_<T, F>(
    inp: &[T],
    out: &mut [T],
    tmp: &mut [T],
    get_key: &F,
    mut bits: usize,
    num_buckets: usize
) -> Vec<DefInt> where
    F: Fn(T) -> DefInt + Sync + Send,
    T: Copy + Send + Sync,
{
    if bits == 0 {
        bits = log2_up(
            inp
                .par_iter()
                .map(|&k| get_key(k) as DefInt)
                .max()
                .unwrap() as usize
            );
    }
    integer_sort_r(inp, out, tmp, get_key, bits, num_buckets, 1.0)
}

pub fn integer_sort<T, F>(
    inp: &[T],
    get_key: &F,
    bits: usize,
    out: &mut Vec<T>
) where
    F: Fn(T) -> DefInt + Sync + Send,
    T: Copy + Send + Sync,
{
    if inp.len() == 0 {
        *out = vec![];
    } else {
        let mut tmp = maybe_uninit_vec![inp[0]; inp.len()];
        *out = maybe_uninit_vec![inp[0]; inp.len()];
        integer_sort_(inp, out, &mut tmp, get_key, bits, 0);
    }
}
