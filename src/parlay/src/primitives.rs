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

use rayon::prelude::*;
use rayon::iter::Map;
use rayon::range::Iter;
use num_traits::PrimInt;
use enhanced_rayon::prelude::*;

use crate::maybe_uninit_vec;
use crate::internal::sequence_ops::*;


/* -------------------- Pack -------------------- */
fn pack_serial_at<T, F>(arr_f: F, flags:&[bool], dest: &mut [T]) -> usize
where
    T: Copy,
    F: Fn(usize) -> T,
{
    let mut k = 0;
    let n = flags.len();
    for i in 0..n {
        if flags[i] {
            dest[k] = arr_f(i);
            k += 1;
        }
    }
    k
}

pub fn pack_serial<T, F>(arr_f: F, flags:&[bool], dest: &mut Vec<T>)
where
    T: Copy + Clone,
    F: Fn(usize) -> T,
{
    let m = sum_bool_serial(flags);
    *dest = vec![arr_f(0); m];
    pack_serial_at(arr_f, flags, dest);
}

fn pack_helper<T, F>(arr_f: F, flags:&[bool], dest: &mut Vec<T>)
where
    T: Copy + Clone + Send + Sync,
    F: Fn(usize) -> T + Send + Sync,
{
    let n = flags.len();
    let bls = _BLOCK_SIZE * 10;
    let block_no =  num_blocks(n, bls);
    if block_no == 1 {
        pack_serial(arr_f, flags, dest);
        return;
    }

    let mut sums: Vec<usize> = flags
        .par_chunks(bls)
        .map(|chunk| sum_bool_serial(chunk))
        .collect();

    let m = scan_inplace(&mut sums, false, |a, b| a + b);

    *dest = maybe_uninit_vec![arr_f(0); m];

    dest
        .par_ind_chunks_mut(&sums)
        .zip(flags.par_chunks(bls))
        .enumerate()
        .for_each(|(i, (out_chunk, flag_chunk))| {
            let s = i * bls;
            let arr_slice_f = |i| arr_f(s + i);
            pack_serial_at(arr_slice_f, flag_chunk, out_chunk);
        });
}

pub fn pack<T>(arr: &[T], flags:&[bool], dest: &mut Vec<T>)
where
    T: Copy + Send + Sync + Clone
{
    if arr.len() > 0 {
        let arr_f = |i| arr[i];
        pack_helper(arr_f, flags, dest);
    } else { *dest = vec![]; }
}


pub fn pack_index<T>(flags: &[bool], dest: &mut Vec<T>)
where
    T: Copy + Send + Sync + Clone + PrimInt
{
    debug_assert_ne!(flags.len(), 0);
    let arr_f = |i| T::from(i).expect("pack_index: invalid conversion");
    pack_helper(arr_f, flags, dest);
}


// non copy version of pack:
// =========================
unsafe fn nc_pack_serial_at<T, F>(arr_f: F, flags:&[bool], dest: &mut [T])
where
    F: Fn(usize, *mut T),
{
    let mut k = 0;
    for i in 0..flags.len() {
        if flags[i] {
            arr_f(i, &mut dest[k]);
            k += 1;
        }
    }
}

pub unsafe fn nc_pack_serial<T, F>(arr_f: F, flags:&[bool], dest: &mut Vec<T>)
where
    F: Fn(usize, *mut T),
{
    let m = sum_bool_serial(flags);
    *dest = maybe_uninit_vec![T::default(); m];
    nc_pack_serial_at(arr_f, flags, dest);
}

unsafe fn nc_pack_helper<T, F>(arr_f: F, flags:&[bool], dest: &mut Vec<T>)
where
    T: Send + Sync,
    F: Fn(usize, *mut T) + Send + Sync,
{
    let n = flags.len();
    let bls = _BLOCK_SIZE * 10;
    let block_no =  num_blocks(n, bls);
    if block_no == 1 { nc_pack_serial(arr_f, flags, dest); return; }

    let mut sums: Vec<usize> = flags
        .par_chunks(bls)
        .map(|chunk| sum_bool_serial(chunk))
        .collect();
    let m = scan_inplace(&mut sums, false, |a, b| a + b);

    *dest = maybe_uninit_vec![T::default(); m];

    dest
        .par_ind_chunks_mut(&sums)
        .zip(flags.par_chunks(bls))
        .enumerate()
        .for_each(|(i, (out_chunk, flag_chunk))| {
            let s = i * bls;
            let arr_slice_f = |i, d| arr_f(s + i, d);
            nc_pack_serial_at(arr_slice_f, flag_chunk, out_chunk);
        });
}

