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
#[cfg(feature = "AW_safe")]
use std::sync::atomic::AtomicBool;

use crate::DefInt;
use crate::{graph::EdgeArray};
#[cfg(feature = "AW_safe")]
use crate::ORDER;

#[path="../../common/spec_for.rs"] mod spec_for;

use spec_for::{SpecFor, Reservation};


pub fn maximal_matching(ea: &EdgeArray) -> Vec<DefInt> {
    let n = std::cmp::max(ea.num_rows, ea.num_cols);
    let m = ea.non_zeros;
    let rs: Vec<Reservation> = (0..n)
        .into_par_iter()
        .map(|_| Reservation::new())
        .collect();

    #[cfg(not(feature = "AW_safe"))]
    let matched: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|_| false)
        .collect();
    #[cfg(feature = "AW_safe")]
    let matched: Vec<AtomicBool> = (0..n)
        .into_par_iter()
        .map(|_| AtomicBool::new(false))
        .collect();

    #[cfg(not(feature = "AW_safe"))]
    let matched_ptr = matched.as_ptr() as usize;

    let reserve = |i: usize| -> bool {
        let (u, v) = (ea[i].u as usize, ea[i].v as usize);
        let i = i as u32;
        #[cfg(not(feature = "AW_safe"))]
        if matched[u] || matched[v] || u == v { false }
        else {
            rs[u].reserve(i);
            rs[v].reserve(i);
            true
        }
        #[cfg(feature = "AW_safe")]
        if matched[u].load(ORDER) || matched[v].load(ORDER) || u == v {
            false
        } else {
            rs[u].reserve(i);
            rs[v].reserve(i);
            true
        }
    };

    let commit = |i: usize| -> bool {
        let (u, v) = (ea[i].u as usize, ea[i].v as usize);
        let i = i as u32;
        if rs[v].check(i) {
            rs[v].reset();
            if rs[u].check(i) {
                #[cfg(not(feature = "AW_safe"))]
                unsafe {
                    (matched_ptr as *mut bool).add(u).write(true);
                    (matched_ptr as *mut bool).add(v).write(true);
                }
                #[cfg(feature = "AW_safe")]
                {
                    matched[u].store(true, ORDER);
                    matched[v].store(true, ORDER);
                }
                return true;
            }
        }
        else if rs[u].check(i) { rs[u].reset(); }
        return false;
    };

    (0..m).spec_for(
        reserve,
        commit,
        10,
        Some(1024),
        Some(2048)
    ).unwrap();

    let mut matching_idx = vec![];
    parlay::primitives::pack(
        &rs
            .par_iter()
            .map(|r| r.get() as DefInt)
            .collect::<Vec<DefInt>>(),
        &rs
            .par_iter()
            .map(|r| r.reserved())
            .collect::<Vec<bool>>(),
        &mut matching_idx
    );

    matching_idx
}
