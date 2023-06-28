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


#[allow(dead_code)]
pub(crate) type DefInt = u32;

#[allow(dead_code)]
pub(crate) type DefIntS = i32;

#[allow(dead_code)]
pub(crate) type DefFloat = f32;

#[allow(dead_code)]
pub(crate) type DefChar = u8;

#[allow(dead_code)]
pub(crate) type DefAtomInt = std::sync::atomic::AtomicU32;

#[allow(dead_code)]
pub(crate) type DefAtomIntS = std::sync::atomic::AtomicI32;

#[allow(dead_code)]
pub(crate) static ORDER: std::sync::atomic::Ordering
    = std::sync::atomic::Ordering::Relaxed;
