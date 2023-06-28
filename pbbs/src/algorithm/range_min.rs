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

use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU32;
use rayon::prelude::*;
use num_traits::PrimInt;
use num_traits::cast::{FromPrimitive, ToPrimitive};

use parlay::utilities::log2_up;
use crate::ORDER;


#[allow(dead_code)]
pub struct RangeMin<'a, T, F> where
{
    arr: &'a [T],
    table: Vec<Vec<T>>,
    less: F,
    block_size: usize,
}

impl<'a, T, F> RangeMin<'a, T, F> where
    T: PrimInt + FromPrimitive + Send + Sync,
    F: Fn(&T, &T) -> bool + Clone + Copy + Send + Sync,
{
    #[inline(always)]
    fn min_idx(&self, i: T, j: T) -> T where
    {
        if (self.less)(
            &self.arr[j.to_usize().unwrap()],
            &self.arr[i.to_usize().unwrap()]
        ) { j } else { i }
    }

    pub fn new(a: &'a [T], less: F, block_size: usize) -> Self {
        let n = a.len();
        let m = 1 + (n-1) / block_size;
        let depth = log2_up(m + 1);
        let mut table: Vec<_> = (0..depth)
            .map(|_| vec![unsafe{MaybeUninit::uninit().assume_init()}; m])
            .collect();
        let min_idx = |i: T, j: T| -> T {
            if (less)(
                &a[j.to_usize().unwrap()],
                &a[i.to_usize().unwrap()]
            ) { j } else { i }
        };
        // minimums within each block
        let l = n / block_size;
        (&mut table[0][..l+1])
            .into_par_iter()
            .enumerate()
            .for_each(|(i, ti)| {
                let s = i * block_size;
                let e = n.min(s + block_size);
                let mut k = T::from_usize(s).unwrap();
                for j in s + 1..e { k = min_idx(T::from_usize(j).unwrap(), k); }
                *ti = k;
            });

        // minimum across layers
        let mut dist = 1;
        for j in 1..depth {
            let (l, r) = table.split_at_mut(j);
            let tj = &mut r[0];
            let tj_1 = & l[j-1];
            tj[..m-dist].par_iter_mut().enumerate().for_each(|(i, t)|
                *t = min_idx(tj_1[i], tj_1[i+dist])
            );
            tj[m-dist..m].par_iter_mut().enumerate().for_each(|(i, t)|
                *t = tj_1[i]
            );
            dist *= 2;
        }

        Self { arr: a, table, less, block_size }
    }

    pub fn query(&self, i: T, j: T) -> T {
        let bls = T::from_usize(self.block_size).unwrap();

        // same or adjacent blocks
        if j - i < bls {
            let mut r = i;
            let mut k = i + T::one();
            while k <= j {
                r = self.min_idx(r, k);
                k = k + T::one();
            }
            return r;
        }
        let mut block_i = i / bls;
        let mut block_j = j / bls;
        let mut minl = i;

        // min suffix of first block
        let mut k = minl + T::one();
        while k < (block_i + T::one()) * bls {
            minl = self.min_idx(minl, k);
            k = k + T::one();
        }

        //min prefix of last block
        let mut minr = block_j * bls;
        let mut k = minr + T::one();
        while k <= j {
            minr = self.min_idx(minr, k);
            k = k + T::one();
        }

        // if adjacent, then done
        if block_j == block_i + T::one() {
            return self.min_idx(minl, minr);
        }

        let out_of_block_min;
        block_i = block_i.add(T::one());
        block_j = block_j.sub(T::one());
        if block_i == block_j {
            out_of_block_min = self.table[0][block_i.to_usize().unwrap()];
        } else if block_j == block_i + T::one() {
            out_of_block_min = self.table[1][block_i.to_usize().unwrap()];
        } else {
            let k = log2_up((block_j-block_i+T::one()).to_usize().unwrap()) - 1;
            let p = 1 << k;
            out_of_block_min = self.min_idx(
                self.table[k][block_i.to_usize().unwrap()],
                self.table[k][block_j.to_usize().unwrap() + 1 - p]
            );
        }

        self.min_idx(minl, self.min_idx(out_of_block_min, minr))
    }
}



