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
use rayon::prelude::*;
use rayon::iter::{MaxLen, MinLen};

mod chunks;
mod chunks_by;

use chunks::Chunks;
use chunks_by::ChunksBy;


/// This trait will add support for sng_ind and rng_ind irregular patterns
/// to all indexed parallel iterators.
/// `with_gran` conviently sets min and max granularity.
pub trait EnhancedParallelIterator: IndexedParallelIterator {
    fn with_gran(self, size: usize) -> MaxLen<MinLen<Self>>{
        self.with_min_len(size).with_max_len(size)
    }

    fn sng_ind<OTy>(self, _offsets: &[OTy]) { todo!() }
    fn sng_ind_by<OTy>(self, _off_key: OTy) { todo!() }

    fn rng_ind_by<F>(self, offset: F, len: usize) -> ChunksBy<Self, F>
    where
        F: Fn(usize) -> usize + Send + Clone
    {
        ChunksBy::new(self, offset, len)
    }

    fn rng_ind<'offs, OTy>(
        self,
        offsets: &'offs [OTy]
    ) -> Chunks<'offs, Self, OTy>
    where
        OTy: PrimInt + Sync,
    {
        Chunks::new(self, offsets)
    }
}

impl<T: IndexedParallelIterator> EnhancedParallelIterator for T {}
