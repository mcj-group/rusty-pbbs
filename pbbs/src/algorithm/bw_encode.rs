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

mod suffix_array;

use crate::{DefChar, DefInt};
use suffix_array::suffix_array;
use parlay::maybe_uninit_vec;


pub fn bw_encode(s: &[DefChar]) -> Vec<DefChar> {
    let n = s.len();

    let ss: Vec<DefChar> = (0..1)
        .into_par_iter()
        .chain(s.par_iter().cloned())
        .collect();

    let mut sa: Vec<DefInt> = maybe_uninit_vec![0; n+1];
    suffix_array(&ss, &mut sa);

    (0..n+1).into_par_iter().map(|i| {
        let j = sa[i];
        if j==0 { ss[n] } else { ss[j as usize - 1] }
    }).collect()
}
