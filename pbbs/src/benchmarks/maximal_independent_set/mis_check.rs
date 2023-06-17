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

use clap::Parser;
use rayon::prelude::*;

use pbbs::common::graph::Graph;
use pbbs::common::io::read_file_to_vec_seq;
use pbbs::common::graph_io::read_graph_from_file;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// mis results filename
    #[clap(value_parser, required=true)]
    rfname: String,

    /// the input graph's filename
    #[clap(value_parser, required=true)]
    ifname: String,
}

pub fn check(g: Graph, selected: &[bool]) -> bool {
    let mut violation_no = 0usize;
    let mut self_or_ngh_selected: Vec<bool> = selected
        .par_iter()
        .map(|q| *q)
        .collect();
    for a in 0..g.n {
        for &b in g.index(a).neighbors {
            let b = b as usize;
            if a < b {
                if selected[a] || selected[b] {
                    self_or_ngh_selected[a] = true;
                    self_or_ngh_selected[b] = true;
                    if selected[a] && selected[b] {
                        violation_no += 1;
                    }
                }

            }
        }
    }
    if violation_no != 0 || self_or_ngh_selected.contains(&false) {
        println!("violations_no:{} missed_nodes:{} selected:{}",
            violation_no,
            self_or_ngh_selected.iter().filter(|q| **q == false).count(),
            selected.iter().filter(|q| **q == true).count()
        );
        false
    } else { true }
}

fn main() {
    let args = Args::parse();
    let g = read_graph_from_file(&args.ifname);
    let r: Vec<bool> = read_file_to_vec_seq(&args.rfname)
        .iter()
        .map(|q: &u8| *q == 1)
        .collect();
    if check(g, &r) { println!("OK"); }
    else { println!("ERR"); std::process::exit(1); }
}
