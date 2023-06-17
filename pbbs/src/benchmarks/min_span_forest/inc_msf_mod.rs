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
use std::sync::atomic::{AtomicBool};

use parlay::primitives::pack_index;
use parlay::internal::sample_sort_inplace;
use crate::{DefInt, DefIntS};
use crate::graph::WghEdgeArray;
#[cfg(not(feature = "AW_safe"))]
use crate::union_find::UnionFind;
#[cfg(feature = "AW_safe")]
use crate::union_find::AtomicUnionFind;
#[cfg(feature = "AW_safe")]
use crate::ORDER;

#[path="../../common/spec_for.rs"] mod spec_for;
use spec_for::{SpecFor, Reservation};


pub fn minimum_spanning_forest(wea: &WghEdgeArray, dest: &mut Vec<DefInt>) {
    #[cfg(feature = "AW_safe")]
    eprintln!("WARNING: AW_safe is enabled, \
        but this algorithm has an array that requires synchronization.");

    let mut t = parlay::Timer::new("msf"); t.start();

    let m = wea.m;
    let n = wea.n;
    let mut idxs: Vec<_> = (0..m)
        .into_par_iter()
        .map(|i| i as DefInt)
        .collect();
    t.next("Creating indices");

    let cmp = |i: DefInt, j: DefInt| {
        let (i, j) = (i as usize, j as usize);
        if wea[i].w == wea[j].w { i < j }
        else { wea[i].w < wea[j].w }
    };
    sample_sort_inplace(&mut idxs, cmp, false);
    t.next("Sorting");

    #[cfg(not(feature = "AW_safe"))]
    let uf = UnionFind::new(n);
    #[cfg(feature = "AW_safe")]
    let uf = AtomicUnionFind::new(n);
    let _uf_ptr = &uf as *const _ as usize;

    #[cfg(not(feature = "AW_safe"))]
    let msf_flags: Vec<_> = (0..m).into_par_iter().map(|_| false).collect();
    #[cfg(feature = "AW_safe")]
    let msf_flags: Vec<_> = (0..m).map(|_| AtomicBool::new(false)).collect();
    let _msf_flags_ptr = msf_flags.as_ptr() as usize;

    let rs: Vec<Reservation> = (0..n).map(|_| Reservation::new()).collect();

    let wea_ptr = wea as *const WghEdgeArray as usize;

    let reserve = |i: usize| {
        let wea = unsafe { (wea_ptr as *mut WghEdgeArray).as_mut().unwrap() };
        let e = wea.get_mut(idxs[i] as usize);
        #[cfg(not(feature = "AW_safe"))]
        let luf = unsafe { (_uf_ptr as *mut UnionFind).as_mut().unwrap() };
        #[cfg(feature = "AW_safe")]
        let luf = &uf;

        e.u = luf.find(e.u as DefIntS) as DefInt;
        e.v = luf.find(e.v as DefIntS) as DefInt;
        if e.u != e.v {
            rs[e.v as usize].reserve(idxs[i]);
            rs[e.u as usize].reserve(idxs[i]);
            true
        } else { false }
    };

    #[cfg(not(feature = "AW_safe"))]
    let commit = |i: usize| {
        let luf = unsafe { (_uf_ptr as *mut UnionFind).as_mut().unwrap() };
        let e = wea[idxs[i] as usize];
        let (u, v) = (e.u, e.v);
        if rs[v as usize].check(idxs[i]) {
            rs[u as usize].check_reset(idxs[i]);
            luf.link(v as DefIntS, u as DefIntS);
            unsafe { (_msf_flags_ptr as *mut bool)
                .add(idxs[i] as usize).write(true); }
            true
        } else if rs[u as usize].check(idxs[i]) {
            luf.link(u as DefIntS, v as DefIntS);
            unsafe { (_msf_flags_ptr as *mut bool)
                .add(idxs[i] as usize).write(true);}
            true
        } else {
            false
        }
    };

    #[cfg(feature = "AW_safe")]
    let commit = |i: usize| {
        let luf = &uf;
        let e = wea[idxs[i] as usize];
        let (u, v) = (e.u, e.v);
        if rs[v as usize].check(idxs[i]) {
            rs[u as usize].check_reset(idxs[i]);
            luf.link(v as DefIntS, u as DefIntS);
            msf_flags[idxs[i] as usize].store(true, ORDER);
            true
        } else if rs[u as usize].check(idxs[i]) {
            luf.link(u as DefIntS, v as DefIntS);
            msf_flags[idxs[i] as usize].store(true, ORDER);
            true
        } else {
            false
        }
    };

    let (rcs, ccs) = (Some(1024), Some(2048));
    t.next("Initializations");
    (0..m)
        .spec_for(reserve, commit, 20, rcs, ccs)
        .expect("failed speculative for");
    t.next("Specualtive For");

    #[cfg(feature = "AW_safe")]
    let msf_flags: Vec<_> = msf_flags
        .into_par_iter()
        .map(|f| f.into_inner())
        .collect();
    pack_index(&msf_flags, dest);
    t.next("Packing");
}
