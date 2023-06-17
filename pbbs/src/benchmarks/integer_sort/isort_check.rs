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

use pbbs::common::io::read_file_to_vec;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// sort results filename
    #[clap(value_parser, required=true)]
    rfname: String,
}

pub fn check(arr: &[usize]) -> Result<(), String> {
    let mut violation_no = 0usize;
    for i in 0..arr.len()-1 {
        if arr[i] > arr[i+1] { violation_no += 1; }
    }
    if violation_no != 0 { Err("{violation_no} violations}") }
    else { Ok(()) }
}

fn main() {
    let args = Args::parse();
    let r: Vec<usize> = read_file_to_vec(
        &args.rfname,
        Some{ 0:|_: &[&str]| {debug_assert_eq!(true, true)} }
    );

    check(&r).unwrap();
}
