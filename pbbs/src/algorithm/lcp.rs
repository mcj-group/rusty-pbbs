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

use parlay::{Timer, maybe_uninit_vec};
use parlay::primitives::pack_index;
use crate::{DefChar, DefInt, DefAtomInt, ORDER};
use crate::range_min::{RangeMin, AtomU32RangeMin};

#[allow(dead_code)]
pub fn lcp(s: &[DefChar], sa: &[DefInt]) -> Vec<DefInt> {
    let mut t = Timer::new("lcp"); //t.start();
    let mut len = 111;
    let n = sa.len();
    t.next("init");

    // compare first len characters of adjacent strings from SA.
    #[allow(unused_mut)]
    let mut l: Vec<_> = (0..n-1).into_par_iter().map(|i| {
        let mut j = 0;
        let max_j = len.min(n-sa[i] as usize);
        while j < max_j &&
            {
                #[cfg(feature = "mem_safe")]
                { s[sa[i] as usize + j] == s[sa[i+1] as usize + j] }
                #[cfg(not(feature = "mem_safe"))]
                unsafe {
                    s.as_ptr().add(sa[i] as usize + j).read() ==
                    s.as_ptr().add(sa[i+1] as usize + j).read()
                }
            }
        { j+=1; }
        (if j < len { j } else { n }) as DefInt
    }).collect();

    t.next("head");

    // keep indices for which we do not yet know their LCP (i.e. LCP >= len)
    let mut remain = Vec::<u32>::new();
    pack_index(
        &l.par_iter().map(|&li| li as usize == n).collect::<Vec<bool>>(),
        &mut remain
    );
    t.next("pack");

    if remain.len() == 0 { return l; }

    // an inverse permutation for SA
    #[allow(unused_mut)]
    let mut isa: Vec<DefInt> = maybe_uninit_vec![DefInt::default(); n];

    isa
        .par_ind_iter_mut(sa)
        .enumerate()
        .for_each(|(i, isa_i)| *isa_i = i as DefInt);
    t.next("inverse");

    // repeatedly double len determining LCP by joining next len chars
    // invariant: before i^th round L contains correct LCPs less than len
    //            and n for the rest of them
    //            remain holds indices of the rest of them (i.e., LCP[i] >= len)
    //      after round, len = 2*len and invariant holds for the new len
    loop {
        let rq = RangeMin::new(&l, |a, b| a < b, 111);
        t.next("make range");

        remain = remain.into_par_iter().filter(|&i| {
            let i = i as usize;
            if sa[i] as usize + len >= n {
                unsafe {(l.as_ptr() as *mut DefInt).add(i).write(len as DefInt);}
                return false;
            }
            let i1 = isa[len + sa[i] as usize];
            let i2 = isa[len + sa[i+1] as usize];
            let li = l[rq.query(i1, i2-1) as usize];
            if (li as usize) < len {
                unsafe {(l.as_ptr() as *mut DefInt).add(i).write(len as DefInt + li);}
                return false;
            }
            else { return true; }
        }).collect();
        t.next("filter");

        len *= 2;
        if remain.len() <= 0 { break; }
    }
    return l;
}


#[allow(dead_code)]
pub fn atomic_lcp(s: &[DefChar], sa: &[DefAtomInt]) -> Vec<DefAtomInt> {
    let mut t = Timer::new("lcp"); //t.start();
    let mut len = 111;
    let n = sa.len();
    t.next("init");

    // compare first len characters of adjacent strings from SA.
    let l: Vec<_> = (0..n-1).into_par_iter().map(|i| {
        let mut j = 0;
        let max_j = len.min(n-sa[i].load(ORDER) as usize);
        while j < max_j &&
            {
                #[cfg(feature = "mem_safe")]
                {
                    s[sa[i].load(ORDER) as usize + j]
                        == s[sa[i+1].load(ORDER) as usize + j]
                }
                #[cfg(not(feature = "mem_safe"))]
                unsafe {
                    s.as_ptr().add(sa[i].load(ORDER) as usize + j).read()
                        == s.as_ptr().add(sa[i+1].load(ORDER) as usize + j).read()
                }
            }
        { j+=1; }
        DefAtomInt::new((if j < len { j } else { n }) as DefInt)
    }).collect();

    t.next("head");

    // keep indices for which we do not yet know their LCP (i.e. LCP >= len)
    let mut remain = Vec::<u32>::new();
    pack_index(
        &l.par_iter().map(|li| li.load(ORDER) as usize == n).collect::<Vec<bool>>(),
        &mut remain
    );
    t.next("pack");

    if remain.len() == 0 { return l; }

    // an inverse permutation for SA
    let isa: Vec<DefAtomInt> = (0..n)
        .into_par_iter()
        .map(|i| DefAtomInt::new(i as DefInt))
        .collect();

    (0..n).into_par_iter().for_each(|i|
        isa[sa[i].load(ORDER) as usize].store(i as DefInt, ORDER)
    );
    t.next("inverse");

    // repeatedly double len determining LCP by joining next len chars
    // invariant: before i^th round L contains correct LCPs less than len
    //            and n for the rest of them
    //            remain holds indices of the rest of them (i.e., LCP[i] >= len)
    //      after round, len = 2*len and invariant holds for the new len
    loop {
        let rq = AtomU32RangeMin::new(&l, |a, b| a < b, 111);
        t.next("make range");

        remain = remain.into_par_iter().filter(|&i| {
            let i = i as usize;
            if sa[i].load(ORDER) as usize + len >= n {
                l[i].store(len as DefInt, ORDER);
                return false;
            }
            let i1 = isa[len + sa[i].load(ORDER) as usize].load(ORDER);
            let i2 = isa[len + sa[i+1].load(ORDER) as usize].load(ORDER);
            let li = l[rq.query(i1, i2-1) as usize].load(ORDER);
            if (li as usize) < len {
                l[i].store(len as DefInt + li, ORDER);
                return false;
            }
            else { return true; }
        }).collect();
        t.next("filter");

        len *= 2;
        if remain.len() <= 0 { break; }
    }
    return l;
}
