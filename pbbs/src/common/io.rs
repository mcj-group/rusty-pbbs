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

use std::{fs, io, io::prelude::*};
use rayon::prelude::*;

#[allow(dead_code)]
#[inline(always)]
pub(crate) fn fmt_f64(num: f64, precision: usize, exp_pad: usize) -> String {
    let mut num = format!("{:.precision$e}", num, precision = precision);
    let exp = num.split_off(num.find('e').unwrap());

    let (sign, exp) = if exp.starts_with("e-") {
        ('-', &exp[2..])
    } else {
        ('+', &exp[1..])
    };
    
    num.push_str(&format!("e{}{:0>pad$}", sign, exp, pad = exp_pad));
    num
}

#[allow(dead_code)]
pub(crate) fn write_slice_to_file_seq<T, F>(s: &[T], of: F)
where
    T: std::string::ToString,
    F: AsRef<std::path::Path>,
{
    let s: Vec<String> = s
        .into_iter()
        .map(T::to_string)
        .collect();
    fs::write(
        of,
        s.join("\n")
    ).expect("cannot write to output");
}

#[allow(dead_code)]
pub(crate) fn read_file_to_vec_seq<T, P>(fname: P) -> Vec<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err : std::fmt::Debug,
    P: AsRef<std::path::Path>
{
    let s = fs::read_to_string(fname)
        .expect("cannot read input file");
    let w: Vec<_> = s.split('\n').collect();
    w
        .into_iter()
        .map(str::parse)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect::<Vec<T>>()
}

#[allow(dead_code)]
pub(crate) fn read_file_to_vec<T, P, F>(
    fname: P,
    debug_assert: Option<F>
) -> Vec<T> where
    T: std::str::FromStr + Send,
    <T as std::str::FromStr>::Err : std::fmt::Debug + Send,
    P: AsRef<std::path::Path>,
    F: Fn(&[&str]),
{
    let s = fs::read_to_string(fname)
        .expect("cannot read input file");
    let w: Vec<_> = s.par_split('\n').collect();
    if debug_assert.is_some() {
        debug_assert.unwrap()(&w);
    }
    w
        .into_par_iter()
        .map(str::parse)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect::<Vec<T>>()
}

#[allow(dead_code)]
pub(crate) fn read_big_file_to_vec<T, P, F>(
    fname: P,
    debug_assert: Option<F>,
    dest: &mut Vec<T>
) where
    T: std::str::FromStr + Send,
    <T as std::str::FromStr>::Err : std::fmt::Debug + Send,
    P: AsRef<std::path::Path>,
    F: Fn(&[&str]),
{
    if debug_assert.is_some() {
        eprintln!("debug_assert is not supported for read_big_file_to_vec");
    }
    *dest = fs::read_to_string(fname)
        .expect("cannot read input file")
        .par_split('\n')
        .map(str::parse)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect::<Vec<T>>();
}

#[allow(dead_code)]
pub(crate) fn chars_from_file<P: AsRef<std::path::Path>>(
    fname: P,
    null_terminate: bool
) -> io::Result<Vec<u8>>
{
    let mut f = fs::File::open(fname)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    if null_terminate { buffer.push(0); }
    Ok(buffer)
}

#[allow(dead_code)]
pub(crate) fn chars_to_file<P: AsRef<std::path::Path>>(
    buffer: &[u8],
    fname: P
) -> io::Result<()>
{
    let mut f = fs::File::create(fname)?;
    f.write_all(buffer)?;
    Ok(())
}
