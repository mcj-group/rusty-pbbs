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

use rayon::iter::plumbing::*;
use rayon::iter::*;
use num_traits::PrimInt;


/// `Chunks` is an iterator that groups elements of an underlying iterator.
///
/// This struct is created by the [`chunks()`] method on [`IndexedParallelIterator`]
///
/// [`chunks()`]: trait.IndexedParallelIterator.html#method.chunks
/// [`IndexedParallelIterator`]: trait.IndexedParallelIterator.html
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct Chunks<'offs, I: IndexedParallelIterator, O: PrimInt> {
    offsets: &'offs [O],
    i: I,
}

impl<'offs, I, O> Chunks<'offs, I, O>
where
    I: IndexedParallelIterator,
    O: PrimInt + Sync,
{
    pub(super) fn new(i: I, offsets: &'offs [O]) -> Self {
        Self { i, offsets }
    }
}

impl<'offs, I, O> ParallelIterator for Chunks<'offs, I, O>
where
    I: IndexedParallelIterator,
    O: PrimInt + Sync,
{
    type Item = Vec<I::Item>;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'offs, I, O> IndexedParallelIterator for Chunks<'offs, I, O>
where
    I: IndexedParallelIterator,
    O: PrimInt + Sync,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.offsets.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        let len = self.i.len();
        return self.i.with_producer(Callback {
            offsets: self.offsets,
            len,
            callback,
        });

        struct Callback<'offs, O, CB> {
            offsets: &'offs [O],
            len: usize,
            callback: CB,
        }

        impl<'offs, O, T, CB> ProducerCallback<T> for Callback<'offs, O, CB>
        where
            O: PrimInt + Sync,
            CB: ProducerCallback<Vec<T>>,
        {
            type Output = CB::Output;

            fn callback<P>(self, base: P) -> CB::Output
            where
                O: PrimInt + Sync,
                P: Producer<Item = T>,
            {
                let producer = ChunkProducer::new(
                    self.offsets,
                    self.len,
                    base,
                    Vec::from_iter
                );
                self.callback.callback(producer)
            }
        }
    }
}

pub(super) struct ChunkProducer<'offs, P, F, O: PrimInt> {
    offsets: &'offs [O],
    len: usize,
    base: P,
    map: F,
}

impl<'offs, P, F, O: PrimInt> ChunkProducer<'offs, P, F, O> {
    pub(super) fn new(offsets: &'offs [O], len: usize, base: P, map: F) -> Self {
        Self {
            offsets,
            len,
            base,
            map,
        }
    }
}

impl<'offs, P, F, O, T> Producer for ChunkProducer<'offs, P, F, O>
where
    O: PrimInt + Sync,
    P: Producer,
    F: Fn(P::IntoIter) -> T + Send + Clone
{
    type Item = T;
    type IntoIter = std::iter::Map<ChunkSeq<'offs, P, O>, F>;

    fn into_iter(self) -> Self::IntoIter {
        let chunks = ChunkSeq{
            offsets: self.offsets,
            len: self.len,
            inner: if self.offsets.len() > 0 { Some(self.base) } else { None },
        };
        chunks.map(self.map)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let elem_index = (self.offsets[index] - self.offsets[0]).to_usize().unwrap();
        let (left, right) = self.base.split_at(elem_index);
        (
            ChunkProducer {
                offsets: &self.offsets[..index],
                len: elem_index,
                base: left,
                map: self.map.clone(),
            },
            ChunkProducer {
                offsets: &self.offsets[index..],
                len: self.len - elem_index,
                base: right,
                map: self.map,
            },
        )
    }
}


pub(super) struct ChunkSeq<'offs, P, O: PrimInt> {
    offsets: &'offs [O],
    len: usize,
    inner: Option<P>,
}

impl <'offs, P, O> Iterator for ChunkSeq<'offs, P, O>
where
    P: Producer,
    O: PrimInt
{
    type Item = P::IntoIter;

    fn next(&mut self) -> Option<Self::Item> {
        let producer = self.inner.take()?;
        if self.offsets.len() != 1 {
            let size = (self.offsets[1] - self.offsets[0]).to_usize().unwrap();
            self.offsets = &self.offsets[1..];
            let (left, right) = producer.split_at(size);
            self.inner = Some(right);
            self.len -= size;
            Some(left.into_iter())
        } else {
            debug_assert!(self.offsets.len() > 0);
            self.len = 0;
            Some(producer.into_iter())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl <'offs, P, O> ExactSizeIterator for ChunkSeq<'offs, P, O>
where
    P: Producer,
    O: PrimInt
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

impl <'offs, P, O> DoubleEndedIterator for ChunkSeq<'offs, P, O>
where
    P: Producer,
    O: PrimInt
{
    fn next_back(&mut self) -> Option<Self::Item> {
                let producer = self.inner.take()?;
        match self.offsets.len() {
            1 => {
                self.len = 0;
                Some(producer.into_iter())
            }
            n => {
                let skip = (self.offsets[n - 1] - self.offsets[0]).to_usize().unwrap();
                self.offsets = &self.offsets[..n - 1];
                let (left, right) = producer.split_at(skip);
                self.inner = Some(left);
                self.len -= skip;
                Some(right.into_iter())
            }
        }
    }
}
