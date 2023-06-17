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
use std::cmp::{min, max};
use std::sync::atomic::AtomicU32;
use rayon::prelude::*;
use enhanced_rayon::prelude::*;
use parlay::maybe_uninit_vec;
use parlay::utilities::write_min;
use crate::ORDER;



pub struct Reservation(AtomicU32);

impl Reservation {
    const MAX_IDX: u32 = u32::MAX;

    pub fn new() -> Self {
        Self(AtomicU32::new(Self::MAX_IDX))
    }
    
    pub const fn max_idx() -> u32 {
        u32::MAX
    }

    pub fn get(&self) -> u32 {
        self.0.load(ORDER)
    }

    pub fn reserve(&self, i: u32) -> bool {
        write_min(&self.0, i)
    }

    pub fn reserved(&self) -> bool {
        self.0.load(ORDER) < Self::MAX_IDX
    }

    pub fn reset(&self) {
        self.0.store(Self::MAX_IDX, ORDER)
    }

    pub fn check(&self, i: u32) -> bool {
        self.0.load(ORDER) == i
    }

    pub fn check_reset(&self, i: u32) -> bool {
        if self.0.load(ORDER) == i {
            self.0.store(Self::MAX_IDX, ORDER); true
        } else {
            false
        }
    }
}

pub trait SpecFor<T, F> {
    fn spec_for(
        &self,
        reserve: T,
        commit: F,
        granularity: usize,
        rcs: Option<usize>,
        ccs: Option<usize>
    ) -> Result<usize, String> where
        T: Fn(usize) -> bool + Send + Sync,
        F: Fn(usize) -> bool + Send + Sync;
}

impl<T, F> SpecFor<T, F> for Range<usize> {
    fn spec_for(
        &self,
        reserve: T,
        commit: F,
        granularity: usize,
        rcs: Option<usize>,
        ccs: Option<usize>
    ) -> Result<usize, String> where
        T: Fn(usize) -> bool + Send + Sync,
        F: Fn(usize) -> bool + Send + Sync,
    {
        // initialization:
        let (s, e) = (self.start, self.end);
        let _max_tries = 100 + 200 * granularity;
        let max_round_size = (e - s) / granularity + 1;
        let mut current_round_size = max_round_size / 4;
        let (
            mut _round,
            mut number_keep,
            mut total_processed,
            mut number_done
        ) = (0usize, 0usize, 0usize, s);
        let r_chunk_size = rcs.unwrap_or(granularity);
        let c_chunk_size = ccs.unwrap_or(granularity);

        let mut i_hold = Vec::<usize>::new();
        let mut idxs = maybe_uninit_vec![0usize; max_round_size];
        let mut keep = maybe_uninit_vec![false; max_round_size];

        // main loop:
        while number_done < e {
            // if round > max_tries {
            //     return Err("Too many iterations.".to_string());
            // } else { round += 1; }
            let size = min(current_round_size, e - number_done);
            total_processed += size;

            // reserve
            idxs[..size]
                .par_iter_mut()
                .zip(keep[..size].par_iter_mut())
                .with_gran(r_chunk_size)
                .enumerate()
                .for_each(|(i, (ii, ki))| {
                    *ii = if i < number_keep { i_hold[i] }
                        else { number_done + i };
                    *ki = reserve(*ii);
                });

            // commit
            keep[..size]
                .par_iter_mut()
                .with_gran(c_chunk_size)
                .enumerate()
                .for_each(|(i, ki)| {
                    if *ki { *ki = !commit(idxs[i]); }
                });

            // keep iterations that failed for next round
            parlay::primitives::pack(
                &idxs[..size],
                &keep[..size],
                &mut i_hold
            );
            number_keep = i_hold.len();
            number_done = number_done + size - number_keep;

            // adjust round size based on the number of failed attempts
            if (number_keep as f32 / size as f32) > 0.2 {
                current_round_size = max(
                    current_round_size / 2,
                    max(max_round_size/64 + 1, number_keep)
                );
            } else if (number_keep as f32 / size as f32) < 0.1 {
                current_round_size = min(
                    current_round_size * 2,
                    max_round_size
                );
            }
        }
        Ok(total_processed)
    }
}

pub trait StatefulSpecFor<T, F, S> {
    fn stateful_spec_for(
        &self,
        reserve: T,
        commit: F,
        state: S,
        granularity: usize,
        rcs: Option<usize>,
        ccs: Option<usize>
    ) -> Result<usize, String>
    where
        T: Fn(usize, &mut S) -> bool + Send + Sync,
        F: Fn(usize, &mut S) -> bool + Send + Sync,
        S: Clone + Send + Sync;
}

impl<T, F, S> StatefulSpecFor<T, F, S> for Range<usize> {
    fn stateful_spec_for(
        &self,
        reserve: T,
        commit: F,
        _st: S,
        granularity: usize,
        rcs: Option<usize>,
        ccs: Option<usize>
    ) -> Result<usize, String>
    where
        T: Fn(usize, &mut S)->bool + Send + Sync,
        F: Fn(usize, &mut S)->bool + Send + Sync,
        S: Clone + Send + Sync,
    {
        // initialization:
        let (s, e) = (self.start, self.end);
        let max_tries = 100 + 200 * granularity;
        let max_round_size = (e-s) / granularity + 1;
        let mut current_round_size = max_round_size / 4;
        let (
            mut round,
            mut number_keep,
            mut total_processed,
            mut number_done
        ) = (0usize, 0usize, 0usize, s);
        let r_chunk_size = rcs.unwrap_or(granularity);
        let c_chunk_size = ccs.unwrap_or(granularity);

        let mut i_hold = Vec::<usize>::new();
        let mut idxs = maybe_uninit_vec![0usize; max_round_size];
        let mut keep = maybe_uninit_vec![false; max_round_size];
        let mut state = maybe_uninit_vec![_st; max_round_size];

        // main loop:
        while number_done < e {
            if round > max_tries {
                return Err("too many iterations.".to_string());
            } else { round += 1; }
            let size = min(current_round_size, e - number_done);
            total_processed += size;

            // reserve
            (
                &mut idxs[..size],
                &mut keep[..size],
                &mut state[..size]
            )
                .into_par_iter()
                .with_gran(r_chunk_size)
                .enumerate()
                .for_each(
                    |(i, (ii, ki, si))| {
                        *ii = if i < number_keep { i_hold[i] }
                            else { number_done + i };
                        *ki = reserve(*ii, si);
                    });

            // commit
            (
                &mut keep[..size],
                &mut state[..size]
            )
                .into_par_iter()
                .with_min_len(c_chunk_size)
                .with_max_len(c_chunk_size)
                .enumerate()
                .for_each(
                    |(i, (ki, si))| {
                        if *ki { *ki = !commit(idxs[i], si); }
                    });

            // keep iterations that failed for next round
            parlay::primitives::pack(
                &idxs[..size],
                &keep[..size],
                &mut i_hold
            );
            number_keep = i_hold.len();
            number_done = number_done + size - number_keep;

            // adjust round size based on number of failed attempts
            if (number_keep as f32 / size as f32) > 0.2 {
                current_round_size = max(
                    current_round_size / 2,
                    max(max_round_size/64 + 1, number_keep)
                );
            } else if (number_keep as f32 / size as f32) < 0.1 {
                current_round_size = min(
                    current_round_size * 2,
                    max_round_size
                );
            }
        }

        Ok(total_processed)
    }
}
