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

#[path="../common/atomics.rs"] mod atomics;

use std::mem::swap;
use std::sync::atomic::Ordering;

use atomics::atomic_cas;
use crate::{DefIntS, DefAtomIntS};

static ORD: Ordering = Ordering::Relaxed;


pub struct UnionFind {
    parents: Vec<DefIntS>
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self { parents: vec![-1; n] }
    }

    fn is_root(&self, u: DefIntS) -> bool {
        self.parents[u as usize] < 0
    }

    pub fn find(&mut self, mut u: DefIntS) -> DefIntS {
        if self.is_root(u) { return u; }
        let mut p = self.parents[u as usize];
        if self.is_root(p) { return p; }

        loop {
            let gp = self.parents[p as usize];
            self.parents[u as usize] = gp;
            u = p;
            p = gp;
            if self.is_root(p) { return p;}
        }
    }

    pub fn union_roots(&mut self, u: DefIntS, v: DefIntS) {
        let (mut u, mut v) = (u as usize, v as usize);
        if self.parents[u] < self.parents[v]{
            swap(&mut u, &mut v);
        };
        self.parents[u] += self.parents[v];
        self.parents[v] = u as DefIntS;
    }

    pub fn link(&mut self, u: DefIntS, v: DefIntS) {
        self.parents[u as usize] = v;
    }

    pub fn try_link(&mut self, u: DefIntS, v: DefIntS) -> bool {
        self.parents[u as usize] == -1 &&
            atomic_cas(&mut self.parents[u as usize], -1, v)
    }
}

pub struct AtomicUnionFind {
    parents: Vec<DefAtomIntS>
}

impl AtomicUnionFind {
    pub fn new(n: usize) -> Self {
        Self { parents: (0..n).map(|_| DefAtomIntS::new(-1)).collect() }
    }

    fn is_root(&self, u: DefIntS) -> bool {
        self.parents[u as usize].load(ORD) < 0
    }

    pub fn find(&self, u: DefIntS) -> DefIntS {
        if self.is_root(u) { return u; }
        let mut p = self.parents[u as usize].load(ORD);
        if self.is_root(p) { return p; }

        let mut u = u;
        loop {
            let gp = self.parents[p as usize].load(ORD);
            self.parents[u as usize].store(gp, ORD);
            u = p;
            p = gp;
            if self.is_root(p) { return p;}
        }
    }

    pub fn union_roots(&self, u: DefIntS, v: DefIntS) {
        let (mut u, mut v) = (u as usize, v as usize);
        if self.parents[u].load(ORD) < self.parents[v].load(ORD) {
            swap(&mut u, &mut v);
        };
        self.parents[u].fetch_add(self.parents[v].load(ORD), ORD);
        self.parents[v].store(u as DefIntS, ORD);
    }

    pub fn link(&self, u: DefIntS, v: DefIntS) {
        self.parents[u as usize].store(v, ORD);
    }

    pub fn try_link(&self, u: DefIntS, v: DefIntS) -> bool {
        self.parents[u as usize].load(ORD) == -1 &&
            self.parents[u as usize].compare_exchange(-1, v, ORD, ORD).is_ok()
    }
}
