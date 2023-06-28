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
use rayon;
use rayon::prelude::*;
use num_traits::cast::ToPrimitive;

use crate::{DefInt, maybe_uninit_vec};
use crate::internal::sequence_ops::scan_inplace;

const SEQ_THRESHOLD: usize = 8192;


fn seq_count_<T: Copy, F: ToPrimitive>(
    inp: &[T],
    keys: &[F],
    counts: &mut [DefInt],
    num_buckets: usize
) {
    let n = inp.len();
    // local counts to avoid false sharing
    let mut lcnt = vec![0; num_buckets];
    keys[..n]
        .iter()
        .for_each(|k| {
            let k = k.to_usize().unwrap();
            debug_assert!(k < num_buckets);
            lcnt[k] += 1;
        });
    counts[..num_buckets]
        .iter_mut()
        .zip(lcnt.iter())
        .for_each(|(a, b)| *a = *b);
}

fn seq_write_<T: Copy, F: ToPrimitive>(
    inp: &[T],
    keys: &[F],
    offsets: &[usize],
    num_buckets: usize,
) {
    let mut local_offsets = maybe_uninit_vec![0; num_buckets];
    local_offsets
        .iter_mut()
        .zip(offsets.iter())
        .for_each(|(a, b)| *a = *b);
    inp
        .iter()
        .zip(keys.iter())
        .for_each(|(i, k)| {
            let k = k.to_usize().unwrap();
            // reading a raw_pointer
            unsafe {
                *(local_offsets[k] as *mut T) = *i;
            }
            local_offsets[k] += size_of::<T>();
        });
}

fn seq_write_down_<T: Copy, F: ToPrimitive>(
    inp: &[T],
    out: &mut [T],
    keys: &[F],
    offsets: &mut [DefInt]
) {
    inp
        .iter()
        .zip(keys.iter())
        .for_each(|(i, k)| {
            let k = k.to_usize().unwrap();
            offsets[k] -= 1;
            out[offsets[k] as usize] = *i;
        });
}

pub(crate) fn seq_count_sort_<T: Copy, F: ToPrimitive>(
    inp: &[T],
    out: &mut [T],
    keys: &[F],
    counts: &mut [DefInt],
    num_buckets: usize
) {
    seq_count_(inp, keys, counts, num_buckets);

    // generate offsets
    let mut s = 0;
    counts[..num_buckets]
        .iter_mut()
        .for_each(|c| {
            s += *c;
            *c = s;
        });

    // send to destination
    seq_write_down_(inp, out, keys, counts);
}

pub fn seq_count_sort<T: Copy, F: ToPrimitive>(
    inp: &[T],
    out: &mut [T],
    keys: &[F],
    num_buckets: usize
) -> Vec<DefInt>
{
    let mut counts = maybe_uninit_vec![0; num_buckets+1];
    seq_count_sort_(inp, out, keys, &mut counts, num_buckets);
    counts[num_buckets] = inp.len() as DefInt;
    return counts;
}

fn count_sort_helper<T, F>(
    inp: &[T],
    out: &mut [T],
    keys: &[F],
    num_buckets: usize,
    _parallelism: f32
) -> (Vec<DefInt>, bool) where
    T: Copy + Send + Sync,
    F: ToPrimitive + Sync,
{
    let n = inp.len();
    if n == 0 { return (vec![], false); }
    let num_threads = rayon::current_num_threads();

    let num_blocks = 1 + n * size_of::<T>() / 5000.max(num_buckets * 500);

    if n < SEQ_THRESHOLD || num_blocks == 1 || num_threads == 1 {
        return (seq_count_sort(inp, out, keys, num_buckets), false);
    }

    let block_size = ((n - 1) / num_blocks) + 1;
    let m = num_blocks * num_buckets;

    let mut counts = maybe_uninit_vec![0; m];

    counts
        .par_chunks_mut(num_buckets)
        .zip(inp.par_chunks(block_size))
        .zip(keys.par_chunks(block_size))
        .for_each(|((cnt_chunk, inp_chunk), keys_chunk)| {
            seq_count_(inp_chunk, keys_chunk, cnt_chunk, num_buckets);
        });

    // aggregate blocks counts and calculate offsets
    let mut bucket_offsets = maybe_uninit_vec![0; num_buckets + 1];
    bucket_offsets[..num_buckets]
        .par_iter_mut()
        .enumerate()
        .for_each(
            |(i, dst)| {
                let mut v = 0;
                for j in 0..num_blocks {v += counts[j * num_buckets + i];}
                *dst = v;
            }
        );
    bucket_offsets[num_buckets] = 0;

    // scan (prefix sum) on offsets array
    let _t = scan_inplace(&mut bucket_offsets, false, |a, b| a + b);
    debug_assert_eq!(_t as usize, n);

    // calculate destination offsets
    let dest_offsets = maybe_uninit_vec![0usize; num_blocks * num_buckets];

    bucket_offsets[..num_buckets]
        .par_iter()
        .enumerate()
        .for_each(|(i, bo)| {
            let mut v = *bo as usize * size_of::<T>() + out.as_ptr() as usize;
            for j in 0..num_blocks {
                unsafe {
                    (dest_offsets.as_ptr() as *mut usize)
                        .add(j * num_buckets + i)
                        .write(v);
                }
                v += counts[j * num_buckets + i] as usize * size_of::<T>();
            }
        });

    // write the results in destination
    inp
        .par_chunks(block_size)
        .zip(keys.par_chunks(block_size))
        .zip(dest_offsets.par_chunks(num_buckets))
        .for_each(|((inp_chunk, keys_chunk), dest_chunk)| {
            seq_write_(inp_chunk, keys_chunk, dest_chunk, num_buckets);
        });

    return (bucket_offsets, false);
}

pub fn count_sort<T, F>(
    inp: &[T],
    out: &mut [T],
    keys: &[F],
    num_buckets: usize,
    parallelism: f32
) -> (Vec<DefInt>, bool) where
    T: Copy + Send + Sync,
    F: ToPrimitive + Sync,
{
    #[cfg(feature = "AW_safe")]
    eprintln!("AW_safe cannot be used with count_sort for now. \
        Switched to unsafe AW.");

    let n = inp.len();
    debug_assert_eq!(n, out.len());
    debug_assert_eq!(n, keys.len());
    count_sort_helper(inp, out, keys, num_buckets, parallelism)
}
