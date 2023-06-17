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

use parlay::primitives::pack;
use pbbs::DefInt;
use pbbs::common::graph::EdgeArray;
use pbbs::common::io::read_file_to_vec_seq;
use pbbs::common::graph_io::read_edge_array_from_file;
use pbbs::benchmarks::spanning_forest::serial_sf::spanning_forest;

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

pub fn check(ea: &EdgeArray, out: &[DefInt]) -> bool {
    let n = out.len();

    //run serial ST
    let serial_st = spanning_forest(ea);
    if n != serial_st.len() {
        println!(
            "Wrong edge count: ST has {}
                edges but algorithm returned {} edges.",
            serial_st.len(),
            n
        );
        return false;
    }

    //check if ST has cycles by running serial ST on it
    //and seeing if result changes
    let mut flags = vec![false; ea.non_zeros];
    out
        .iter()
        .for_each(|&o| flags[o as usize] = true);
    let mut new_es = vec![];
    pack(&ea.es, &flags, &mut new_es);
    let m = new_es.len();

    let new_ea = EdgeArray::new(new_es, ea.num_rows, ea.num_cols);
    let check = spanning_forest(&new_ea);

    if m != check.len() {
        println!("Result is not a spanning tree");
        return false
    }

    true
}

fn main() {
    let args = Args::parse();
    let ea = read_edge_array_from_file(&args.ifname);
    let r: Vec<DefInt> = read_file_to_vec_seq(&args.rfname);
    if check(&ea, &r) { println!("OK"); }
    else { println!("ERR"); std::process::exit(1); }
}
