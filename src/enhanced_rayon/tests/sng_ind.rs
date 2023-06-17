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
fn can_collect() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<u32> = (0..100).collect();

    let r: Vec<usize> = v
        .par_ind_iter_mut(&offs)
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, (0..100).collect::<Vec<usize>>());
}

#[test]
fn can_collect_reverse() {
    let mut v: Vec<u32> = (0..100).collect();
    let offs: Vec<usize> = (0..100).step_by(2).rev().collect();

    let r: Vec<u32> = v
        .par_ind_iter_mut(&offs)
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, (0..100).step_by(2).rev().collect::<Vec<u32>>());
}

#[test]
fn can_offset() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<u32> = vec![0, 85, 35, 13, 76];

    let r: Vec<usize> = v
        .par_ind_iter_mut(&offs)
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, vec![0, 85, 35, 13, 76]);
}

#[test]
fn can_mutate() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<usize> = vec![1, 85, 35, 13, 76];

    v
        .par_ind_iter_mut(&offs)
        .with_gran(1)
        .enumerate()
        .for_each(|(i, vi)| *vi *= i);

    let mut correct: Vec<usize> = (0..100).collect();
    correct[1] *= 0;
    correct[85] *= 1;
    correct[35] *= 2;
    correct[13] *= 3;
    correct[76] *= 4;
    assert_eq!(v, correct);
}

#[test]
#[should_panic]
fn ignore_duplicates() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<usize> = vec![1, 85, 35, 13, 76, 23, 13, 49, 29];

    v
        .par_ind_iter_mut(&offs)
        .with_gran(1)
        .enumerate()
        .for_each(|(i, vi)| *vi *= i);
}


#[test]
fn can_collect_by() {
    let mut v: Vec<usize> = (0..100).collect();

    let r: Vec<usize> = v
        .par_ind_iter_mut_by(|i| i, 100)
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, (0..100).collect::<Vec<usize>>());
}

#[test]
fn can_collect_reverse_by() {
    let mut v: Vec<u32> = (0..100).collect();

    let r: Vec<u32> = v
        .par_ind_iter_mut_by(|i| 98 - 2*i, 50)
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, (0..100).step_by(2).rev().collect::<Vec<u32>>());
}

#[test]
fn can_offset_by() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<usize> = vec![0, 85, 35, 13, 76];

    let r: Vec<usize> = v
        .par_ind_iter_mut_by(|i| offs[i], offs.len())
        .with_gran(1)
        .map(|vi| *vi)
        .collect();
    assert_eq!(r, vec![0, 85, 35, 13, 76]);
}

#[test]
fn can_mutate_by() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<usize> = vec![1, 85, 35, 13, 76];

    v
        .par_ind_iter_mut_by(|i| offs[i], offs.len())
        .with_gran(1)
        .enumerate()
        .for_each(|(i, vi)| *vi *= i);

    let mut correct: Vec<usize> = (0..100).collect();
    correct[1] *= 0;
    correct[85] *= 1;
    correct[35] *= 2;
    correct[13] *= 3;
    correct[76] *= 4;
    assert_eq!(v, correct);
}

#[test]
#[should_panic]
fn ignore_duplicates_by() {
    let mut v: Vec<usize> = (0..100).collect();
    let offs: Vec<usize> = vec![1, 85, 35, 13, 76, 23, 13, 49, 29];

    v
        .par_ind_iter_mut_by(|i| offs[i], offs.len())
        .with_gran(1)
        .enumerate()
        .for_each(|(i, vi)| *vi *= i);
}
