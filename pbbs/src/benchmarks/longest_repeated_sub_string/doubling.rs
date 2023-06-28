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

use crate::DefChar;

#[cfg(not(any(feature = "AW_safe", feature = "sng_ind_atomic")))]
use crate::{DefInt, lcp::lcp, suffix_array::suffix_array};
#[cfg(any(feature = "AW_safe", feature = "sng_ind_atomic"))]
use crate::{ORDER, DefAtomInt, lcp::atomic_lcp, suffix_array::atomic_suffix_array};

type Result = (usize, usize, usize);

#[cfg(not(any(feature = "AW_safe", feature = "sng_ind_atomic")))]
pub fn lrs(s: &[DefChar]) -> Result {
    let mut t = parlay::Timer::new("lrs"); //t.start();

    let mut sa: Vec<DefInt> = parlay::maybe_uninit_vec![
        DefInt::default(); s.len()];
    suffix_array(s, &mut sa);
    t.next("suffix array");

    let lcps = lcp(s, &sa);
    t.next("lcps");

    let idx = (&lcps).into_par_iter().enumerate().reduce(
        || (0, &0),
        |a, b| if a.1 < b.1 { b } else { a }
    ).0;
    t.next("max element");

    (lcps[idx] as usize, sa[idx] as usize, sa[idx+1] as usize)
}

#[cfg(any(feature = "AW_safe", feature = "sng_ind_atomic"))]
pub fn lrs(s: &[DefChar]) -> Result {
    let mut t = parlay::Timer::new("lrs"); //t.start();

    let mut sa: Vec<_> = (0..s.len())
        .into_par_iter()
        .map(|_| DefAtomInt::default())
        .collect();
    atomic_suffix_array(s, &mut sa);
    t.next("suffix array");

    let lcps = atomic_lcp(s, &sa);
    t.next("lcps");

    let dummy = DefAtomInt::default();
    let idx = (&lcps).into_par_iter().enumerate().reduce(
        || (0, &dummy),
        |a, b| if a.1.load(ORDER) < b.1.load(ORDER) { b } else { a }
    ).0;
    t.next("max element");

    (
        lcps[idx].load(ORDER) as usize,
        sa[idx].load(crate::ORDER) as usize,
        sa[idx+1].load(crate::ORDER) as usize
    )
}
