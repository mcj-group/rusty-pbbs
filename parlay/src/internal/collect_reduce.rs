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

use std::mem::size_of;
use rayon::prelude::*;
use num_traits::{PrimInt, ToPrimitive};

use crate::primitives::flatten_by_val;
use crate::utilities::{log2_up, hash64};
use crate::{maybe_uninit_vec, DefInt, Timer};
use crate::internal::counting_sort::count_sort;
use crate::internal::integer_sort::integer_sort_;

const CR_SEQ_THR: usize = 8192;
const CRS_SEQ_THR: usize = 10000;
const CACHE_PER_THREAD: usize = 1000000;


pub trait HashEq {
    type IT;
    type KT;
    type RT;

    fn hash(&self, _a: Self::KT) -> usize { todo!() }
    fn get_key(&self, _a: Self::IT) -> Self::KT { todo!() }
    
    fn get_key_mut<'a>(&'a self, _a: &'a mut Self::RT) -> &mut Self::KT {
        todo!()
    }
    
    fn get_key_from_result(&self, _a: Self::RT) -> Self::KT { todo!() }
    fn equal(&self, _a: Self::KT, _b: Self::KT) -> bool { todo!() }
}

struct GetBucket<KT: Copy + Default, HEQ>
{
    hash_table: Vec<(KT, i32)>,
    table_mask: usize,
    bucket_mask: usize,
    heavy_hitters: usize,
    heq: HEQ,
}

impl<IT, KT, HEQ> GetBucket<KT, HEQ>
where
    IT: Clone + Copy,
    KT: Copy + Default + Send,
    HEQ: HashEq<IT=IT, KT=KT>,
{
    fn new(inp: &[IT], bits: usize, heq: HEQ) -> Self {
        let n = inp.len();
        let num_buckets = 1 << bits;
        const COPY_CUTOFF: usize = 5;
        let num_samples = num_buckets;
        let table_size = 4 * num_samples;
        let table_mask = table_size - 1;
        let bucket_mask = num_buckets - 1;

        let mut hash_table_count = (0..table_size)
            .into_par_iter()
            .map(|_| (KT::default(), -1i32))
            .collect::<Vec<_>>();

        for i in 0..num_samples {
            let a = inp[hash64(i as u64) as usize % n];
            let s = heq.get_key(a);
            let mut idx = heq.hash(s) & table_mask;
            loop {
                let htc = &mut hash_table_count[idx];
                if htc.1 == -1 {
                    *htc = (s, 0);
                    break;
                } else if heq.equal(htc.0, s) {
                    htc.1 += 1;
                    break;
                } else {
                    idx = (idx + 1) & table_mask;
                }
            }
        }

        let mut heavy_hitters = 0usize;
        let mut hash_table = (0..table_size)
            .into_par_iter()
            .map(|_| (KT::default(), -1))
            .collect::<Vec<_>>();
        for i in 0..table_size {
            let htc = &hash_table_count[i];
            if (htc.1 + 2) as usize > COPY_CUTOFF {
                let key = htc.0;
                let idx = heq.hash(key) & table_mask;
                if hash_table[idx].1 == -1 {
                    hash_table[idx] = (key, heavy_hitters as i32);
                    heavy_hitters += 1;
                }
            }
        }

        Self { hash_table, table_mask, bucket_mask, heavy_hitters, heq }
    }

    fn op(&self, v: &IT) -> usize {
        let mut hash_val = self.heq.hash(self.heq.get_key(*v));
        if self.heavy_hitters > 0 {
            let h = &self.hash_table[hash_val & self.table_mask];
            if h.1 != -1 && self.heq.equal(h.0, self.heq.get_key(*v)) {
                return h.1 as usize;
            }
            hash_val = hash_val & self.bucket_mask;
            if (hash_val & self.bucket_mask) < self.heavy_hitters {
                return hash_val % (self.bucket_mask+1-self.heavy_hitters)
                    + self.heavy_hitters
            }
            return hash_val & self.bucket_mask;
        }
        return hash_val & self.bucket_mask;
    }
}

pub trait RCHashEq {
    type IT;
    type KT;

    fn init(&self) -> Self::IT;
    fn get_val(&self, a: Self::IT) -> Self::IT;
    fn update(&self, r: &mut Self::IT, inp: Self::IT);
    fn combine(&self, r: &mut Self::IT, inp: &[Self::IT]);
}

pub fn seq_collect_reduce<T, HEQ>(
    inp: &[T],
    helper: HEQ,
    num_buckets: usize,
    res: &mut Vec<T>
) where
    T: PrimInt + Default,
    HEQ: RCHashEq<IT=T> + HashEq<IT=T> + Copy + Clone,
    <HEQ as HashEq>::KT: PrimInt + Default,
    <HEQ as RCHashEq>::KT: PrimInt + Default,
{
    *res = vec![helper.init(); num_buckets];
    for j in 0..inp.len() {
        let k = helper.get_key(inp[j]).to_usize().unwrap();
        debug_assert!(k < num_buckets);
        helper.update(&mut res[k], helper.get_val(inp[j]));
    }
}

