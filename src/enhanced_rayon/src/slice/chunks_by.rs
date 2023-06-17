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
use std::marker::PhantomData;
use rayon::iter::plumbing::*;
use rayon::iter::*;


/// Parallel iterator over mutable non-overlapping chunks of a slice
#[derive(Debug)]
pub struct ChunksMutBy<'data, T: Send, O> {
    offset: O,
    range: Range<usize>,
    slice: &'data mut [T],
}

impl<'data, T, O> ChunksMutBy<'data, T, O>
where
    T: Send,
    O: Fn(usize) -> usize + Send + Clone,
{
    pub(super) fn new(
        offset: O,
        range: Range<usize>,
        slice: &'data mut [T]
    ) -> Self {
        Self { offset, range, slice }
    }
}

impl<'data, T, O> ParallelIterator for ChunksMutBy<'data, T, O>
where
    T: Send + 'data,
    O: Fn(usize) -> usize + Send + Clone,
{
    type Item = &'data mut [T];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'data, T, O> IndexedParallelIterator for ChunksMutBy<'data, T, O>
where
    T: Send + 'data,
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
        callback.callback(ChunksMutByProducer {
            offset: self.offset,
            range: self.range,
            slice: self.slice,
        })
    }
}

struct ChunksMutByProducer<'data, T: Send, O> {
    offset: O,
    range: Range<usize>,
    slice: &'data mut [T],
}

impl<'data, T, O> Producer for ChunksMutByProducer<'data, T, O>
where
    O: Fn(usize) -> usize + Send + Clone,
    T: 'data + Send,
{
    type Item = &'data mut [T];
    type IntoIter = ChunkSeqMut<'data, T, O>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkSeqMut {
            offset: self.offset,
            range: self.range,
            ptr: self.slice,
            _marker: PhantomData,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let bias = self.range.start;
        let size = (self.offset)(index + bias) - (self.offset)(bias);
        let (left, right) = self.slice.split_at_mut(size);
        (
            ChunksMutByProducer {
                offset: self.offset.clone(),
                range: bias..index + bias,
                slice: left,
            },
            ChunksMutByProducer {
                offset: self.offset,
                range: index + bias..self.range.end,
                slice: right,
            },
        )
    }
}


pub(super) struct ChunkSeqMut<'data, T, O>
where
    T: 'data,
    O: Fn(usize) -> usize + Clone,
{
    offset: O,
    range: Range<usize>,
    ptr: *mut [T],
    _marker: PhantomData<&'data mut T>
}

impl <'data, T, O> Iterator for ChunkSeqMut<'data, T, O>
where
    O: Fn(usize) -> usize + Clone,
{
    type Item = &'data mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        match self.range.len() {
            0 => None,
            1 => {
                self.range = self.range.end..self.range.end;
                Some(unsafe { &mut *self.ptr })
            },
            _ => {
                let bias = self.range.start;
                let size = (self.offset)(1 + bias) - (self.offset)(bias);
                self.range = bias + 1..self.range.end;
                let (left, right) = unsafe { (*self.ptr).split_at_mut(size) };
                self.ptr = right as *mut [T];
                Some(left)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl <'data, T, O> ExactSizeIterator for ChunkSeqMut<'data, T, O>
where
    O: Fn(usize) -> usize + Clone,
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl <'data, T, O> DoubleEndedIterator for ChunkSeqMut<'data, T, O>
where
    O: Fn(usize) -> usize + Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.len() {
            0 => None,
            1 => {
                self.range = self.range.start..self.range.start;
                Some(unsafe { &mut *self.ptr })
            },
            n => {
                let bias = self.range.start;
                let skip = (self.offset)(n - 1 + bias) - (self.offset)(bias);
                self.range = bias..n - 1 + bias;
                let (left, right) = unsafe { (*self.ptr).split_at_mut(skip) };
                self.ptr = left as *mut [T];
                Some(right)
            }
        }
    }
}

