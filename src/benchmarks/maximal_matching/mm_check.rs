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

use pbbs::common::graph::EdgeArray;
use pbbs::common::io::read_file_to_vec_seq;
use pbbs::common::graph_io::read_edge_array_from_file;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// mm results filename
    #[clap(value_parser, required=true)]
    rfname: String,

    /// the input graph's filename
    #[clap(value_parser, required=true)]
    ifname: String,
}

pub fn check(ea: EdgeArray, matching: &[usize]) -> bool {
    let m = ea.non_zeros;
    let n = std::cmp::max(ea.num_rows, ea.num_cols);
    let mut vs = vec![usize::MAX; n];
    let mut flags = vec![false; m];

    matching.iter().for_each(|&i| {
        (vs[ea[i].u as usize], vs[ea[i].v as usize]) = (i, i);
        flags[i] = true;
    });

    for i in 0..m {
        let u = ea[i].u as usize;
        let v = ea[i].v as usize;
        if flags[i] {
            if vs[u] != i {
                println!("mm_check: edges share vertex {u}");
                return false;
            }
            if vs[v] != i {
                println!("mm_check: edges share vertex {v}");
                return false;
            }
        } else {
            if u != v && vs[u] == usize::MAX && vs[v] == usize::MAX {
                println!("mm_check: neither endpoint matched for edge {i}"); return false;
            }
        }
    }
    true
}

fn main() {
    let args = Args::parse();
    let ea = read_edge_array_from_file(&args.ifname);
    let r: Vec<usize> = read_file_to_vec_seq(&args.rfname);
    if check(ea, &r) { println!("OK"); }
    else { println!("ERR"); std::process::exit(1); }
}
