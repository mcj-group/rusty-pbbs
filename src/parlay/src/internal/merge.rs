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

use rayon::prelude::*;

use crate::internal::binary_search::binary_search;

const MERGE_BASE: usize = 2000;


pub(crate) fn seq_merge<T, F>(in1: &[T], in2: &[T], out: &mut [T], less: F)
where
    T: Copy,
    F: Fn(T, T) -> bool,
{
    let (n1, n2) = (in1.len(), in2.len());
    debug_assert!(n1 + n2 <= out.len());
    let (mut i, mut j) = (0, 0);

    loop {
        if i == n1 { // if inp1 has no more elements
            for j in j..n2 { out[i+j] = in2[j]; }
            break;
        }
        if j == n2 { // if inp2 has no more elements
            for i in i..n1 { out[i+j] = in1[i]; }
            break;
        }

        let oi = &mut out[i+j];
        if less(in2[j], in1[i]) {
            *oi = in2[j];
            j += 1;
        } else {
            *oi = in1[i];
            i+=1;
        }
    }
}

pub(crate) fn merge_into<T, F>(in1: &[T], in2: &[T], out: &mut [T], less: F)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Clone + Send,
{
    let (n1, n2) = (in1.len(), in2.len());
    let no = n1 + n2;
    debug_assert_eq!(no, out.len());

    if no < MERGE_BASE {
        seq_merge(in1, in2, out, less);
    }
    else if n1 == 0 {
        out.par_iter_mut().zip(in2.par_iter()).for_each(|(o, i)| *o = *i); }
    else if n2 == 0 {
        out.par_iter_mut().zip(in1.par_iter()).for_each(|(o, i)| *o = *i); }
    else {
        let mut m1 = n1 / 2;
        let m2 = binary_search(in2, in1[m1], less.clone());
        if m2 == 0 { m1 += 1; }
        let mo = m1 + m2;
        let (l_out, r_out) = out.split_at_mut(mo);
        let less_clone = less.clone();
        rayon::join(
            || merge_into(&in1[0..m1], &in2[0..m2], l_out, less_clone),
            || merge_into(&in1[m1..n1], &in2[m2..n2], r_out, less),
        );
    }
}
