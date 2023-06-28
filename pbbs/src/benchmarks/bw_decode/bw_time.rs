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

#[path ="mod.rs"] mod bw;
#[path ="../../misc.rs"] mod misc;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;
#[path ="../../algorithm/bw_encode.rs"] mod bw_encode;

use misc::*;
use bw_encode::bw_encode;
use io::{chars_from_file, chars_to_file};

define_args!(Algs::ListRank);

define_algs!((ListRank, "list-rank"));

pub fn run(alg: Algs, rounds: usize, inp: &[DefChar]) -> (Vec<DefChar>, Duration) {
    let f = match alg {
        Algs::ListRank => {bw::list_rank::bw_decode},
    };

    let mut r = vec![];

    let mean = time_loop(
        "bw",
        rounds,
        Duration::new(1, 0),
        || {},
        || { r = f(&inp); },
        || {}
    );
    (r, mean)
}

fn main() {
    init!();
    let args = Args::parse();
    let arr = chars_from_file(&args.ifname, false).unwrap();

    let encoded = bw_encode(&arr);

    let (r, d) = run(args.algorithm, args.rounds, &encoded);

    finalize!(
        args,
        r,
        d,
        chars_to_file(&r, args.ofname).unwrap()
    );
}
