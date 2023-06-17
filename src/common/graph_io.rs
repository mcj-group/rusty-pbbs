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

use std::cmp::max;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use rayon::prelude::*;

use parlay::verbose_println;
use crate::DefInt;
use super::io::{read_file_to_vec, read_big_file_to_vec};
use super::graph::*;

const ADJ_GRAPH_HEADER: &str = "AdjacencyGraph";

pub fn read_graph_from_file(fname: &str) -> Graph {
    let file = File::open(&fname).unwrap();
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    verbose_println!("reading file header...");
    assert_eq!(lines.next().unwrap().unwrap(), ADJ_GRAPH_HEADER);
    let n = lines.next().unwrap().unwrap().parse().unwrap();
    let m = lines.next().unwrap().unwrap().parse().unwrap();

    verbose_println!("making the graph (n={n}, m={m})...");
    let mut g = Graph {
        offsets: Vec::with_capacity(n+1),
        edges: Vec::with_capacity(m),
        degrees: vec![],
        n,
        m,
    };
    unsafe { g.offsets.set_len(n+1); g.edges.set_len(m); }

    verbose_println!("reading offsets...");
    for i in 0..n {
        let tt = lines.next().unwrap().unwrap();
        let ttt = tt.parse();
        if ttt.is_err() { println!("{i}, {tt}") }
        g.offsets[i] = ttt.unwrap();
    }
    g.offsets[n] = m as DefInt;

    verbose_println!("reading edges...");
    for i in 0..m {
        g.edges[i] = lines.next().unwrap().unwrap().parse().unwrap();
    }

    verbose_println!("graph generated.");
    g
}

pub fn read_edge_array_from_file(fname: &str) -> EdgeArray {
    let mut ea = EdgeArray {
        es: vec![],
        num_rows: 0,
        num_cols: 0,
        non_zeros: 0,
    };

    verbose_println!("reading file...");
    read_big_file_to_vec(
        fname,
        Some { 0: |w: &[&str]| {debug_assert_eq!(w[0], "EdgeArray")} },
        &mut ea.es
    );
    let n = ea.es.len();

    verbose_println!("finding_max...");
    let max = (&ea.es)
        .into_par_iter()
        .cloned()
        .reduce(
            || Edge::new(0, 0),
            |a, b| Edge::new(a.u.max(b.u), a.v.max(b.v)
        ));
    let rm = max.u.max(max.v) as usize + 1;

    ea.non_zeros = n;
    ea.num_rows = rm;
    ea.num_cols = rm;

    verbose_println!("done.");
    ea
}

pub fn read_wgh_edge_array_from_file(fname: &str) -> WghEdgeArray {
    let es: Vec<WghEdge> = read_file_to_vec(
        fname,
        Some { 0: |w: &[&str]| {
            debug_assert_eq!(w[0], "WeightedEdgeArray")
        }} );

    let m = es
        .par_iter()
        .cloned()
        .reduce(
            || WghEdge::new(0, 0, 0.0),
            |a, b| WghEdge::new(max(a.u, b.u), max(a.v, b.v), 0.0)
        );

    println!("extracted graph n={} m={}", max(m.u, m.v) as usize + 1, es.len());

    WghEdgeArray::new(es, max(m.u, m.v) as usize + 1)
}
