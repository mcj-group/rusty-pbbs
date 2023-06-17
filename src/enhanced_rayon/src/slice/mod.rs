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

mod chunks;
mod chunks_by;
mod sng_ind;
mod sng_ind_by;

use num_traits::PrimInt;

use crate::dedup;
use chunks::ChunksMut;
use chunks_by::ChunksMutBy;
use sng_ind::SngInd;
use sng_ind_by::SngIndBy;


pub trait EnhancedParallelSlice<'data, T: Send> {
    // Ranged Indirection:
    fn par_ind_chunks<'offs, O: PrimInt + Sync>(&self, _offsets: &[O])
    { todo!() }

    fn par_ind_chunks_mut<'offs, O: PrimInt + Sync>(
        &'data mut self,
        _offsets: &'offs [O],
    ) -> ChunksMut<'data, 'offs, T, O>
    { todo!() }

    fn par_ind_chunks_mut_by<F>(
        &'data mut self,
        _offset: F,
        _len: usize
    ) -> ChunksMutBy<'data, T, F>
    where
        F: Fn(usize) -> usize + Send + Sync + Clone
    { todo!() }


    // Single valued Indirection:
    fn par_ind_iter<'offs, O: PrimInt + Sync>(&self, _offsets: &[O])
    { todo!() }

    fn par_ind_iter_mut<'offs, O: PrimInt + Sync>(
        &'data mut self,
        _offsets: &'offs [O]
    ) -> SngInd<'data, 'offs, T, O>
    { todo!() }

    fn par_ind_iter_mut_by<F>(
        &'data mut self,
        _offset: F,
        _len: usize
    ) -> SngIndBy<'data, T, F>
    where
        F: Fn(usize) -> usize + Send + Sync + Clone
    { todo!() }
}

impl<'data, T: Send> EnhancedParallelSlice<'data, T> for [T]
{
    fn par_ind_chunks_mut<'offs, O: PrimInt + Sync>(
        &'data mut self,
        offsets: &'offs [O]
    ) -> ChunksMut<'data, 'offs, T, O>
    { ChunksMut::new(offsets, self) }

    fn par_ind_chunks_mut_by<F>(
        &'data mut self,
        offset: F,
        len: usize
    ) -> ChunksMutBy<'data, T, F>
    where
        F: Fn(usize) -> usize + Send + Sync + Clone
    { ChunksMutBy::new(offset, 0..len, self) }


    fn par_ind_iter_mut<'offs, O: PrimInt + Sync>(
        &'data mut self,
        offsets: &'offs [O]
    ) -> SngInd<'data, 'offs, T, O>
    {
        #[cfg(feature = "sng_ind_safe")]
        dedup::parallel(offsets, self.len());
        unsafe { SngInd::new(self, offsets) }
    }

    fn par_ind_iter_mut_by<F>(
        &'data mut self,
        offset: F,
        len: usize
    ) -> SngIndBy<'data, T, F>
    where
        F: Fn(usize) -> usize + Send + Sync + Clone
    {
        #[cfg(feature = "sng_ind_safe")]
        dedup::parallel_by(offset.clone(), len, self.len());
        unsafe { SngIndBy::new(self, offset, len) }
    }
}
