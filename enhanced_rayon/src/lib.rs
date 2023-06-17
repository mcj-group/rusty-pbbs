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

mod iter;
mod slice;
mod dedup;
pub mod prelude;


// #[cfg(feature = "sng_ind_safe")]
// compile_error!("The sng_ind_safe is enabled!");
// #[cfg(feature = "sng_ind_unsafe")]
// compile_error!("The sng_ind_unsafe is enabled!");

#[cfg(all(feature = "sng_ind_unsafe", feature = "sng_ind_safe"))]
compile_error!("Only one of the following features can be enabled:
    sng_ind_safe, sng_ind_unsafe");

// the unsafe version is the safe version because there are
// no runtime checks that can be easily avoided.
#[cfg(all(feature = "rng_ind_unsafe", feature = "rng_ind_safe"))]
compile_error!("Only one of the following features can be enabled:
    rng_ind_safe, rng_ind_unsafe");
