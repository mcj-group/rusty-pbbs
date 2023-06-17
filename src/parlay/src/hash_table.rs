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

use std::cmp::Ordering;
use rayon::prelude::*;

type Idx = usize;

pub trait HashHelper {
    type KT;
    type ET: Copy + Clone + Send + Sync;

    fn empty() -> Self::ET;
    fn hash(s: Self::KT) -> usize;
    fn is_empty(s: &Self::ET) -> bool;
    fn get_key(v: Self::ET) -> Self::KT;
    fn cmp(s: Self::KT, s2:Self::KT) -> Ordering;
    fn replace_q(s: Self::ET, s2: Self::ET) -> bool;
    fn cas(p: &mut Self::ET, o: Self::ET, n: Self::ET) -> bool;
}

pub struct HashTable<H: HashHelper>
{
    m: usize,
    ta: Vec<H::ET>,
}

impl<'a, H: HashHelper> HashTable<H>
{
    fn _clear(_a: &'a mut [H::ET]) {
        todo!();
    }

    fn hash_to_range(&self, h: Idx) -> Idx {
        h % self.m
    }

    fn first_index(&self, v: H::KT) -> Idx {
        self.hash_to_range(H::hash(v))
    }

    fn increment_index(&self, h: Idx) -> Idx {
        if h + 1 == self.m { 0 } else { h + 1 }
    }

    fn _decrement_index(&self, h: Idx) -> Idx {
        if h == 0 { self.m - 1 } else { h - 1 }
    }

    fn _less_index(&self, a: Idx, b: Idx) -> bool {
        if a < b { 2 * (b - a) < self.m } else { 2 * (a - b) > self.m }
    }

    fn _less_eq_index(&self, a: Idx, b: Idx) -> bool {
        a == b || self._less_index(a, b)
    }


    pub fn new(size: usize, load: f64) -> Self {
        let m = (size as f64 * load) as usize + 100;
        Self {
            m,
            ta: vec![H::empty(); m]
        }
    }

    pub fn insert(&self, mut v: H::ET) -> bool {
        let mut i = self.first_index(H::get_key(v));
        loop {
            let c = self.ta[i];
            let clone = unsafe {
                (self.ta.as_ptr().add(i) as *mut H::ET).as_mut().unwrap()
            };
            if H::is_empty(&c) {
                if H::cas(clone, c, v) { return true; }
            } else {
                match H::cmp(H::get_key(v), H::get_key(c)) {
                    Ordering::Less => i = self.increment_index(i),
                    Ordering::Equal => {
                        if !H::replace_q(v, c) { return false; }
                        else if H::cas(clone, c, v) { return true; }
                    },
                    Ordering::Greater => {
                        if H::cas(clone, c, v) {
                            v = c;
                            i = self.increment_index(i);
                        }
                    }
                }
            }
        }
    }

    pub fn _update(&self, _v: H::ET) -> bool {
        todo!()
    }

    pub fn _delete_val(&self, _v: H::KT) -> bool {
        todo!()
    }

    pub fn find(&self, v: H::KT) -> Option<H::ET>
    where
        H::KT: Copy
    {
        let mut h = self.first_index(v);
        let mut c = self.ta[h];
        loop {
            match H::cmp(v, H::get_key(c)) {
                Ordering::Less => {
                    h = self.increment_index(h);
                    c = self.ta[h];
                },
                Ordering::Equal => return Some(c),
                Ordering::Greater => return None,
            }
        }
    }

    pub fn _count(&self) -> usize {
        todo!()
    }

    pub fn entries(&self) -> Vec<H::ET> {
        // FIXME:JA: replace with our own filter after implementing that
        self.ta
            .par_iter()
            .cloned()
            .filter(|v| !H::is_empty(v))
            .collect()
    }

    pub fn _find_index(&self, _v: H::KT) -> Idx {
        todo!()
    }

    pub fn _get_index(&self) -> Vec<Idx> {
        todo!()
    }

    pub fn print(&self) {
        println!("implement Display for ET");
        print!("vals = ");
        for i in 0..self.m {
            if !H::is_empty(&self.ta[i]) { print!("{i}, ") }
            // if &self.ta[i] as *const H::ET != &self.empty as *const H::ET { print!("{i}, ") }
            // if self.ta[i] != self.empty { print!("{i}:{}, ", self.ta[i]) }
        }
        println!();
    }
}
