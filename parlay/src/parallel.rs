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

use std::{ops::Range, sync::atomic::{AtomicBool, Ordering}};
use num_traits::PrimInt;
use rayon::prelude::*;
use affinity::set_thread_affinity;

pub trait ParallelFor<T, F> {
    fn par_for(&self, f: F, granularity: Option<usize>) where
        T: PrimInt,
        F: Fn(T) + Send + Sync;
}

impl<F> ParallelFor<usize, F> for Range<usize> {
    #[inline(always)]
    fn par_for(&self, f: F, granularity: Option<usize>) where
        F: Fn(usize) + Send + Sync
    {
        let (start, end) = (self.start, self.end);
        if let Some(g) = granularity {
            (start..end).into_par_iter().with_min_len(g).with_max_len(g).for_each(|i| { f(i); });
            // (start..end).into_par_iter().step_by(g).with_min_len(1).with_max_len(1).for_each(|s| {
            //     let e = end.min(s + g);
            //     // for j in s..e { f(j); }
            //     let mut j = s;
            //     while j < e { f(j); j+=1; }
            // });
            // (start..end).into_par_iter().with_min_len(g).for_each(f);
        } else {
            (start..end).into_par_iter().for_each(f);
        }
    }
}

pub trait RangeIndirectFor<T, S, F> {
    fn range_ind_for(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut [T], &[S]) + Send + Sync;
}

pub trait SingleIndirectFor<T, S, F> {
    fn sng_val_ind_for(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut T) + Send + Sync;

    fn sng_val_ind_for_seq(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut T) + Send + Sync;
}

type RTy<'a, T, S> = (usize, &'a mut [T], &'a [S], S);
fn spliter<T, S>(r: RTy<T, S>) -> (RTy<T, S>, Option<RTy<T, S>>) where
    T: Send + Sync,
    S: PrimInt + Send + Sync,
{
    let (i, _, idxs, offset) = r;
    if idxs.len() == 1 {
        return (r, None);
    } else {
        let mi = idxs.len()/2;
        let m = idxs[mi] - offset;
        let (l, r) = r.1.split_at_mut(m.to_usize().unwrap());
        return ((i, l, &idxs[..mi], offset), Some((i+mi, r, &idxs[mi..], offset+m)));
    }
}

impl<T, S, F> RangeIndirectFor<T, S, F> for [T]
{
    #[inline(always)]
    fn range_ind_for(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut [T], &[S]) + Send + Sync,
    {
        rayon::iter::split((0, self, offsets, S::zero()), spliter)
            .for_each(|(i, dest, idxs, _)| { f(i, dest, idxs); });
    }
}

impl<T, S, F> SingleIndirectFor<T, S, F> for [T]
{
    fn sng_val_ind_for(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut T) + Send + Sync,
    {
        // Check offsets are unique
        let bool_table: Vec<AtomicBool> = (0..self.len()).into_par_iter().map(|_| AtomicBool::new(false)).collect();

        offsets.into_par_iter().for_each( |x| {
            bool_table[x.to_usize().unwrap()].compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).unwrap();
        });

        // Do loop
        let ptr = self.as_mut_ptr() as usize;
        offsets.into_par_iter().enumerate().for_each(|(i, oi)| {
            let ai = unsafe {(ptr as *mut T).add(oi.to_usize().unwrap()).as_mut().unwrap()};
            f(i, ai);
        });
    }

    fn sng_val_ind_for_seq(&mut self, offsets: &[S], f: F) where
        T: Send + Sync,
        S: PrimInt + Send + Sync,
        F: Fn(usize, &mut T) + Send + Sync,
    {
        // Check offsets are unique
        let mut bool_table: Vec<bool> = (0..self.len()).into_par_iter().map(|_| false).collect();

        for &x in offsets{
            if bool_table[x.to_usize().unwrap()] { panic!(); }
            else { bool_table[x.to_usize().unwrap()] = true; }
        };

        // Do loop
        let ptr = self.as_mut_ptr() as usize;
        offsets.into_par_iter().enumerate().for_each(|(i, oi)| {
            let ai = unsafe {(ptr as *mut T).add(oi.to_usize().unwrap()).as_mut().unwrap()};
            f(i, ai);
        });
    }
}


pub fn config_rayon() {
    // TODO: find a better way to do this.
    (0..rayon::current_num_threads()).par_bridge().for_each(|_| {
        set_thread_affinity([rayon::current_thread_index().unwrap()]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100))
    })
}
