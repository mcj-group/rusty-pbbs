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

#[path ="../../common/io.rs"] mod io;
use io::chars_from_file;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// BW results filename
    #[clap(value_parser, required=true)]
    rfname: String,

    /// the input graph's filename
    #[clap(value_parser, required=true)]
    ifname: String,
}

pub fn check(inp: &[u8], out: &[u8]) -> bool {
    if inp.len() != out.len() {
        eprintln!("files' lengthes differ.");
        return false;
    }
    let diff: Vec<_> = (inp, out)
        .into_par_iter()
        .filter(|(i, o)| i != o)
        .collect();
    if diff.len() == 0 { true }
    else {
        eprintln!("different chars: {}", diff.len());
        false
    }
}

fn main() {
    let args = Args::parse();
    let inp = chars_from_file(&args.ifname, false).unwrap();
    let out = chars_from_file(&args.rfname, false).unwrap();
    if check(&inp, &out) { println!("OK"); }
    else { eprintln!("ERR"); std::process::exit(1); }
}
