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

#[macro_export]
#[cfg(not(memSafe))]
macro_rules! maybe_uninit_vec {
    ($elem:expr; $n:expr) => {
        {
            let mut v = Vec::with_capacity($n);
            unsafe { v.set_len($n); }
            v
        }
    };

    ($($tokens:tt)*) => { vec!($($tokens)*) };
}

#[macro_export]
#[cfg(memSafe)]
macro_rules! maybe_uninit_vec { ($($tokens:tt)*) => { vec!($($tokens)*) }; }






#[macro_export]
macro_rules! uget {
    ($ptr: expr, $offset: expr, $t: ty)
        => {*(($ptr + $offset * std::mem::size_of::<$t>()) as *const $t)};
    ($ptr: expr, $t: ty)
        => {*($ptr as *const $t)};
}

#[macro_export]
macro_rules! uget_mut {
    ($ptr: expr, $offset: expr, $t: ty)
        => {*(($ptr + $offset * std::mem::size_of::<$t>()) as *mut $t)};
    ($ptr: expr, $t: ty)
        => {*($ptr as *mut $t)};
}

#[macro_export]
macro_rules! make_mut {
    ($immut_ref: expr, $t: ty)
        => {($immut_ref as *const $t as usize as *mut $t).as_mut()};
}

#[macro_export]
macro_rules! uget_slice {
    ($arr: expr, $start: expr, $end: expr, $t: ty)
        => {std::slice::from_raw_parts(
            ($arr.as_ptr() as usize + $start * std::mem::size_of::<$t>()) as *const $t,
            $end - $start)
        };
    ($arr: expr, $len: expr, $t: ty)
        => {std::slice::from_raw_parts(
            $arr.as_ptr() as usize as *const $t,
            $len)
        };
    ($arr: expr, $t: ty)
        => {std::slice::from_raw_parts(
            $arr.as_ptr() as usize as *const $t,
            $arr.len())
        };
}

#[macro_export]
macro_rules! uget_slice_mut {
    ($arr: expr, $start: expr, $end: expr, $t: ty)
        => {std::slice::from_raw_parts_mut(
            ($arr.as_ptr() as usize + $start * std::mem::size_of::<$t>()) as *mut $t,
            $end - $start)
        };
    ($arr: expr, $len: expr, $t: ty)
        => {std::slice::from_raw_parts_mut(
            ($arr.as_ptr() as usize) as *mut $t,
            $len)
        };
    ($arr: expr, $t: ty)
        => {std::slice::from_raw_parts_mut(
            $arr.as_ptr() as usize as *mut $t,
            $arr.len())
        };
}

#[macro_export]
macro_rules! swap {
    ($base: expr, $a: expr, $b: expr, $t: ty)
        => {
            let x = ($base + $a * std::mem::size_of::<$t>()) as *mut T;
            let y = ($base + $b * std::mem::size_of::<$t>()) as *mut T;
            // std::ptr::swap(x, y);
            let z = *x;
            *x = *y;
            *y = z;
        };
}

#[macro_export]
macro_rules! uninit_vec {
    ($arr: expr, $n: expr, $t: ty, $default: expr)
        => {
            $arr = Vec::<$t>::with_capacity($n);
            $arr.set_len($n);
        };
    ($arr: expr, $n: expr, $default: expr)
        => {
            $arr = Vec::with_capacity($n);
            $arr.set_len($n);
        };
    ($arr: expr, $n: expr)
    => {
        $arr = Vec::with_capacity($n);
        $arr.set_len($n);
    };
}

#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*)
        => {
            #[cfg(pbbsVerbose)]
            println!($($arg)*);
        };
}
