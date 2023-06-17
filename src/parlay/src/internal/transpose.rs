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

use crate::maybe_uninit_vec;
use crate::utilities::log2_up;
use crate::internal::sequence_ops::scan_inplace;

const TRANS_THRESHHOLD: usize = 500;
const NON_CACHE_OBLIVIOUS_THRESHOLD: usize = 1 << 22;


#[inline(always)]
fn split(n: usize) -> usize { n / 2 }

struct Transpose<'a, T> { a: &'a [T], b: &'a mut [T] }

impl<'a, T: Copy + Send + Sync> Transpose<'a, T>
{
    fn new(a: &'a [T], b: &'a mut [T]) -> Self { Self { a, b } }

    fn trans_r(
        &mut self,
        r_start: usize,
        r_count: usize,
        r_length: usize,
        c_start: usize,
        c_count: usize,
        c_length: usize
    ) {
        #[cfg(feature = "AW_safe")]
        panic!("AW_safe is not supported for transpose");

        #[cfg(not(feature = "AW_safe"))]
        if c_count * r_count < TRANS_THRESHHOLD {
            for i in r_start..r_start+r_count {
                for j in c_start..c_start+c_count {
                    self.b[j*c_length + i] = self.a[i*r_length + j];
                }
            }
        } else if c_count > r_count {
            let l1 = split(c_count);
            let l2 = c_count - l1;
            let self_shadow = unsafe { (self as *mut Self).as_mut().unwrap() };
            rayon::join(
                || self.trans_r(r_start, r_count, r_length, c_start, l1, c_length),
                || self_shadow.trans_r(r_start, r_count, r_length, c_start + l1, l2, c_length)
            );
        } else {
            let l1 = split(c_count);
            let l2 = r_count - l1;
            let self_shadow = unsafe { (self as *mut Self).as_mut().unwrap() };
            rayon::join(
                || self.trans_r(r_start, l1, r_length, c_start, c_count, c_length),
                || self_shadow.trans_r(r_start + l1, l2, r_length, c_start, c_count, c_length)
            );
        }
    }

    fn trans(&mut self, r_count: usize, c_count: usize) {
        self.trans_r(0, r_count, c_count, 0, c_count, r_count);
    }
}

struct BlockTrans<'a, T> {
    a: &'a [T],
    b: &'a mut [T],
    oa: &'a [usize],
    ob: &'a [usize]
}

impl<'a, T: Copy + Send + Sync> BlockTrans<'a, T>
{
    fn new(a: &'a [T], b: &'a mut [T], oa: &'a [usize], ob: &'a [usize]) -> Self
    {
        Self { a, b, oa, ob }
    }

    fn trans_r(
        &mut self,
        r_start : usize,
        r_count : usize,
        r_length: usize,
        c_start : usize,
        c_count : usize,
        c_length: usize
    ) {
        if c_count * r_count < TRANS_THRESHHOLD * 16 {
            let b_ptr = self.b.as_mut_ptr() as usize;
            (r_start..r_start+r_count).into_par_iter().for_each(|i| {
                for j in c_start..c_start+c_count {
                    let sa = self.oa[i*r_length + j];
                    let sb = self.ob[j*c_length + i];
                    let l = self.oa[i*r_length + j + 1] - sa;
                    for k in 0..l { unsafe {
                        (b_ptr as *mut T).add(sb + k).write(self.a[sa + k]);
                    }}
                }
            });
        } else if c_count > r_count {
            let l1 = split(c_count);
            let l2 = c_count - l1;
            let self_shadow = unsafe { (self as *mut Self).as_mut().unwrap() };
            rayon::join(
                || self_shadow.trans_r(r_start, r_count, r_length, c_start, l1, c_length),
                || self.trans_r(r_start, r_count, r_length, c_start + l1, l2, c_length)
            );
        } else {
            let l1 = split(c_count);
            let l2 = r_count - l1;
            let self_shadow = unsafe { (self as *mut Self).as_mut().unwrap() };
            rayon::join(
                || self_shadow.trans_r(r_start, l1, r_length, c_start, c_count, c_length),
                || self.trans_r(r_start + l1, l2, r_length, c_start, c_count, c_length)
            );
        }
    }

    fn trans(&mut self, r_count: usize, c_count: usize) {
        self.trans_r(0, r_count, c_count, 0, c_count, r_count);
    }
}

pub(crate) fn transpose_buckets<T: Copy + Send + Sync>(
    from: &[T],
    to: &mut [T],
    counts: &mut [usize],
    offsets: &mut Vec<usize>,
    n: usize,
    block_size: usize,
    num_blocks: usize,
    num_buckets: usize
) {
    let m = num_buckets * num_blocks;
    let mut dest_offsets: Vec<usize>;

    // for smaller input do non-cache oblivious version
    if n < NON_CACHE_OBLIVIOUS_THRESHOLD
        || num_buckets <= 512
        || num_blocks <= 512
    {
        let block_bits = log2_up(num_blocks);
        let block_mask = num_blocks - 1;

        // determine the destination offsets
        dest_offsets = (0..m)
            .into_par_iter()
            .map(|i|
                counts[(i >> block_bits) + num_buckets * (i & block_mask)]
            ).collect();

        let _sum = scan_inplace(&mut dest_offsets, false, |a, b| a + b);
        debug_assert_eq!(_sum, n);

        // send each key to correct location within its bucket
        let to_ptr = to.as_mut_ptr() as usize;
        (0..num_blocks).into_par_iter().for_each(|i| {
            let mut s_offset = i * block_size;
            for j in 0..num_buckets {
                let mut d_offset = dest_offsets[i + num_blocks * j];
                let len = counts[i * num_buckets + j];
                for _ in 0..len {
                    unsafe {
                        (to_ptr as *mut T).add(d_offset).write(from[s_offset]);
                    }
                    d_offset+=1; s_offset+=1;
                }
            }
        });
    } else {    // for larger input do cache efficient transpose
        dest_offsets = maybe_uninit_vec![0; m];
        Transpose::new(counts, &mut dest_offsets)
            .trans(num_blocks, num_buckets);

        let _total = scan_inplace(&mut dest_offsets, false, |a, b| a + b);
        let _total2 = scan_inplace(counts, false, |a, b| a + b);
        debug_assert_eq!(_total, n); debug_assert_eq!(_total2, n);

        counts[m] = n;

        BlockTrans::new(from, to, counts, &dest_offsets)
            .trans(num_blocks, num_buckets);
    }
    // return the bucket offsets, padded with n at the end
    *offsets = (0..num_buckets+1)
        .into_par_iter()
        .map(|i|
            if i == num_buckets { n } else { dest_offsets[i*num_blocks] }
        ).collect();
}
