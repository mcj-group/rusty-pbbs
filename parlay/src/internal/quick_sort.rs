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

const SERIAL_QS_TR: usize = 1 << 8;


fn base_case<T>(inp: &[T]) -> bool {
    let n = inp.len();
    let large = size_of::<T>() > 8;
    if large { n < 16 } else { n < 24 }
}

/// Simple serial insertion sort
pub(crate) fn insertion_sort<T, F>(inp: &mut [T], less: F)
where
    T: Copy,
    F: Fn(T, T) -> bool,
{
    for i in 1..inp.len() {
        let mut j = i;
        while j > 0 && less(inp[j], inp[j-1]) {
            inp.swap(j, j-1);
            j-=1;
        }
    }
}

/// sorts 5 elements taken at even stride and puts them at the front
fn sort5<T, F>(inp: &mut [T], less: F)
where
    T: Copy,
    F: Fn(T, T) -> bool,
{
    let size = 5;
    let m = inp.len() / (size + 1);
    for l in 0..size { inp.swap(l, m * (l+1)); }
    insertion_sort(&mut inp[..size], less);
}

/// Dual-pivot partition. Picks two pivots from the input A
/// and then divides it into three parts:
///   [x < p1), [p1 <= x <= p2], (p2 < x]
fn split3<T, F>(inp: &mut [T], less: F) -> (usize, usize, bool)
where
    T: Copy,
    F: Fn(T, T) -> bool + Copy,
{
    let n = inp.len();
    sort5(inp, less);
    let _inp_ptr = inp.as_ptr() as usize;

    // Use A[1] and A[3] as the pivots. Move them to
    // the front so that A[0] and A[1] are the pivots
    inp.swap(0, 1); inp.swap(1, 3);
    let (p1, p2) = (inp[0], inp[1]);
    let pivots_equal = !less(p1, p2);

    // set up initial invariants
    let mut li = 2;
    let mut ri = n - 1;
    while less(inp[li], p1) { li+=1 };
    while less(p2, inp[ri]) { ri-=1 };
    let mut mi = li;

    // invariants:
    //  below li is less than p1,
    //  above ri is greater than p2
    //  between li and mi are between p1 and p2 inclusive
    //  between mi and ri are unprocessed
    while mi <= ri {
        if less(inp[mi], p1) {
            inp.swap(mi, li);
            li+=1;
        } else if less(p2, inp[mi]) {
            inp.swap(mi, ri);
            if less(inp[mi], p1) {
                inp.swap(li, mi);
                li+=1;
            }
            ri-=1;
            while less(p2, inp[ri]) { ri-=1; }
        }
        mi+=1;
    }

    // Swap the pivots into position
    li-=2;
    inp.swap(1, li+1); inp.swap(0, li); inp.swap(li+1, ri);
    (li, mi, pivots_equal)
}

/// Serial quick sort
pub fn quick_sort_serial<T, F>(inp: &mut [T], less: F)
where
    T: Copy,
    F: Fn(T, T) -> bool + Copy,
{
    let mut n = inp.len();
    while !base_case(&inp[..n]) {
        let (l, m, mid_eq) = split3(&mut inp[..n], less);
        if !mid_eq {
            quick_sort_serial(&mut inp[l+1..m], less)
        };
        quick_sort_serial(&mut inp[m..n], less);
        n = l;
    }

    insertion_sort(inp, less);
}

/// Parallel quick sort
pub fn quick_sort<T, F>(inp: &mut [T], less: F)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Copy + Send + Sync,
{
    // serial sort for small inputs
    if inp.len() < SERIAL_QS_TR {
        quick_sort_serial(inp, less);
    } else {  // parallel sort for large enough inputs
        let (l, m, mid_eq) = split3(inp, less);

        let (l_inp, t1) = inp.split_at_mut(l);
        let (t2, r_inp) = t1.split_at_mut(m-l);
        let m_inp = &mut t2[1..];

        let left = || quick_sort(l_inp, less);
        let mid = || quick_sort(m_inp, less);
        let right = || quick_sort(r_inp, less);

        if mid_eq {
            rayon::join(left, right);
        } else {
            let left_mid = || rayon::join(left, mid);
            rayon::join(left_mid, right);
        }
    }
}
