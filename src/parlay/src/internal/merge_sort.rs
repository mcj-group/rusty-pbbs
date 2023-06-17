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

use crate::internal::merge::merge_into;
use crate::maybe_uninit_vec;

use crate::internal::quick_sort::insertion_sort;

const MERGE_SORT_BASE: usize = 48;

pub(crate) fn merge_sort_<T, F>(
    inp: &mut [T],
    out: &mut [T],
    less: F,
    inplace: bool
) where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Clone + Send,
{
    let n = inp.len();
    if n < MERGE_SORT_BASE {
        insertion_sort(inp, less);
        if !inplace { out.iter_mut().zip(inp.iter()).for_each(|(o, i)| *o = *i); }
    } else {
        let m = n / 2;
        let (l_inp, r_inp) = inp.split_at_mut(m);
        let (l_out, r_out) = out.split_at_mut(m);
        let (cmp_clone_1, cmp_clone_2) = (less.clone(), less.clone());
        let l = || merge_sort_(l_inp, l_out, cmp_clone_1, !inplace);
        let r = || merge_sort_(r_inp, r_out, cmp_clone_2, !inplace);
        if n > 64 { rayon::join(l, r); }
        else { l(); r(); }

        if inplace {
            merge_into(&out[0..m], &out[m..n], inp, less);
        } else {
            merge_into(&inp[0..m], &inp[m..n], out, less);
        }
    }
}

pub fn merge_sort_inplace<T, F>(inp: &mut [T], less: F)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Clone + Send,
{
    let n = inp.len();
    if n < MERGE_SORT_BASE { insertion_sort(inp, less);}
    else {
        let mut out = maybe_uninit_vec![inp[0]; n];
        merge_sort_(inp, &mut out, less, true);
    }
}

pub fn merge_sort<T, F>(inp: &mut [T], out: &mut [T], less: F)
where
    T: Copy + Send + Sync,
    F: Fn(T, T) -> bool + Clone + Send,
{
    merge_sort_(inp, out, less, false);
}
