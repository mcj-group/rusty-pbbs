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

#[cfg(feature = "AW_safe")]
use std::sync::atomic::AtomicBool;
use rayon::prelude::*;
use enhanced_rayon::prelude::*;

use crate::{DefChar, DefInt};
#[cfg(any(feature = "sng_ind_atomic", feature = "AW_safe"))]
use crate::{DefAtomInt, ORDER};
use parlay::{Timer, maybe_uninit_vec};
use parlay::random::Random;
use parlay::primitives::{pack_index, flatten};
use parlay::internal::counting_sort::count_sort;

fn bw_decode_(s: &[DefChar]) -> Vec<DefChar> {
    let mut t = Timer::new("bw");
    // t.start();
    let n = s.len();

    #[derive(Clone, Copy)]
    struct Link {next: DefInt, c: DefChar}

    // sort character, returning original locations in sorted order
    let lnks: Vec<Link> = (0..n)
        .into_par_iter()
        .map(|i| Link {next: i as DefInt, c: s[i]})
        .collect();
    t.next("making lnks");

    let mut links: Vec<Link> = maybe_uninit_vec![Link{next: 0, c: 0}; n];
    count_sort(&lnks, &mut links, s, 256, 1.0);
    t.next("count sort");

    // break lists into blocks
    let block_size = 5000;

    // pick a set of about n/block_size locations as heads
    // head_flags are set to true for heads
    // links that point to a head are set to their original position + n
    // the overall first character is made to be a head
    let r = Random::new(0);
    #[allow(unused_mut)]
    let mut head_flags: Vec<_>;
    let start = links[0].next as usize;
    links[0].next += n as DefInt;
    #[cfg(feature = "AW_safe")]
    {
        // make arrays atomics
        let atom_head_flags: Vec<_> = (0..n)
            .into_par_iter()
            .map(|_| AtomicBool::new(false))
            .collect();
        atom_head_flags[start].store(true, ORDER);
        let atom_link_n: Vec<_> = (&links)
            .into_par_iter()
            .map(|l| DefAtomInt::new(l.next))
            .collect();

        (0..(n/block_size+2))
            .into_par_iter()
            .for_each(|i| {
                let j = r.ith_rand(i as u64) as usize % n;
                let lnk = atom_link_n[j].load(ORDER);
                if (lnk as usize) < n {
                    atom_head_flags[lnk as usize].store(true, ORDER);
                    atom_link_n[j].store(lnk + n as DefInt, ORDER);
                }
            });

        // retrive arrays
        (&atom_link_n, &mut links)
            .into_par_iter()
            .for_each(|(n, l)| l.next = n.load(ORDER));
        head_flags = atom_head_flags
            .into_par_iter()
            .map(|ahf| ahf.into_inner())
            .collect();
    }
    #[cfg(not(feature = "AW_safe"))]
    {
        head_flags = (0..n).into_par_iter().map(|_| false).collect();
        head_flags[start] = true;
        (0..(n/block_size+2)).into_par_iter().for_each(|i| {
            let j = r.ith_rand(i as u64) as usize % n;
            let lnk = links[j].next;
            unsafe { if (lnk as usize) < n {
                (head_flags.as_ptr() as *mut bool).add(lnk as usize).write(true);
                (links.as_ptr() as *mut Link).add(j).as_mut().unwrap().next
                    = lnk + n as DefInt;
            }}
        });
    }
    t.next("set next");

    // indices of heads;
    let mut heads: Vec<DefInt> = vec![];
    pack_index(&head_flags, &mut heads);
    t.next("pack index");

    // map over each head and follow the list until reaching the next head
    // as following the list, add characters to a buffer
    // at the end return the buffer trimmed to fit the substring exactly
    // also return pointer to the next head
    let blocks: Vec<_> = heads.par_iter().map(|&cur_head| {
        let buffer_len = block_size * 30;
        let mut buffer = maybe_uninit_vec![DefChar::default(); buffer_len];
        let mut i = 0;
        let mut pos = cur_head as usize;
        loop {
            let ln = links[pos];
            buffer[i] = ln.c;
            i += 1;
            if i == buffer_len {
                panic!("ran out of buffer space in bw decode");
            }
            pos = ln.next as usize;

            if pos >= n { break; }
        }
        let trimmed: Vec<DefChar> = (0..i).map(|j| buffer[j]).collect();
        (trimmed, pos % n)
    }).collect();
    t.next("follow pointers");

    // location in heads for each head in s
    #[cfg(feature = "sng_ind_atomic")]
    let mut location_in_heads: Vec<DefAtomInt>
        = maybe_uninit_vec![DefAtomInt::default(); n];
    #[cfg(not(feature = "sng_ind_atomic"))]
    let mut location_in_heads: Vec<DefInt>
        = maybe_uninit_vec![DefInt::default(); n];

    let mut pos = start;
    let mut ordered_blocks: Vec<&Vec<u8>>;
    let _dummy: Vec<u8> = vec![];
    #[cfg(feature = "sng_ind_atomic")]
    {
        let m = heads.len();
        (0..m).into_par_iter().for_each(|i| {
            location_in_heads[heads[i] as usize].store(i as DefInt, ORDER);
        });
        t.next("link heads");

        // start at first block and follow next pointers
        // putting each into ordered_blocks
        ordered_blocks = maybe_uninit_vec![&_dummy; m];
        for i in 0..m {
            let j = location_in_heads[pos].load(ORDER) as usize;
            (ordered_blocks[i], pos) = (&blocks[j].0, blocks[j].1);
        }
        t.next("order heads");
    }
    #[cfg(not(feature = "sng_ind_atomic"))]
    {
        let m = heads.len();
        location_in_heads
            .par_ind_iter_mut(&heads)
            .enumerate()
            .for_each(|(i, loc)| *loc = i as DefInt);
        t.next("link heads");

        // start at first block and follow next pointers
        // putting each into ordered_blocks
        ordered_blocks = maybe_uninit_vec![&_dummy; m];
        for i in 0..m {
            let j = location_in_heads[pos] as usize;
            (ordered_blocks[i], pos) = (&blocks[j].0, blocks[j].1);
        }
        t.next("order heads");
    }

    // flatten ordered blocks into final string
    let mut res = vec![];
    flatten(&ordered_blocks, &mut res);
    t.next("flatten");

    // drop the first character, which is a null character
    res.pop().unwrap();
    res
}

pub fn bw_decode(s: &[DefChar]) -> Vec<DefChar> {
    if s.len() >= 1 << 32 {
        panic!("current implementation can't handle an input this big.");
    }
    bw_decode_(s)
}
