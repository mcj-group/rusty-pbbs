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

use std::fs;
use std::fmt::Debug;
use std::str::FromStr;
use num_traits::Float;
use rayon::prelude::*;
use rayon::str::ParallelString;

use crate::common::io::read_file_to_vec;
use crate::common::geometry::*;


const HEADER_TRI: &str = "pbbs_triangles";

pub fn read_points2d_from_file<T>(fname: &str) -> Vec<Point2d<T>>
where
    T: Float + FromStr + Send,
    <T as std::str::FromStr>::Err: Debug
{
    read_file_to_vec(
        fname,
        Some {0: |w: &[&str]| debug_assert_eq!(w[0], "pbbs_sequencePoint2d")}
    )
}

pub fn read_points3d_from_file<T>(fname: &str) -> Vec<Point3d<T>>
where
    T: Float + FromStr + Send,
    <T as std::str::FromStr>::Err: Debug
{
    read_file_to_vec(
        fname,
        Some {0: |w: &[&str]| debug_assert_eq!(w[0], "pbbs_sequencePoint3d")}
    )
}

pub fn read_triangles_from_file<P>(fname: &str, offset: usize) -> Triangles<P>
where
    P: FromStr + Send,
    <P as std::str::FromStr>::Err: Debug + Send
{
    let w = fs::read_to_string(fname)
        .expect("cannot read input triangles's file");
    let mut w: Vec<&str> = w.trim().par_split('\n').collect();

    // Parse header: string
    assert_eq!(w.remove(0), HEADER_TRI);

    // Parse header: n and m
    let n: usize = w.remove(0).parse().unwrap();
    let m: usize = w.remove(0).parse().unwrap();
    debug_assert_eq!(w.len(), n + m);

    // Parse points and triangles
    let pnts: Vec<P> = w[..n]
        .into_par_iter()
        .cloned()
        .map(str::parse)
        .filter(Result::is_ok)
        .map(Result::unwrap)
        .collect();

    let offset = offset as i32;
    let tris: Vec<Tri> = w[n..]
        .into_par_iter()
        .cloned()
        .map( |s: &str| {
            let s: Vec<&str> = s.trim().split_whitespace().collect();
            [
                s[0].parse::<i32>().unwrap() - offset,
                s[1].parse::<i32>().unwrap() - offset,
                s[2].parse::<i32>().unwrap() - offset
            ]
        }).collect();
    debug_assert_eq!(pnts.len(), n);
    debug_assert_eq!(tris.len(), m);

    Triangles::new(pnts, tris)
}

pub fn write_triangles_to_file<P, F>(tris: &Triangles<P>, fname: F)
where
    P: ToString + Sync,
    F: AsRef<std::path::Path>,
{
    let (n, m) = (tris.num_points(), tris.num_triangles());
    let ps: Vec<_> = tris.p
        .par_iter()
        .map(|p| p.to_string())
        .collect();
    let ts: Vec<_> = tris.t
        .par_iter()
        .map(|t| format!("{} {} {}", t[0], t[1], t[2]) )
        .collect();
    fs::write(
        fname,
        format!(
            "{}\n{}\n{}\n{}\n{}",
            HEADER_TRI, n, m, ps.join("\n"),
            ts.join("\n")
        )
    ).expect("cannot write to output");
}
