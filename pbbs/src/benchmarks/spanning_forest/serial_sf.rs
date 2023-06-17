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

use parlay::maybe_uninit_vec;
use crate::{DefInt, DefIntS};
use crate::graph::EdgeArray;
use crate::union_find::UnionFind;

pub fn spanning_forest(ea: &EdgeArray) -> Vec<DefInt> {
    let n = ea.num_rows;
    let m = ea.non_zeros;
    let mut n_inst = 0;
    let mut st = maybe_uninit_vec![0; n];
    let mut uf = UnionFind::new(n);

    for i in 0..m {
        let u = uf.find(ea[i].u as DefIntS);
        let v = uf.find(ea[i].v as DefIntS);
        if u != v {
            uf.union_roots(u, v);
            st[n_inst] = i as DefInt;
            n_inst += 1;
        }
    }
    st[..n_inst].to_vec()
}