pub fn collect_reduce_few<T, HEQ>(
    inp: &[T],
    helper: HEQ,
    num_buckets: usize,
    res: &mut Vec<T>
) where
    T: PrimInt + Send + Sync + Default,
    HEQ: RCHashEq<IT=T> + HashEq<IT=T> + Send + Sync + Copy + Clone,
    <HEQ as HashEq>::KT: PrimInt + Default + Send + Sync,
    <HEQ as RCHashEq>::KT: PrimInt + Default + Send + Sync,
{
    let n = inp.len();
    let num_threads = rayon::current_num_threads();
    let num_blocks = (4*num_threads).min(n/num_buckets/64) + 1;

    // if insufficient parallelism, do sequentially
    if n < CR_SEQ_THR || num_blocks == 1 || num_threads == 1 {
        seq_collect_reduce(inp, helper, num_buckets, res);
        return;
    }

    // partial results for each block
    let block_size = ((n - 1) / num_blocks) + 1;
    let mut out: Vec<Vec<T>> = maybe_uninit_vec![vec![]; num_blocks];
    out
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, oi)| {
            let s = i * block_size;
            let e = n.min(s + block_size);
            seq_collect_reduce(&inp[s..e], helper, num_buckets, oi);
        });

    // comibine partial results into total result
    *res = (0..num_buckets)
        .into_par_iter()
        .map(|i| {
            let mut o_val = helper.init();
            for j in 0..num_blocks {
                helper.update(&mut o_val, out[j][i]);
            }
            o_val
        }).collect();
}

pub fn collect_reduce<T, HEQ>(
    inp: &[T],
    helper: HEQ,
    num_buckets: usize,
    res: &mut Vec<T>
) where
    T: PrimInt + Send + Sync + Default,
    HEQ: RCHashEq<IT=T> + HashEq<IT=T> + Send + Sync + Copy + Clone,
    <HEQ as HashEq>::KT: PrimInt + Default + Send + Sync,
    <HEQ as RCHashEq>::KT: PrimInt + Default + Send + Sync,
{
    let mut t = Timer::new("collect reduce"); //t.start();

    let n = inp.len();

    let bits = (
        log2_up(1 + (2 * size_of::<T>() * n)
        / CACHE_PER_THREAD) as usize
    ).max(4);
    let num_blocks = 1 << bits;

    if num_buckets <= 4 * num_blocks || n < CR_SEQ_THR {
        collect_reduce_few(inp, helper, num_buckets, res);
        return;
    }

    let gb = GetBucket::new(inp, bits, helper);

    let mut b: Vec<T> = maybe_uninit_vec![T::default(); n];
    let mut tmp: Vec<T> = maybe_uninit_vec![T::default(); n];

    // first partition into blocks based on hash using a counting sort
    let get_key = |a: T| { gb.op(&a) as DefInt };
    let block_offsets = integer_sort_(
        inp,
        &mut b,
        &mut tmp,
        &get_key,
        bits,
        num_blocks
    );
    t.next("sort");

    // results
    #[cfg(feature = "AW_safe")]
    {
        use std::sync::Mutex;
        let sums = (0..num_buckets)
            .into_par_iter()
            .map(|_| Mutex::new(helper.init()))
            .collect::<Vec<_>>();

        (0..num_blocks).into_par_iter().for_each(|i| {
            let slice =
                &b[block_offsets[i] as usize..block_offsets[i+1] as usize];

            if i < gb.heavy_hitters {   // heavy hitters with all equal keys
                let k = helper.get_key(slice[0]).to_usize().unwrap();
                helper.combine(
                    &mut sums[k].lock().unwrap(),
                    slice
                );
            } else {    // shared blocks
                for j in 0..slice.len() {
                    let k = helper.get_key(slice[j]).to_usize().unwrap();
                    debug_assert!(k < num_buckets);
                    helper.update(
                        &mut sums[k].lock().unwrap(),
                        helper.get_val(slice[j])
                    );
                }
            }
        });

        *res = sums.into_iter().map(|m| m.into_inner().unwrap()).collect();
    }
    #[cfg(not(feature = "AW_safe"))]
    {
        let sums = (0..num_buckets)
            .into_par_iter()
            .map(|_| helper.init())
            .collect::<Vec<_>>();

        (0..num_blocks).into_par_iter().for_each(|i| {
            let slice =
                &b[block_offsets[i] as usize..block_offsets[i+1] as usize];

            if i < gb.heavy_hitters {   // heavy hitters with all equal keys
                let k = helper.get_key(slice[0]).to_usize().unwrap();
                helper.combine(
                    unsafe { &mut *(sums.as_ptr().add(k) as *mut T) },
                    slice
                );
            } else {    // shared blocks
                for j in 0..slice.len() {
                    let k = helper.get_key(slice[j]).to_usize().unwrap();
                    debug_assert!(k < num_buckets);
                    helper.update(
                        unsafe { &mut *(sums.as_ptr().add(k) as *mut T) },
                        helper.get_val(slice[j])
                    );
                }
            }
        });
        t.next("into_tables");
    
        *res = sums;
    }
}



