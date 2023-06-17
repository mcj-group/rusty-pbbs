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
use rayon::iter::*;
use rayon::iter::plumbing::*;


/// Single indirect Parallel iterator over mutable items in a slice
#[derive(Debug)]
pub struct SngInd<'data, 'offs, T: Send, O: PrimInt> {
    offsets: &'offs [O],
    slice: &'data mut [T],
}

impl<'data, 'offs, T, O> SngInd<'data, 'offs, T, O>
where
    T: Send + 'data,
    O: PrimInt + Sync,
{
    pub(super) unsafe fn new(slice: &'data mut [T], offsets: &'offs [O]) -> Self {
        Self { slice, offsets }
    }
}

impl<'data, 'offs, T, O> ParallelIterator for SngInd<'data, 'offs, T, O>
where
    T: Send + 'data,
    O: PrimInt + Sync,
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

impl<'data, 'offs, T, O> IndexedParallelIterator for SngInd<'data, 'offs, T, O>
where
    T: Send + 'data,
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
        callback.callback(SngIndProducer {
            slice: self.slice,
            offsets: self.offsets,
        })
    }
}

struct SngIndProducer<'data, 'offs, T: Send, O: PrimInt> {
    offsets: &'offs [O],
    slice: &'data mut [T],
}

impl<'data, 'offs, T, O> Producer for SngIndProducer<'data, 'offs, T, O>
where
    T: 'data + Send,
    O: PrimInt + Sync,
{
    type Item = &'data mut T;
    type IntoIter = SngIndSeq<'data, 'offs, T, O>;

    fn into_iter(self) -> Self::IntoIter {
        SngIndSeq {
            slice: self.slice,
            offsets: self.offsets,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let slice_copy = unsafe {
            std::slice::from_raw_parts_mut(
                self.slice.as_mut_ptr(),
                self.slice.len(),
            )
        };
        let (left, right) = self.offsets.split_at(index);
        (
            SngIndProducer { slice: self.slice, offsets: left },
            SngIndProducer { slice: slice_copy, offsets: right },
        )
    }
}

/// Single indirect Sequential iterator over items in a slice
pub(super) struct SngIndSeq<'data, 'offs, T, O: PrimInt> {
    offsets: &'offs [O],
    slice: &'data mut [T],
}

impl <'data, 'offs, T, O> Iterator for SngIndSeq<'data, 'offs, T, O>
where
    O: PrimInt
{
    type Item = &'data mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offsets.len() != 0 {
            let idx = self.offsets[0].to_usize().unwrap();
            let r = unsafe { self.slice.as_mut_ptr().add(idx).as_mut().unwrap() };
            self.offsets = &self.offsets[1..];
            Some(r)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl <'data, 'offs, T, O> ExactSizeIterator for SngIndSeq<'data, 'offs, T, O>
where
    O: PrimInt
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

impl <'data, 'offst, T, O> DoubleEndedIterator for SngIndSeq<'data, 'offst, T, O>
where
    O: PrimInt
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.offsets.len() {
            0 => None,
            n => {
                let idx = self.offsets[n-1].to_usize().unwrap();
                let r = unsafe { self.slice.as_mut_ptr().add(idx).as_mut().unwrap() };
                self.offsets = &self.offsets[..n-1];
                Some(r)
            }
        }
    }
}
