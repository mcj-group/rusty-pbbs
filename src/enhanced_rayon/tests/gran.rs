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
use enhanced_rayon::prelude::*;


#[test]
fn ones() {
    let v: Vec<usize> = (0..1000)
        .into_par_iter()
        .with_gran(10)
        .map(|_| 1)
        .collect();
    assert_eq!(v, vec![1; 1000]);
}


#[test]
fn evens() {
    let v: Vec<usize> = (0..1000)
        .into_par_iter()
        .with_gran(100)
        .map(|a| a*2)
        .collect();
    let expected: Vec<usize> = (0..2000)
        .into_par_iter()
        .step_by(2)
        .collect();
    assert_eq!(v, expected);
}


#[test]
fn delayed() {
    let v: Vec<usize> = (0..1000)
        .into_par_iter()
        .map(|a| a*2)
        .with_gran(100)
        .map(|a| a)
        .collect();
    let expected: Vec<usize> = (0..2000)
        .into_par_iter()
        .step_by(2)
        .collect();
    assert_eq!(v, expected);
}