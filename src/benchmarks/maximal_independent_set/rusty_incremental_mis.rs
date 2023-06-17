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

#[cfg(AW_safe)]
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};
use rayon::prelude::*;

use crate::graph::Graph;
#[path="../../common/spec_for.rs"] mod spec_for;
use spec_for::StatefulSpecFor;


#[derive(Clone)]
struct MISState {
    flag: u8,
}

pub fn maximal_independent_set(g: &Graph) -> Vec<u8> {
    let n = g.n;
    #[cfg(not(feature = "AW_safe"))]
    let flags: Vec<u8> = (0..n)
        .into_par_iter()
        .map(|_| 0)
        .collect();
    #[cfg(feature = "AW_safe")]
    let flags: Vec<AtomicU8> = (0..n)
        .into_par_iter()
        .map(|_| AtomicU8::new(0))
        .collect();
    #[cfg(not(feature = "AW_safe"))]
    let flags_ptr = flags.as_ptr() as usize;

    let reserve = |i: usize, s: &mut MISState| -> bool {
        s.flag = 1;
        let v = g.index(i);
        for j in 0..v.degree {
            let ngh = v.neighbors[j] as usize;
            if ngh < i {
                #[cfg(not(feature = "AW_safe"))]
                let f = flags[ngh];
                #[cfg(feature = "AW_safe")]
                let f = flags[ngh].load(Relaxed);

                if f == 1 { s.flag = 2; return true; }
                else if f == 0 { s.flag = 0; }
            }
        }
        true
    };

    let commit = |i: usize, s: &mut MISState| -> bool {
        #[cfg(not(feature = "AW_safe"))]
        unsafe { (flags_ptr as *mut u8).add(i).write(s.flag); }
        #[cfg(feature = "AW_safe")]
        { flags[i].store(s.flag, Relaxed); }
        s.flag > 0
    };

    (0..n).stateful_spec_for(
        reserve,
        commit,
        MISState { flag: 0 },
        20,
        Some(64),
        Some(256)
    ).expect("failed speculative for");

    #[cfg(not(feature = "AW_safe"))]
    return flags;
    #[cfg(feature = "AW_safe")]
    return flags.into_par_iter().map(|f| f.load(Relaxed)).collect();
}