// The atomic version:
#[allow(dead_code)]
pub struct AtomU32RangeMin<'a, F>
{
    arr: &'a [AtomicU32],
    table: Vec<Vec<u32>>,
    less: F,
    block_size: usize,
}

impl<'a, F> AtomU32RangeMin<'a, F> where
    F: Fn(&u32, &u32) -> bool + Clone + Copy + Send + Sync,
{
    #[inline(always)]
    fn min_idx(&self, i: u32, j: u32) -> u32 where
    {
        if (self.less)(
            &self.arr[j as usize].load(ORDER),
            &self.arr[i as usize].load(ORDER)
        ) { j } else { i }
    }

    pub fn new(a: &'a [AtomicU32], less: F, block_size: usize) -> Self {
        let n = a.len();
        let m = 1 + (a.len() - 1) / block_size;
        let depth = log2_up(m + 1);
        #[allow(invalid_value)]
        let mut table: Vec<_> = (0..depth)
            .map(|_| vec![unsafe{MaybeUninit::uninit().assume_init()}; m])
            .collect();

        let min_idx = |i: u32, j: u32| -> u32 {
            if (less)(
                &a[j as usize].load(ORDER),
                &a[i as usize].load(ORDER)
            ) { j } else { i }
        };

        // minimums within each block
        let l = n / block_size;
        (&mut table[0][..l+1])
            .into_par_iter()
            .enumerate()
            .for_each(|(i, ti)| {
                let s = i * block_size;
                let e = n.min(s + block_size);
                let mut k = s as u32;
                for j in s+1..e { k = min_idx(j as u32, k); }
                *ti = k;
            });

        // minimum across layers
        let mut dist = 1;
        for j in 1..depth {
            let (l, r) = table.split_at_mut(j);
            let tj = &mut r[0];
            let tj_1 = & l[j-1];
            tj[..m-dist].par_iter_mut().enumerate().for_each(|(i, t)|
                *t = min_idx(tj_1[i], tj_1[i+dist])
            );
            tj[m-dist..m].par_iter_mut().enumerate().for_each(|(i, t)|
                *t = tj_1[i]
            );
            dist *= 2;
        }

        Self { arr: a, table, less, block_size }
    }

    pub fn query(&self, i: u32, j: u32) -> u32 {
        let bls = self.block_size as u32;

        // same or adjacent blocks
        if j - i < bls {
            let mut r = i;
            for k in i+1..j+1 { r = self.min_idx(r, k); }
            return r;
        }
        let mut block_i = i/bls;
        let mut block_j = j/bls;
        let mut minl = i;

        // min suffix of first block
        for k in minl+1 .. (block_i+1) * bls {
            minl = self.min_idx(minl, k);
        }

        //min prefix of last block
        let mut minr = block_j * bls;
        for k in minr+1 .. j+1 {
            minr = self.min_idx(minr, k);
        }

        // if adjacent, then done
        if block_j == block_i + 1 {
            return self.min_idx(minl, minr);
        }

        let out_of_block_min;
        block_i = block_i + 1;
        block_j = block_j - 1;
        if block_i == block_j {
            out_of_block_min = self.table[0][block_i.to_usize().unwrap()];
        } else if block_j == block_i + 1 {
            out_of_block_min = self.table[1][block_i.to_usize().unwrap()];
        } else {
            let k = log2_up((block_j-block_i+1).to_usize().unwrap()) - 1;
            let p = 1 << k;
            out_of_block_min = self.min_idx(
                self.table[k][block_i.to_usize().unwrap()],
                self.table[k][block_j.to_usize().unwrap() + 1 - p]
            );
        }

        self.min_idx(minl, self.min_idx(out_of_block_min, minr))
    }
}