pub unsafe fn nc_pack<T>(arr: &[T], flags:&[bool], dest: &mut Vec<T>)
where
    T: Send + Sync
{
    if arr.len() == 0 { *dest = vec![]; }
    else {
        let arr_f = |i, d: *mut T| {std::ptr::copy(&arr[i] as *const T, d, 1)};
        nc_pack_helper(arr_f, flags, dest);
    }
}

/* -------------------- Flatten -------------------- */

pub fn flatten<T>(arr: &[&Vec<T>], dest: &mut Vec<T>)
where
    T: Copy + Send + Sync + Default,
{
    let n = arr.len();
    let mut offsets: Vec<_> = (0..n).into_par_iter().map(|i| arr[i].len()).collect();
    let len = scan_inplace(&mut offsets, false, |a, b| a + b);

    *dest = maybe_uninit_vec![T::default(); len];
    dest
        .par_ind_chunks_mut(&offsets)
        .zip(arr.par_iter())
        .for_each(|(out_chunk, a)| {
            (*a, out_chunk)
                .into_par_iter()
                .with_gran(1024)
                .for_each(|(ai, oi)| *oi = *ai);
        });
}

pub fn flatten_by_val<T>(arr: &[Vec<T>], dest: &mut Vec<T>) where
    T: Copy + Send + Sync + Default,
{
    let ref_arr: Vec<_> = arr.iter().map(|a| a).collect();
    flatten(&ref_arr, dest);
}


/* -------------------- Tokens and split -------------------- */

pub fn map_tokens<'a, R, G>(r: &'a [R], is_space: G, res: &mut Vec<&'a [R]>)
where
    R: Default + Copy + Send + Sync,
    G: Fn(&R) -> bool + Copy + Send + Sync
{
    type Ipair = (i64, i64);
    let n = r.len() - 1;

    if n == 0 {
        *res = vec![];
        return;
    }

    let is_start = |i: usize|
        ((i == 0) || is_space(&r[i - 1])) && (i != n) && !is_space(&r[i    ]);
    let is_end   = |i: usize|
        ((i == n) || is_space(&r[i    ])) && (i != 0) && !is_space(&r[i - 1]);

    // associative combining function
    // first = # of starts, second = index of last start
    let g = |a: Ipair, b: Ipair| {
        if b.0 == 0 {a} else {(a.0 + b.0, b.1)}
    };

    let in_vec: Map<Iter<usize>, _> = (0..n+1)
        .into_par_iter()
        .map(|i| { if is_start(i) {(1, i as i64)} else {(0, 0)} });

    let l = num_blocks(n+1, _BLOCK_SIZE);

    let mut sums: Vec<Ipair>;
    sums = maybe_uninit_vec![Ipair::default(); l];
    let sum = scan_delayed(in_vec.clone(), &mut sums, g);

    let z = in_vec.chunks(_BLOCK_SIZE).zip(sums);

    #[cfg(AW_safe)]
    {
        let offsets: Vec<Ipair> = z.map(|(arr, sum_p)| {
            let mut pair = sum_p;
            let n = arr.len();
            for i in 0..n {
                pair = g(pair, arr[i]);
            }
            pair
        }).collect();
        *res = offsets
            .into_par_iter()
            .enumerate()
            .filter_map(|x| {
                if is_end(x.0) { Some(&r[(x.1.1 as usize)..(x.0)]) }
                else { None }
            }).collect();
    }
    #[cfg(not(AW_safe))]
    {
        *res = Vec::with_capacity(sum.0 as usize);
        unsafe{ res.set_len(sum.0 as usize); }
        let res_ptr = res.as_ptr() as usize;

        z.enumerate().for_each(|(b, (arr, sum_p))| {
            let mut pair = sum_p;
            let n = arr.len();
            for i in 0..n {
                pair = g(pair, arr[i]);
                //result
                let x0 = b * _BLOCK_SIZE + i;
                if is_end(x0) {
                    unsafe {
                        (res_ptr as *mut &[R])
                            .add(pair.0 as usize - 1)
                            .write(&r[(pair.1 as usize)..(x0 as usize)]);
                    }}
            }
        });
    }
}
