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

use std::mem::swap;
use rayon::prelude::*;

use crate::{DefIntS, DefInt};
use crate::graph::EdgeArray;

#[path="../../common/spec_for.rs"] mod spec_for;
use spec_for::{Reservation, StatefulSpecFor};

#[cfg(not(feature = "AW_safe"))]
use crate::union_find::UnionFind;
#[cfg(feature = "AW_safe")]
use crate::union_find::AtomicUnionFind;


#[derive(Clone)]
struct SFState {
    u: DefIntS,
    v: DefIntS,
}

pub fn spanning_forest(ea: &EdgeArray) -> Vec<u32> {
    let m = ea.non_zeros;
    let n = ea.num_rows;

    #[cfg(not(feature = "AW_safe"))]
    let uf = UnionFind::new(n);
    #[cfg(not(feature = "AW_safe"))]
    let uf_ptr = &uf as *const UnionFind as usize;

    #[cfg(feature = "AW_safe")]
    let uf = AtomicUnionFind::new(n);

    let rs: Vec<Reservation> = (0..n)
        .into_par_iter()
        .map(|_| Reservation::new())
        .collect();

    let reserve = |i: usize, s: &mut SFState| -> bool {
        let e = &ea[i];

        #[cfg(feature = "AW_safe")]
        {
            s.u = uf.find(e.u as i32);
            s.v = uf.find(e.v as i32);
        }
        #[cfg(not(feature = "AW_safe"))]
        unsafe {
            s.u = (uf_ptr as *mut UnionFind).as_mut().unwrap().find(e.u as i32);
            s.v = (uf_ptr as *mut UnionFind).as_mut().unwrap().find(e.v as i32);
        }

        if s.u > s.v { swap(&mut s.u, &mut s.v); }

        if s.u != s.v {
            rs[s.v as usize].reserve(i as DefInt);
            true
        } else { false }
    };

    let commit = |i: usize, s: &mut SFState| -> bool {
        if rs[s.v as usize].check(i as DefInt) {
            #[cfg(feature = "AW_safe")] { uf.link(s.v, s.u); }
            #[cfg(not(feature = "AW_safe"))]
            unsafe {
                (uf_ptr as *mut UnionFind).as_mut().unwrap().link(s.v, s.u);
            }
            true
        } else { false }
    };

    let (rcs, ccs) = (Some(1024), Some(4096));
    (0..m).stateful_spec_for(
        reserve,
        commit,
        SFState { u: -1, v: -1 },
        100,
        rcs,
        ccs
    ).expect("failed speculative for");

    rs
        .into_par_iter()
        .filter_map(|r| if r.reserved() {Some(r.get())} else {None})
        .collect()
}
