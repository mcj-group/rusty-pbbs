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

use std::time::Duration;

#[path ="mod.rs"] mod hist;
#[path ="../../misc.rs"] mod misc;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;

use misc::*;
use hist::{sequential, parallel};
use io::{read_big_file_to_vec, write_slice_to_file_seq};

define_args!(
    Algs::PARALLEL,
    (buckets, usize, 0)
);

define_algs!(
    (PARALLEL, "parallel"),
    (SEQUENTIAL, "sequential")
);

pub fn run(alg: Algs, rounds: usize, buckets: usize, arr: &[u32]) -> (Vec<u32>, Duration) {
    let f = match alg {
        Algs::PARALLEL => {parallel::hist},
        Algs::SEQUENTIAL => {sequential::hist}
    };

    let mut r = vec![];
    let r_ptr = &r as *const Vec<u32> as usize;

    let mean = time_loop(
        "hist",
        rounds,
        Duration::new(1, 0),
        || { unsafe { *(r_ptr as *mut Vec<u32>) = vec![]; } },
        || { f(&arr, buckets, &mut r); },
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
    let (r, d) = run(args.algorithm, args.rounds, args.buckets, &arr);

    finalize!(
        args,
        r,
        d,
        write_slice_to_file_seq(&r, args.ofname)
    );
}
