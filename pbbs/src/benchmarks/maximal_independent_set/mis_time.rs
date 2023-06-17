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

use std::time::Duration;

#[path ="mod.rs"] mod mis;
#[path ="../../misc.rs"] mod misc;
#[path ="../macros.rs"] mod macros;
#[path ="../../common/io.rs"] mod io;
#[path ="../../common/graph.rs"] mod graph;
#[path ="../../common/graph_io.rs"] mod graph_io;

use misc::*;
use mis::{ serial_mis, rusty_incremental_mis};
use graph::Graph;
use io::write_slice_to_file_seq;
use graph_io::read_graph_from_file;

define_args!(Algs::RUSTINC);

define_algs!(
    (SERIAL, "serial"),
    (RUSTINC, "rustinc")
);

pub fn run(alg: Algs, rounds: usize, g: Graph) -> (Vec<u8>, Duration) {
    let mis = match alg {
        Algs::SERIAL => serial_mis::maximal_independent_set,
        Algs::RUSTINC => rusty_incremental_mis::maximal_independent_set,
    };

    let mut r = vec![];

    let mean = time_loop(
        "mis",
        rounds,
        Duration::new(1, 0),
        || {},
        || { r = mis(&g); },
        || {}
    );
    (r, mean)
}

fn main() {
    init!();

    let args = Args::parse();
    let g = read_graph_from_file(&args.ifname);
    let (r, d) = run(args.algorithm, args.rounds, g);

    finalize!(
        args,
        r,
        d,
        write_slice_to_file_seq(&r, args.ofname)
    );
}
