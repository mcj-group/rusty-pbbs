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

use std::mem::size_of;
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering::SeqCst};

macro_rules! cast {
    ($a: expr, $o: expr, $n: expr, $t: ty) => {
        ($a as *mut $t, *($o as *const $t), *($n as *const $t))
    };
}

#[inline(always)]
pub fn atomic_cas<T>(a: &mut T, old: T, new: T) -> bool {
    let sz = size_of::<T>();
    let a_ptr = a as *mut T;
    let o_ptr = &old as *const T;
    let n_ptr = &new as *const T;
    debug_assert!(sz <= 8);
    unsafe {
        match sz {
            1 => {
                let (a_cast, o, n) = cast!(a_ptr, o_ptr, n_ptr, u8);
                (*(a_cast as *const AtomicU8))
                    .compare_exchange(o, n, SeqCst, SeqCst).is_ok()
            },
            4 => {
                let (a_cast, o, n) = cast!(a_ptr, o_ptr, n_ptr, u32);
                (*(a_cast as *const AtomicU32))
                    .compare_exchange(o, n, SeqCst, SeqCst).is_ok()
            },
            8 => {
                let (a_cast, o, n) = cast!(a_ptr, o_ptr, n_ptr, u64);
                (*(a_cast as *const AtomicU64))
                    .compare_exchange(o, n, SeqCst, SeqCst).is_ok()
            },
            _ => { panic!("atomic_cas: not yet implemented for this type!") }
        }
    }
}

pub fn write_max_i32(a: &mut i32, b: i32) -> bool
{
    loop {
        let c = *a;
        if c >= b { return false; }
        else if atomic_cas(a, c, b) {
            return true;
        }
    }
}
