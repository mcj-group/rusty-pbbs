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

use enhanced_rayon::prelude::*;
use crate::maybe_uninit_vec;
use crate::internal::{quick_sort, merge_sort_};
use crate::utilities::hash64;


fn radix_step<T: Copy>(
    inp: &[T],
    out: &mut [T],
    keys: &[u8],
    counts: &mut [usize]
) {
    counts.iter_mut().for_each(|c| *c=0);
    keys.iter().for_each(|&k| counts[k as usize] += 1);
    let mut s = 0;
    counts.iter_mut().for_each(|c| { s += *c; *c = s; });
    keys
        .iter()
        .zip(inp.iter())
        .for_each(|(&k, &v)| {
            let c = &mut counts[k as usize];
            *c -= 1;
            out[*c] = v;
        });
}

/// Given the sequence In[l, r), write to Out, the values of
/// In arranged in a balanced tree. Values are indexed using
/// the standard implicit tree ordering (a la binary heaps)
///
/// //                   root (i)
/// //                 /          \
/// //           left (2i+1)   right(2i+2)
///
/// i.e. the root of the tree is at index 0, and the left
/// and right children of the node at index i are at indices
/// 2i+1 and 2i+2 respectively.
fn to_balanced_tree<T: Copy>(
    inp: &[T],
    out: &mut [T],
    root: usize,
    l: usize,
    r: usize
) {
    let n = r - l;
    let m = l + n / 2;
    out[root] = inp[m];
    if n == 1 { return; }
    to_balanced_tree(inp, out, 2 * root + 1, l, m);
    to_balanced_tree(inp, out, 2 * root + 2, m + 1, r);
}

// returns true if all equal
fn get_buckets<T, F>(
    inp: &[T],
    buckets: &mut [u8],
    less: F,
    rounds: usize
) -> bool where
    T: Copy + Sync,
    F: Fn(T, T) -> bool + Copy + Sync,
{
    let n = inp.len();
    let num_buckets = 1 << rounds;
    let over_sample = 1 + n / (num_buckets * 400);
    let sample_set_size = num_buckets * over_sample;
    let num_pivots = num_buckets - 1;

    // choosing random pivots
    let mut sample_set: Vec<_> = (0..sample_set_size)
        .map(|i| hash64(i as u64) as usize % n)
        .collect();

    // sort the samples
    quick_sort(&mut sample_set, |a, b| less(inp[a], inp[b]));
    
    let pivots: Vec<_> = (0..num_pivots)
        .map(|i| sample_set[over_sample * (i+1)])
        .collect();
    if !less(inp[pivots[0]], inp[pivots[num_pivots-1]]) { return true; }

    let pivot_tree = &mut sample_set;
    to_balanced_tree(&pivots, pivot_tree, 0, 0, num_pivots);

    assert!(n <= buckets.len());
    for i in 0..n {
        let mut j = 0;
        for _ in 0..rounds {
            j = 1 + 2 * j + (!less(inp[i], inp[pivot_tree[j]])) as usize;
        }
        debug_assert!(j - num_pivots <= u8::MAX as usize);
        buckets[i] = (j - num_pivots) as u8;
    }
    false
}

fn base_sort<T, F>(
    inp: &mut [T],
    out: &mut [T],
    less: F,
    stable: bool,
    inplace: bool
) where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    if stable {
        merge_sort_(inp, out, less, inplace);
    } else {
        quick_sort(inp, less);
        if !inplace { out.copy_from_slice(inp); }
    }
}

pub fn bucket_sort_r<T, F>(
    inp: &mut [T],
    out: &mut [T],
    less: F,
    stable: bool,
    inplace: bool
) where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    let n = inp.len();
    const BITS: usize = 4;
    let num_buckets = 1 << BITS;
    if n < num_buckets * 32 {
        base_sort(inp, out, less, stable, inplace);
    } else {
        let mut counts = maybe_uninit_vec![0usize; num_buckets];
        let mut buckets = maybe_uninit_vec![0u8; n];
        if get_buckets(inp, &mut buckets, less, BITS) {
            base_sort(inp, out, less, stable, inplace);
        } else {
            radix_step(inp, out, &mut buckets, &mut counts);
            out
                .par_ind_chunks_mut(&counts)
                .zip(inp.par_ind_chunks_mut(&counts))
                .for_each(|(o, i)| {
                    bucket_sort_r(o, i, less, stable, !inplace);
                });
        }
    }
}

pub fn bucket_sort<T, F>(inp: &mut [T], less: F, stable: bool)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    if inp.len() < 2 { return; }
    let n = inp.len();
    let mut tmp = maybe_uninit_vec![inp[0]; n];
    bucket_sort_r(inp, &mut tmp, less, stable, true);
}
