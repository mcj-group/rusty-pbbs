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

use crate::DefInt;
use crate::graph::EdgeArray;


pub fn maximal_matching(ea: &EdgeArray) -> Vec<DefInt> {
    let n = ea.num_rows.max(ea.num_cols);
    let m = ea.non_zeros;
    let mut matching = vec![DefInt::default(); n];
    let mut matched = vec![false; n];
    let mut offset = 0;

    for i in 0..m {
        let e = &ea[i];
        let (u, v) = (e.u as usize, e.v as usize);
        if matched[u] || matched[v] { continue; }
        else {
            (matched[u], matched[v]) = (true, true);
            matching[offset] = i as DefInt;
            offset += 1;
        }
    }
    matching[..offset].to_owned()
}
