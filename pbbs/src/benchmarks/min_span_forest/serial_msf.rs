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

use std::cmp::{Ordering, min};

use crate::{DefInt, DefIntS, DefFloat};
use crate::graph::{WghEdge, WghEdgeArray};
use crate::union_find::UnionFind;

#[derive(Clone, Copy)]
pub struct IndexedEdge { pub u: DefInt, pub v: DefInt, pub id: DefInt, pub w: DefFloat }

impl IndexedEdge {
    pub fn new(e: WghEdge, id: DefInt) -> Self {
        Self { u: e.u, v: e.v, id, w: e.w }
    }
}

impl Default for IndexedEdge {
    fn default() -> Self {
        Self { u: 0, v: 0, id: 0, w: 0.0 }
    }
}

#[inline(always)]
fn cmp_idx_edge(a: &IndexedEdge, b: &IndexedEdge) -> Ordering {
    if a.w < b.w || (a.w == b.w && a.id < b.id) { Ordering::Less }
    else { Ordering::Greater }
}

fn union_find_loop(es: &[IndexedEdge], m: usize, uf: &mut UnionFind, msf: &mut Vec<DefInt>) {
    for i in 0..m {
        let u = uf.find(es[i].u as DefIntS);
        let v = uf.find(es[i].v as DefIntS);
        if u != v {
            uf.union_roots(u, v);
            msf.push(es[i].id);
        }
    }
}

pub fn minimum_spanning_forest(wea: &WghEdgeArray, dest: &mut Vec<DefInt>) {
    eprintln!(
        "Serial MSF has an unidentified bug on some inputs. \
        It's better not to use this function."
    );

    let m = wea.m;
    let n = wea.n;

    let mut wea: Vec<IndexedEdge> = (0..m)
        .map(|i| IndexedEdge::new(wea[i], i as u32))
        .collect();

    let l = min(4*n/3, m);
    wea.select_nth_unstable_by(if l==m {l-1} else {l}, cmp_idx_edge);
    wea[..l].sort_by(cmp_idx_edge);

    let mut uf = UnionFind::new(n);

    union_find_loop(&wea, l, &mut uf, dest);

    let mut k = 0;
    for i in l..m {
        let u = uf.find(wea[i].u as DefIntS);
        let v = uf.find(wea[i].v as DefIntS);
        if u != v { wea[l + k] = wea[i]; }
        k += 1;
    }

    wea[l..l+k].sort_by(cmp_idx_edge);
    union_find_loop(&wea[l..], k, &mut uf, dest);
}