pub trait RCSHashEq {
    type IT;
    type KT;
    type RT;

    fn init(&self, r: &mut Self::RT, inp: Self::IT);
    fn reduce(&self, s: &[Self::IT]) -> Self::RT;
    fn update(&self, r: &mut Self::RT, inp: Self::IT);
}

pub fn seq_collect_reduce_sparse<T, R, HEQ>(
    inp: &[T],
    helper: HEQ,
    res: &mut Vec<R>
)
where
    T: Send + Sync + Clone + Copy + Default,
    R: Send + Sync + Clone + Copy + Default,
    HEQ: RCSHashEq<IT=T, RT=R> + HashEq<IT=T, RT=R> + Send + Sync,
    <HEQ as HashEq>::KT: Copy + Default + Send + Sync,
    <HEQ as RCSHashEq>::KT: Copy + Default + Send + Sync,
{
    let table_size = 3 * inp.len() / 2;
    let mut count = 0usize;
    let mut table: Vec<R> = maybe_uninit_vec![R::default(); table_size];
    let mut flags = vec![false; table_size];

    // hash into buckets
    for j in 0..inp.len() {
        let key = helper.get_key(inp[j]);
        let mut k: usize = helper.hash(key) % table_size;
        while
            flags[k]
            && !helper.equal(helper.get_key_from_result(table[k]), key)
        {
            k = if k + 1 == table_size { 0 }  else { k + 1 } ;
        }

        if flags[k] {
            helper.update(&mut table[k], inp[j]);
        } else {
            flags[k] = true;
            count+=1;
            helper.init(&mut table[k], inp[j]);
            *helper.get_key_mut(&mut table[k]) = helper.get_key(inp[j]);
        }
    }

    // pack non-empty entries of table into result sequence
    let mut r: Vec<R> = maybe_uninit_vec![R::default(); count];
    let mut j = 0usize;
    for i in 0..table_size {
        if flags[i] { r[j] = table[i]; j+=1; }}
    debug_assert_eq!(j, count);

    *res = r;
}

pub fn collect_reduce_sparse<T, R, HEQ>(
    inp: &[T],
    helper: HEQ,
    res: &mut Vec<R>
) where
    T: Copy + Eq + Send + Sync + Default,
    R: Copy + Send + Sync + Default,
    HEQ: RCSHashEq<IT=T, RT=R> + HashEq<IT=T, RT=R> + Send + Sync + Copy + Clone,
    <HEQ as HashEq>::KT: Copy + Default + Send + Sync,
    <HEQ as RCSHashEq>::KT: Copy + Default + Send + Sync,
{
    let mut t = Timer::new("collect reduce sparse"); //t.start();

    let n = inp.len();
    if n < CRS_SEQ_THR {
        seq_collect_reduce_sparse(&inp, helper, res);
        t.next("seq_collect");
        return;
    }

    let bits = log2_up(
        (1.0 + (1.2 * 2.0 * size_of::<T>() as f64 * n as f64)
        / CACHE_PER_THREAD as f64) as usize
    ).max(4);
    let num_buckets = 1 << bits;

    let gb = GetBucket::new(inp, bits, helper);

    let mut b: Vec<T> = maybe_uninit_vec![T::default(); n];
    #[allow(unused_mut)]
    let keys: Vec<usize> = (0..n)
        .into_par_iter()
        .map(|i| gb.op(&inp[i]))
        .collect();
    let (bucket_offsets, _) = count_sort(inp, &mut b, &keys, num_buckets, 1.0);
    t.next("integer sort");

    let heavy_cutoff = gb.heavy_hitters;
    let tables: Vec<Vec<R>> = (0..num_buckets)
        .into_par_iter()
        .map(|i| {
            let block =
                &b[bucket_offsets[i] as usize..bucket_offsets[i+1] as usize];
            if i < heavy_cutoff {
                vec![helper.reduce(block)]
            } else {
                let mut r = vec![];
                seq_collect_reduce_sparse(&block, helper, &mut r);
                r
            }
        }).collect();
    t.next("block hash");

    // flatten the results
    flatten_by_val(&tables, res);
    t.next("flatten");
}
