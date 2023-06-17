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

use std::ops::Range;
use rayon::iter::*;
use rayon::iter::plumbing::*;


/// `Chunks` is an iterator that groups elements of an underlying iterator.
///
/// This struct is created by the [`chunks()`] method on [`IndexedParallelIterator`]
///
/// [`chunks()`]: trait.IndexedParallelIterator.html#method.chunks
/// [`IndexedParallelIterator`]: trait.IndexedParallelIterator.html
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct ChunksBy<I, O> {
    offset: O,
    range: Range<usize>,
    i: I,
}

impl<I, O> ChunksBy<I, O>
where
    I: IndexedParallelIterator,
    O: Fn(usize) -> usize + Send + Clone,
{
    pub(super) fn new(i: I, offset: O, len: usize) -> Self {
        Self { i, offset, range: 0..len }
    }
}

impl<I, O> ParallelIterator for ChunksBy<I, O>
where
    I: IndexedParallelIterator,
    O: Fn(usize) -> usize + Send + Clone,
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

impl<I, O> IndexedParallelIterator for ChunksBy<I, O>
where
    I: IndexedParallelIterator,
    O: Fn(usize) -> usize + Send + Clone,
{
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.range.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        let len = self.i.len();
        return self.i.with_producer(Callback {
            offset: self.offset,
            range: self.range,
            len,
            callback,
        });

        struct Callback<O, CB> {
            offset: O,
            range: Range<usize>,
            len: usize,
            callback: CB,
        }

        impl<O, T, CB> ProducerCallback<T> for Callback<O, CB>
        where
            O: Fn(usize) -> usize + Send + Clone,
            CB: ProducerCallback<Vec<T>>,
        {
            type Output = CB::Output;

            fn callback<P>(self, base: P) -> CB::Output
            where
                P: Producer<Item = T>,
            {
                let producer = ChunkProducer::new(
                    self.offset,
                    self.range,
                    self.len,
                    base,
                    Vec::from_iter
                );
                self.callback.callback(producer)
            }
        }
    }
}

pub(super) struct ChunkProducer<P, F, O> {
    offset: O,
    range: Range<usize>,
    len: usize,
    base: P,
    map: F,
}

impl<P, F, O: Fn(usize) -> usize> ChunkProducer<P, F, O> {
    pub(super) fn new(offset: O, range: Range<usize>, len: usize, base: P, map: F) -> Self {
        Self {
            offset,
            range,
            len,
            base,
            map,
        }
    }
}

impl<P, F, O, T> Producer for ChunkProducer<P, F, O>
where
    O: Fn(usize) -> usize + Send + Clone,
    P: Producer,
    F: Fn(P::IntoIter) -> T + Send + Clone
{
    type Item = T;
    type IntoIter = std::iter::Map<ChunkSByeq<P, O>, F>;

    fn into_iter(self) -> Self::IntoIter {
        let chunks = ChunkSByeq{
            offset: self.offset,
            len: self.len,
            inner: if self.range.len() > 0 { Some(self.base) } else { None },
            range: self.range,
        };
        chunks.map(self.map)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let bias = self.range.start;
        let size = (self.offset)(index + bias) - (self.offset)(bias);
        let (left, right) = self.base.split_at(size);
        (
            ChunkProducer {
                offset: self.offset.clone(),
                range: bias..bias + index,
                len: size,
                base: left,
                map: self.map.clone(),
            },
            ChunkProducer {
                offset: self.offset,
                range: bias + index..self.range.end,
                len: self.len - size,
                base: right,
                map: self.map,
            },
        )
    }
}


pub(super) struct ChunkSByeq<P, O> {
    offset: O,
    range: Range<usize>,
    len: usize,
    inner: Option<P>,
}

impl <P, O> Iterator for ChunkSByeq<P, O>
where
    P: Producer,
    O: Fn(usize) -> usize
{
    type Item = P::IntoIter;

    fn next(&mut self) -> Option<Self::Item> {
        let producer = self.inner.take()?;
        if self.range.len() != 1 {
            let bias = self.range.start;
            let size = (self.offset)(bias+1) - (self.offset)(bias);
            self.range = bias+1..self.range.end;
            let (left, right) = producer.split_at(size);
            self.inner = Some(right);
            self.len -= size;
            Some(left.into_iter())
        } else {
            debug_assert!(self.range.len() > 0);
            self.len = 0;
            Some(producer.into_iter())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl <P, O> ExactSizeIterator for ChunkSByeq<P, O>
where
    P: Producer,
    O: Fn(usize) -> usize
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl <P, O> DoubleEndedIterator for ChunkSByeq<P, O>
where
    P: Producer,
    O: Fn(usize) -> usize
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let producer = self.inner.take()?;
        match self.range.len() {
            1 => {
                self.len = 0;
                Some(producer.into_iter())
            }
            n => {
                let bias = self.range.start;
                let skip = (self.offset)(n-1+bias) - (self.offset)(bias);
                self.range = bias..n-1+bias;
                let (left, right) = producer.split_at(skip);
                self.inner = Some(left);
                self.len -= skip;
                Some(right.into_iter())
            }
        }
    }
}
