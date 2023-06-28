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

#![feature(slice_partition_dedup)]
use clap::Parser;
use rayon::prelude::*;

#[path ="../../common/io.rs"] mod io;
use io::read_file_to_vec;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// BW results filename
    #[clap(value_parser, required=true)]
    rfname: String,

    /// the input graph's filename
    #[clap(value_parser, required=true)]
    ifname: String,

    /// the number of buckets
    #[clap(short, long, value_parser, required=false, default_value_t=0)]
    buckets: usize,
}

pub fn check(inp: &mut [usize], out: &mut [usize], buckets: usize) -> bool {
    assert_eq!(out.len(), buckets);

    inp.par_sort_unstable();
    let mut hist = vec![0; buckets];

    for &i in inp.iter() { hist[i] += 1; }

    let mut diff_count = 0usize;
    for i in 0..hist.len() {
        if hist[i] != out[i] {
            diff_count+=1;
        }
    }

    if diff_count!=0 {
        eprintln!("output file has {diff_count} differences.");
        false
    } else { true }
}

fn main() {
    eprintln!("WARNING: hist_check needs improvement.");
    let args = Args::parse();
    let mut inp = read_file_to_vec(
        &args.ifname,
        Some(|a: &[&str]| assert_eq!(a[0], "sequenceInt"))
    );
    let mut out = read_file_to_vec(&args.rfname, Some(|_: &[&str]| {}));
    if check(&mut inp, &mut out, args.buckets) { println!("OK"); }
    else { eprintln!("ERR"); std::process::exit(1); }
}
