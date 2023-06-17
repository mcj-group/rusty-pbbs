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


/// Single indirect Parallel iterator over mutable items in a slice
#[derive(Debug)]
pub struct SngIndBy<'data, T: Send, O> {
    offset: O,
    range: Range<usize>,
    slice: &'data mut [T],
}

impl<'data, T, O> SngIndBy<'data, T, O>
where
    T: Send + 'data,
    O: Fn(usize) -> usize + Send + Clone,
{
    pub(super) unsafe fn new(slice: &'data mut [T], offset: O, len: usize) -> Self {
        Self { slice, offset, range: 0..len }
    }
}

impl<'data, T, O> ParallelIterator for SngIndBy<'data, T, O>
where
    T: Send + 'data,
    O: Fn(usize) -> usize + Send + Clone,
{
    type Item = &'data mut T;

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

impl<'data, T, O> IndexedParallelIterator for SngIndBy<'data, T, O>
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
        callback.callback(SngIndByProducer {
            slice: self.slice,
            offset: self.offset,
            range: self.range,
        })
    }
}

struct SngIndByProducer<'data, T: Send, F> {
    offset: F,
    slice: &'data mut [T],
    range: Range<usize>,
}

impl<'data, T, O> Producer for SngIndByProducer<'data, T, O>
where
    T: 'data + Send,
    O: Fn(usize) -> usize + Send + Clone,
{
    type Item = &'data mut T;
    type IntoIter = SngIndBySeq<'data, T, O>;

    fn into_iter(self) -> Self::IntoIter {
        SngIndBySeq {
            slice: self.slice,
            offset: self.offset,
            range: self.range,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let slice_copy = unsafe {
            std::slice::from_raw_parts_mut(
                self.slice.as_mut_ptr(),
                self.slice.len(),
            )
        };
        let bias = self.range.start;
        (
            SngIndByProducer {
                slice: self.slice,
                offset: self.offset.clone(),
                range: bias..bias+index
            },
            SngIndByProducer {
                slice: slice_copy,
                offset: self.offset,
                range: bias+index..self.range.end
            },
        )
    }
}

/// Single indirect Sequential iterator over items in a slice
pub(super) struct SngIndBySeq<'data, T, O> {
    offset: O,
    slice: &'data mut [T],
    range: Range<usize>,
}

impl <'data, T, O> Iterator for SngIndBySeq<'data, T, O>
where
    O: Fn(usize) -> usize + Send + Clone
{
    type Item = &'data mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range.len() != 0 {
            let idx = (self.offset)(self.range.start);
            let r = unsafe { self.slice.as_mut_ptr().add(idx).as_mut().unwrap() };
            self.range = self.range.start+1..self.range.end;
            Some(r)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.range.len();
        (len, Some(len))
    }
}

impl <'data, T, O> ExactSizeIterator for SngIndBySeq<'data, T, O>
where
    O: Fn(usize) -> usize + Send + Clone
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl <'data, T, O> DoubleEndedIterator for SngIndBySeq<'data, T, O>
where
    O: Fn(usize) -> usize + Send + Clone
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.range.len() {
            0 => None,
            _ => {
                let last = self.range.end-1;
                let idx = (self.offset)(last);
                let r = unsafe { self.slice.as_mut_ptr().add(idx).as_mut().unwrap() };
                self.range = self.range.start..last;
                Some(r)
            }
        }
    }
}
