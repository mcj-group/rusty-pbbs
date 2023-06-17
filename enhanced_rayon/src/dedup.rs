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

use std::sync::atomic::{AtomicBool, Ordering};
use num_traits::PrimInt;
use rayon::prelude::*;

#[allow(dead_code)]
pub(super) fn serial<T: PrimInt>(offsets: &[T], len: usize) {
    let mut table: Vec<bool> = (0..len)
        .into_par_iter()
        .map(|_| false)
        .collect();

    offsets.into_iter().for_each( |x| {
        if table[x.to_usize().unwrap()] { panic!("Duplicate offset"); }
        else { table[x.to_usize().unwrap()] = true; }
    });
}

#[allow(dead_code)]
pub(super) fn parallel<T: PrimInt + Sync>(offsets: &[T], len: usize) {
    let table: Vec<AtomicBool> = (0..len)
        .into_par_iter()
        .map(|_| AtomicBool::new(false))
        .collect();

    offsets.into_par_iter().for_each( |x| {
        table[x.to_usize().unwrap()].compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).unwrap();
    });
}

#[allow(dead_code)]
pub(super) fn parallel_by<F>(offset: F, off_len: usize, len: usize)
where
    F: Fn(usize) -> usize + Sync + Clone
{
    let table: Vec<AtomicBool> = (0..len)
        .into_par_iter()
        .map(|_| AtomicBool::new(false))
        .collect();

    (0..off_len).into_par_iter().for_each( |i| {
        table[(offset)(i)].compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).unwrap();
    });
}
