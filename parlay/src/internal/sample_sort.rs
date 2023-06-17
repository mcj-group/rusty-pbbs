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
use rayon::prelude::*;
use enhanced_rayon::prelude::*;

use crate::maybe_uninit_vec;
use crate::utilities::{log2_up, hash64};
use crate::internal::{quick_sort, bucket_sort};
use crate::internal::transpose::transpose_buckets;


const QUICKSORT_THRESHOLD: usize = 16384;
const OVER_SAMPLE: usize = 8;


fn get_bucket_counts<T, F>(
    arr: &[T],
    pivots: &[T],
    counts: &mut[usize],
    less: F
) where
    T: Copy,
    F: Fn(T, T) -> bool + Copy,
{
    if arr.len() == 0 || pivots.len() == 0 { return; }
    counts.iter_mut().for_each(|i| *i=0);
    let (mut ai, mut pi, mut ci) = (0, 0, 0);
    let (a_end, p_end, c_end) = (arr.len(), pivots.len(), counts.len());
    loop {
        while less(arr[ai], pivots[pi]) {
            debug_assert_ne!(ci, c_end);
            counts[ci] += 1;
            ai += 1; if ai == a_end { return; }
        }
        pi += 1; ci += 1;
        if pi == p_end { break; }
        if !less(pivots[pi-1], pivots[pi]) {
            while !less(pivots[pi], arr[ai]) {
                debug_assert_ne!(ci, c_end);
                counts[ci] += 1;
                ai += 1; if ai == a_end { return; }
            }
            pi += 1; ci += 1;
            if pi == p_end { break; }
        }
    }
    debug_assert_ne!(ci, c_end);
    counts[ci] = a_end - ai;
}

fn seq_sort_inplace<T, F>(inp: &mut [T], less: F, stable: bool)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    if size_of::<T>() > 8 {
        if !stable { quick_sort(inp, less); }
        else { bucket_sort(inp, less, true); }
    }
    else { bucket_sort(inp, less, stable); }
}

fn seq_sort_<T, F>(inp: &[T], out: &mut [T], less: F, stable: bool)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    out.copy_from_slice(inp);
    seq_sort_inplace(out, less, stable);
}

pub fn sample_sort<T, F>(inp: &[T], out: &mut [T], less: F, stable: bool)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    let n = inp.len();
    if n < QUICKSORT_THRESHOLD {
        seq_sort_(inp, out, less, stable);
    } else {
        let (bucket_quotient, block_quotient) =
            if size_of::<T>() > 8 { (3, 3) } else { (4, 4) };
        let sqrt = f64::sqrt(n as f64) as usize;
        let num_blocks = 1 << log2_up((sqrt / block_quotient) + 1);
        let block_size = ((n-1) / num_blocks) + 1;
        let num_buckets = (sqrt / bucket_quotient) + 1;
        let sample_set_size = num_buckets * OVER_SAMPLE;
        let m = num_blocks * num_buckets;

        // generate and sort random samples
        let mut sample_set: Vec<_> = (0..sample_set_size as u64)
            .into_par_iter()
            .map(|i| inp[hash64(i) as usize % n])
            .collect();
        quick_sort(&mut sample_set, less);
        let pivots: Vec<T> = (0..num_buckets-1)
            .map(|i| sample_set[i * OVER_SAMPLE])
            .collect();

        let mut tmp = maybe_uninit_vec![T::default(); n];
        let mut counts = maybe_uninit_vec![0usize; m+1];
        counts[m] = 0;

        // sort each block and merge with samples to get counts for each bucket
        (
            (&inp).par_chunks(block_size),
            (&mut tmp).par_chunks_mut(block_size),
            (&mut counts).par_chunks_mut(num_buckets)
        )
            .into_par_iter()
            .for_each(|(inp, tmp, cnt)| {
                seq_sort_(&inp, tmp, less, stable);
                get_bucket_counts(tmp, &pivots, cnt, less);
            });

        // move data from blocks to buckets
        let mut bucket_offsets = Vec::<usize>::new();
        transpose_buckets(
            &tmp,
            out,
            &mut counts,
            &mut bucket_offsets,
            n,
            block_size,
            num_blocks,
            num_buckets
        );

        // sort within each bucket
        out
            .par_ind_chunks_mut(&bucket_offsets[..num_buckets])
            .enumerate()
            .for_each(|(i, out)| {
                if i==0 || i==num_buckets-1 || less(pivots[i-1], pivots[i]) {
                    seq_sort_inplace(out, less, stable);
                }
            });
    }
}

pub fn sample_sort_inplace<T, F>(arr: &mut [T], less: F, stable: bool)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    let a_shadow = unsafe { (arr as *const [T]).as_ref().unwrap() };
    sample_sort(a_shadow, arr, less, stable);
}
