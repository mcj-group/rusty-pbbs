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

use std::sync::atomic::{AtomicU32, Ordering};
use num_traits::PrimInt;


/// returns the smallest power of two greater than or equal to i.
pub fn log2_up<T: PrimInt>(i: T) -> usize {
    debug_assert!(i > T::zero());
    let mut a = 0;
    let mut b = i - T::one();
    while b > T::zero() {
        b = b.shr(1);
        a += 1;
    }
    a
}

/// calculates a hash of u based on numerical recipes.
#[inline(always)]
pub fn hash64(u: u64) -> u64 {
    let mut v = u.overflowing_mul(3_935_559_000_370_003_845).0;
    v = v.overflowing_add(2_691_343_689_449_507_681).0;
    v ^= v >> 21;
    v ^= v << 37;
    v ^= v >> 4;
    v = v.overflowing_mul(4_768_777_513_237_032_717).0;
    v ^= v << 20;
    v ^= v >> 41;
    v ^= v << 5;
    v
}

/// calculates a hash of x that is cheaper than `hash64` based on splitmix64.
#[inline(always)]
pub fn hash64_cheap(mut x: u64) -> u64
{
    x = (x ^ (x >> 30)).overflowing_mul(0xbf58476d1ce4e5b9).0;
    x = (x ^ (x >> 27)).overflowing_mul(0x94d049bb133111eb).0;
    x = x ^ (x >> 31);
    x
}

/// tries to write b to a atomically while b is smaller than a.
/// returns true if successful and false otherwise.
#[inline(always)]
pub fn write_min(a: &AtomicU32, b: u32) -> bool {
    let mut c = a.load(Ordering::Relaxed);
    while b < c {
        match a.compare_exchange_weak(
            c,
            b,
            Ordering::Relaxed,
            Ordering::Relaxed
        ) {
            Ok(_) => { return true; },
            Err(new) => c = new,
        }
    }
    false
}