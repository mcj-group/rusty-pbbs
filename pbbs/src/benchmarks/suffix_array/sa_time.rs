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
use rayon::prelude::*;

#[path ="mod.rs"] mod sa;
#[path ="../../misc.rs"] mod misc;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;
#[path ="../../algorithm/suffix_array.rs"] mod suffix_array;

use misc::*;
use sa::parallel_range;
use io::{chars_from_file, write_slice_to_file_seq};

define_args!(Algs::ParRange);
define_algs!((ParRange, "par-range"));

pub fn run(
    alg: Algs,
    rounds: usize,
    inp: &[DefChar]
) -> (Vec<DefInt>, Duration)
{
    let f = match alg {
        Algs::ParRange => {parallel_range::suffix_array},
    };

    #[cfg(not(feature = "AW_safe"))]
    let mut r: Vec<_> = (0..inp.len())
        .into_par_iter()
        .map(|_| DefInt::default())
        .collect();
    #[cfg(feature = "AW_safe")]
    let mut r: Vec<_> = (0..inp.len())
        .into_par_iter()
        .map(|_| DefAtomInt::default())
        .collect();

    let mean = time_loop(
        "sa",
        rounds,
        Duration::new(1, 0),
        || {},
        || { f(&inp, &mut r); },
        || {}
    );
    #[cfg(feature = "AW_safe")]
    let r: Vec<_> = r.into_par_iter().map(|ri| ri.load(ORDER)).collect();
    (r, mean)
}

fn main() {
    init!();
    let args = Args::parse();
    let arr = chars_from_file(&args.ifname, false).unwrap();
    let (r, d) = run(args.algorithm, args.rounds, &arr);

    finalize!(
        args,
        r,
        d,
        write_slice_to_file_seq(&r, args.ofname)
    );
}
