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

#![allow(dead_code)]

#[path ="mod.rs"] mod isort;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;
#[path ="../../common/time_loop.rs"] mod time_loop;

use std::time::Duration;
use io::{read_big_file_to_vec, write_slice_to_file_seq};

define_args!(
    Algs::PARRADIX,
    (bits, usize, 0)
);

define_algs!(
    (PARRADIX, "parradix")
);

pub fn run(
    alg: Algs,
    rounds: usize,
    g: &[u32],
    bits: usize
) -> (Vec<u32>, Duration)
{
    let f = match alg {
        Algs::PARRADIX => isort::parallel_radix_sort::int_sort,
    };

    let mut r = parlay::maybe_uninit_vec![];
    let r_ptr = &r as *const Vec<u32> as usize;

    let mean = time_loop(
        "isort",
        rounds,
        Duration::new(1, 0),
        || { unsafe { *(r_ptr as *mut Vec<u32>).as_mut().unwrap() = vec![];}},
        || { f(&g, bits, &mut r); },
        || {}
    );

    (r, mean)
}

fn main() {
    init!();

    let args = Args::parse();
    let mut arr = Vec::new();
    read_big_file_to_vec(
        &args.ifname,
        Some { 0: |w: &[&str]| {debug_assert_eq!(w[0], "sequenceInt")} },
        &mut arr
    );
    let (r, d) = run(args.algorithm, args.rounds, &arr, args.bits);

    finalize!(
        args,
        r,
        d,
        write_slice_to_file_seq(&r, args.ofname)
    );
}
