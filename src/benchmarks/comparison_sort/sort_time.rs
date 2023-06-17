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

#[path ="mod.rs"] mod comparison_sort;
#[path ="../../common/io.rs"] mod io;
#[path ="../../common/time_loop.rs"] mod time_loop;
#[path ="../macros.rs"] mod macros;

use std::time::Duration;
use io::{read_file_to_vec, write_slice_to_file_seq};


define_args!(
    Algs::MERGE,
    (stable, bool, false)
);

define_algs!(
    (STD,       "std"),
    (RAYON,     "rayon"),
    (MERGE,     "merge"),
    (QUICK,     "quick"),
    (BUCKET,    "bucket"),
    (SAMPLE,    "sample")
);


pub fn run<T, F>(
    alg: Algs,
    rounds: usize, 
    stable: bool,
    less: F,
    inp: &[T]
) -> (Vec<T>, Duration) where
    T: Copy + Send + Sync + Default,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    let f = match alg {
        Algs::MERGE     => comparison_sort::merge_sort::comp_sort,
        Algs::QUICK     => comparison_sort::quick_sort::comp_sort,
        Algs::BUCKET    => comparison_sort::bucket_sort::comp_sort,
        Algs::SAMPLE    => comparison_sort::sample_sort::comp_sort,
        Algs::STD       => comparison_sort::std::comp_sort,
        Algs::RAYON     => comparison_sort::rayon::comp_sort,
    };

    let mut r = parlay::maybe_uninit_vec![T::default(); inp.len()];
    let r_clone = unsafe { (&mut r[..] as *mut [T]).as_mut().unwrap() };

    let mean = time_loop(
        "sort",
        rounds,
        Duration::new(1, 0),
        || { r_clone.copy_from_slice(inp); },
        || { f(&mut r, less, stable) },
        || {}
    );
    
    (r, mean)
}

fn main() {
    init!();

    let args = Args::parse();

    let arr: Vec<i32> = read_file_to_vec(
        &args.ifname,
        Some { 0: |w: &[&str]| {debug_assert_eq!(w[0], "sequenceInt")} }
    );

    let less = |a: i32, b: i32| a < b;

    let (r, d) = run(
        args.algorithm,
        args.rounds,
        args.stable,
        less,
        &arr
    );

    finalize!(
        args,
        r,
        d,
        write_slice_to_file_seq(&r, args.ofname)
    );
}
