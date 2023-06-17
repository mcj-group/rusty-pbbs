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

const BIN_SEARCH_BASE: usize = 16;


fn linear_search<T, F>(inp: &[T], p: T, less: F) -> usize
where
    T: Copy,
    F: Fn(T, T) -> bool,
{
    inp
        .iter()
        .position(|&x| !less(x, p))
        .unwrap_or(inp.len())
}

pub fn binary_search<T, F>(inp: &[T], p: T, less: F) -> usize
where
    T: Copy,
    F: Fn(T, T) -> bool,
{
    let (mut start, mut end) = (0, inp.len());

    while end - start > BIN_SEARCH_BASE {
        let mid = (start + end) / 2;
        if !less(inp[mid], p) { end = mid; }
        else { start = mid + 1; }
    }

    start + linear_search(&inp[start..end], p, less)
}
