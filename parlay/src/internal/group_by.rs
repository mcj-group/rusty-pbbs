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

use num_traits::PrimInt;
use std::marker::PhantomData;

use crate::utilities::hash64_cheap;
use super::collect_reduce::*;


#[derive(Clone, Copy)]
struct DedupHelper<T> { _p: PhantomData<T> }

impl<T: PrimInt> HashEq for DedupHelper<T>
{
    type IT = T;
    type KT = T;
    type RT = T;

    fn hash(&self, a: Self::KT) -> usize { a.to_usize().unwrap() }
    fn get_key(&self, a: Self::IT) -> Self::KT { a }
    fn get_key_mut<'a>(&'a self, a: &'a mut Self::IT) -> &mut Self::KT { a }
    fn get_key_from_result(&self, a: Self::RT) -> Self::KT { a }
    fn equal(&self, a: Self::KT, b: Self::KT) -> bool { a.eq(&b) }
}

impl<T: PrimInt> RCSHashEq for DedupHelper<T> where
{
    type IT = T;
    type KT = T;
    type RT = T;

    fn init(&self, _r: &mut Self::RT, _inp: Self::IT) {}
    fn reduce(&self, s: &[Self::IT]) -> Self::RT { return s[0]; }
    fn update(&self, _r: &mut Self::RT, _inp: Self::IT) {}
}

pub fn remove_duplicates<T>(inp: &[T], res: &mut Vec<T>)
where
    T: PrimInt + Default + Send + Sync,
{
    let helper = DedupHelper { _p: PhantomData };
    collect_reduce_sparse(inp, helper, res);
}

#[derive(Clone, Copy)]
struct HistHelper<T, K> {
    _t: PhantomData<T>,
    _k: PhantomData<K>
}

impl<T, K> HistHelper<T, K> {
    const SHIFT: usize = 8 / std::mem::size_of::<T>();

    fn new() -> Self {
        Self { _t: PhantomData, _k: PhantomData }
    }
}

impl<T: PrimInt, K: PrimInt> HashEq for HistHelper<T, K>
{
    type IT = T;
    type KT = K;
    type RT = T;

    fn hash(&self, a: Self::KT) -> usize {
        let v = ((a.to_usize().unwrap() + Self::SHIFT) & !15) as u64;
        hash64_cheap(v) as usize
    }

    fn get_key(&self, a: Self::IT) -> Self::KT { K::from(a).unwrap() }
    fn equal(&self, a: Self::KT, b: Self::KT) -> bool { a.eq(&b) }
}

impl<T: PrimInt + Default, K> RCHashEq for HistHelper<T, K>
{
    type IT = T;
    type KT = K;

    fn init(&self) -> Self::IT { T::zero() }
    fn get_val(&self, _a: Self::IT) -> Self::IT { T::one() }

    fn update(&self, r: &mut Self::IT, inp: Self::IT) {
        *r = *r + inp;
    }

    fn combine(&self, r: &mut Self::IT, inp: &[Self::IT]) {
        *r = T::from(inp.len()).unwrap();
    }
}

pub fn histogram_by_index<T, K>(
    inp: &[T],
    num_buckets: usize,
    res: &mut Vec<T>
) where
    T: PrimInt + Default + Send + Sync,
    K: PrimInt + Default + Send + Sync,
{
    let helper = HistHelper::<T, K>::new();
    collect_reduce(inp, helper, num_buckets, res);
}



#[derive(Clone, Copy)]
struct CountByKeyHelper<T, S, F> {
    _t: PhantomData<T>,
    _s: PhantomData<S>,
    hash_fn: F
}

impl<T, S, F> CountByKeyHelper<T, S, F>
where
    F: Fn(T) -> usize + Send + Sync + Copy + Clone
{
    fn new(hash_fn: F) -> Self {
        Self { _t: PhantomData, _s: PhantomData, hash_fn}
    }
}

impl<T, S, F> HashEq for CountByKeyHelper<T, S, F>
where
    T: Eq + Default + Send + Sync + Copy + Clone,
    S: PrimInt + Send + Sync + Default,
    F: Fn(T) -> usize + Send + Sync + Copy + Clone
{
    type IT = T;
    type KT = T;
    type RT = (T, S);

    fn hash(&self, a: Self::KT) -> usize { (self.hash_fn)(a) }
    fn get_key(&self, a: Self::IT) -> Self::KT { a }

    fn get_key_mut<'a>(&'a self, a: &'a mut Self::RT) -> &mut Self::KT {
        &mut a.0
    }

    fn get_key_from_result(&self, a: Self::RT) -> Self::KT { a.0 }
    fn equal(&self, a: Self::KT, b: Self::KT) -> bool { a.eq(&b) }
}

impl<T, S, F> RCSHashEq for CountByKeyHelper<T, S, F>
where
    T: Eq + Default + Send + Sync + Copy + Clone,
    S: PrimInt + Send + Sync + Default,
    F: Fn(T) -> usize + Send + Sync + Copy + Clone
{
    type IT = T;
    type KT = T;
    type RT = (T, S);

    fn init(&self, r: &mut Self::RT, _inp: Self::IT) { (*r).1 = S::one() }

    fn reduce(&self, s: &[Self::IT]) -> Self::RT {
        (s[0], S::from(s.len()).unwrap())
    }

    fn update(&self, r: &mut Self::RT, _inp: Self::IT) {
        (*r).1 = (*r).1 + S::one()
    }
}


pub fn histogram_by_key<T, S, F>(inp: &[T], hash: F, res: &mut Vec<(T,S)>)
where
    T: Eq + Default + Send + Sync + Copy + Clone,
    S: PrimInt + Default + Send + Sync,
    F: Fn(T) -> usize + Send + Sync + Copy + Clone
{
    let helper = CountByKeyHelper::<T, S, F>::new(hash);
    collect_reduce_sparse(inp, helper, res);
}
