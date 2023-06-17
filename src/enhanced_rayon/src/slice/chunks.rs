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

use std::marker::PhantomData;
use rayon::iter::plumbing::*;
use rayon::iter::*;
use num_traits::PrimInt;


/// Parallel iterator over mutable non-overlapping chunks of a slice
#[derive(Debug)]
pub struct ChunksMut<'data, 'offs, T: Send, O: PrimInt> {
    offsets: &'offs [O],
    slice: &'data mut [T],
}

impl<'data, 'offs, T: Send, O: PrimInt> ChunksMut<'data, 'offs, T, O> {
    pub(super) fn new(offsets: &'offs [O], slice: &'data mut [T]) -> Self {
        Self { offsets, slice }
    }
}

impl<'data, 'offs, T, O> ParallelIterator for ChunksMut<'data, 'offs, T, O>
where
    T: Send + 'data,
    O: PrimInt + Sync,
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

impl<'data, 'offs, T, O> IndexedParallelIterator for ChunksMut<'data, 'offs, T, O>
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
        callback.callback(ChunksMutProducer {
            offsets: self.offsets,
            slice: self.slice,
        })
    }
}

struct ChunksMutProducer<'data, 'offs, T: Send, O> {
    offsets: &'offs [O],
    slice: &'data mut [T],
}

impl<'data, 'offs, T, O> Producer for ChunksMutProducer<'data, 'offs, T, O>
where
    O: PrimInt + Sync,
    T: 'data + Send,
{
    type Item = &'data mut [T];
    type IntoIter = ChunkSeqMut<'data, 'offs, T, O>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkSeqMut {
            offsets: self.offsets,
            ptr: self.slice,
            _marker: PhantomData,
        }
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let elem_index =
            (self.offsets[index] - self.offsets[0]).to_usize().unwrap();
        let (left, right) = self.slice.split_at_mut(elem_index);
        (
            ChunksMutProducer {
                offsets: &self.offsets[..index],
                slice: left,
            },
            ChunksMutProducer {
                offsets: &self.offsets[index..],
                slice: right,
            },
        )
    }
}


pub(super) struct ChunkSeqMut<'data, 'offs, T: 'data, O: PrimInt> {
    offsets: &'offs [O],
    ptr: *mut [T],
    _marker: PhantomData<&'data mut T>
}

impl <'data, 'offs, T, O: PrimInt> Iterator for ChunkSeqMut<'data, 'offs, T, O> {
    type Item = &'data mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        match self.offsets.len() {
            0 => None,
            1 => {
                self.offsets = &self.offsets[1..];
                Some(unsafe { &mut *self.ptr })
            },
            _ => {
                let size = (self.offsets[1] - self.offsets[0]).to_usize().unwrap();
                self.offsets = &self.offsets[1..];
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

impl <'data, 'offs, T, O> ExactSizeIterator for ChunkSeqMut<'data, 'offs, T, O>
where
    O: PrimInt
{
    fn len(&self) -> usize {
        self.offsets.len()
    }
}

impl <'data, 'offs, T, O> DoubleEndedIterator for ChunkSeqMut<'data, 'offs, T, O>
where
    O: PrimInt
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.offsets.len() {
            0 => None,
            1 => {
                self.offsets = &self.offsets[1..];
                Some(unsafe { &mut *self.ptr })
            },
            n => {
                let skip = (
                    self.offsets[n - 1] - self.offsets[0]
                ).to_usize().unwrap();
                self.offsets = &self.offsets[..self.offsets.len() - 1];
                let (left, right) = unsafe { (*self.ptr).split_at_mut(skip) };
                self.ptr = left as *mut [T];
                Some(right)
            }
        }
    }
}

