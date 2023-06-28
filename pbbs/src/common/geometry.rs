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

use std::ops::*;
use std::str::FromStr;
use std::fmt::{Debug, Display, LowerExp};
use num_traits::Float;

use crate::common::io::fmt_f64;


const PI: f64 = 3.14159;
pub trait PointToVec { type Vec; }

// *************************************************************
//    POINTS AND VECTORS (2d)
// *************************************************************

#[derive(Copy, Clone)]
pub struct Vector2d<T: Float> { pub x: T, pub y: T }

impl<T: Float> Vector2d<T> {
    pub fn new(x: T, y:T) -> Self { Self { x, y } }
    pub fn length(&self) -> T { (self.x*self.x + self.y*self.y).sqrt() }
    pub fn dot(&self, v: Vector2d<T>) -> T { self.x*v.x + self.y*v.y }
    pub fn cross(&self, v: Vector2d<T>) -> T { self.x*v.y - self.y*v.x }
}

impl<T: Float> Mul<T> for Vector2d<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Self {x: self.x * other, y: self.y * other}
    }
}

impl<T: Float> Div<T> for Vector2d<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self { Vector2d::new(self.x/rhs, self.y/rhs) }
}

impl<T: Float> Default for Vector2d<T> {
    fn default() -> Self { Self { x:T::zero(), y:T::zero() } }
}


#[derive(Copy, Clone)]
pub struct Point2d<T: Float> { pub x: T, pub y: T }

impl<T: Float> PointToVec for Point2d<T> { type Vec = Vector2d<T>; }

impl<T: Float> Point2d<T> {
    pub fn new(x: T, y: T) -> Self { Self { x, y } }
}

impl<T: Float> Default for Point2d<T> {
    fn default() -> Self { Self { x:T::zero(), y:T::zero() } }
}

impl<T: Float> Add<Vector2d<T>> for Point2d<T> {
    type Output = Self;
    fn add(self, other: Vector2d<T>) -> Self {
        Self {x: self.x+other.x, y: self.y+other.y}
    }
}

impl<T: Float> Sub for Point2d<T> {
    type Output = Vector2d<T>;
    fn sub(self, other: Self) -> Vector2d<T> {
        Vector2d {x: self.x-other.x, y: self.y-other.y}
    }
}

impl<T: Float + FromStr> FromStr for Point2d<T>
where
    <T as std::str::FromStr>::Err: Debug
{
    type Err = ParsePoint2dError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.trim().split_whitespace().collect();
        if s.len() != 2 { return Err(ParsePoint2dError); }
        let (a, b) = (s[0].parse(), s[1].parse());
        if a.is_err() || b.is_err() { return Err(ParsePoint2dError); }
        Ok(Self::new(a.unwrap(), b.unwrap()))
    }
}

impl<T: Float + Display + LowerExp> Display for Point2d<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            fmt_f64(self.x.to_f64().unwrap(), 11, 2),
            fmt_f64(self.y.to_f64().unwrap(), 11, 2)
        )
    }
}

pub struct ParsePoint2dError;

impl Display for ParsePoint2dError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not parse as Point2d.")
    }
}

impl Debug for ParsePoint2dError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{{ file: {}, line: {} }}: can not parse as Point2d.",
            file!(),
            line!()
        )
    }
}

//    POINTS AND VECTORS (2d)
// *************************************************************



// *************************************************************
//    POINTS AND VECTORS (3d)
// *************************************************************

pub type Vector3d<T> = Point3d<T>;

#[derive(Copy, Clone)]
pub struct Point3d<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Float> PointToVec for Point3d<T> { type Vec = Vector3d<T>; }

impl<T: Float> Point3d<T> {
    pub fn new(x: T, y: T, z: T) -> Self { Self { x, y, z } }

    // Returns the vector result of the cross product
    pub fn cross(self, other:Self) -> Self {
        Self{
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }

    // Returns the scalar result of the dot product
    pub fn dot(self, other: Self) -> T {
        return self.x * other.x + self.y * other.y + self.z * other.z;
    }
}

impl<T: Float> Add<Vector3d<T>> for Point3d<T> {
    type Output = Self;

    fn add(self, other: Vector3d<T>) -> Self {
        Self {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

impl<T: Float> Sub for Point3d<T> {
    type Output = Vector3d<T>;

    fn sub(self, other: Self) -> Self {
        Self {x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
    }
}

impl<T: Float> Mul<T> for Vector3d<T> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        Self {x: self.x * other, y: self.y * other, z: self.z * other}
    }
}

impl<T: Float + FromStr> FromStr for Point3d<T>
where
    <T as std::str::FromStr>::Err: Debug
{
    type Err = ParsePoint3dError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<&str> = s.trim().split_whitespace().collect();
        if s.len() != 3 { return Err(ParsePoint3dError); }
        let (a, b, c) = (s[0].parse(), s[1].parse(), s[2].parse());
        if a.is_err() || b.is_err() || c.is_err() {
            Err(ParsePoint3dError)
        } else {
            Ok(Self::new(a.unwrap(), b.unwrap(), c.unwrap()))
        }
    }
}

pub struct ParsePoint3dError;

impl Display for ParsePoint3dError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Can not parse as Point3d.")
    }
}

impl Debug for ParsePoint3dError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{{ file: {}, line: {} }}: can not parse as Point3d.",
            file!(),
            line!()
        )
    }
}

impl<T: Float + Display> Display for Point3d<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

//    POINTS AND VECTORS (3d)
// *************************************************************



// *************************************************************
//    TRIANGLES AND RAYS
// *************************************************************

pub type Tri = [i32; 3];

#[derive(Clone)]
pub struct Triangles<P> { pub p: Vec<P>, pub t: Vec<Tri> }

impl<P> Triangles<P> {
    pub fn new(p: Vec<P>, t: Vec<Tri>) -> Self { Self { p, t } }
    pub fn num_points(&self) -> usize { self.p.len() }
    pub fn num_triangles(&self) -> usize { self.t.len() }
}

#[derive(Copy, Clone)]
pub struct Ray<P> where P: PointToVec {
    pub o: P,
    pub d: P::Vec,
}

impl<P> Ray<P> where P: PointToVec {
    pub fn new(o: P, d: P::Vec) -> Self { Self { o, d } }
}

#[inline(always)]
pub fn angle<T: Float>(a: Point2d<T>, b: Point2d<T>, c: Point2d<T>) -> T {
    let (ba, ca) = (b-a, c-a);
    let (lba, lca) = (ba.length(), ca.length());
    T::from(
        180.0 / PI * (ba.dot(ca) / (lba*lca)).to_f64().unwrap().acos()
    ).unwrap()
}

#[inline(always)]
pub fn min_angle_check<T: Float>(
    a: Point2d<T>,
    b: Point2d<T>,
    c: Point2d<T>,
    angle: T
) -> bool {
    let (ba, ca, cb) = (b-a, c-a, c-b);
    let (lba, lca, lcb) = (ba.length(), ca.length(), cb.length());
    let co = T::from((angle.to_f64().unwrap() * PI / 180.0).cos()).unwrap();
    
    ba.dot(ca) / (lba * lca) > co
        || ca.dot(cb) / (lca * lcb) > co
        || -ba.dot(cb) / (lba * lcb) > co
}

#[inline(always)]
pub fn triangle_circumcenter<T: Float>(
    a: Point2d<T>,
    b: Point2d<T>,
    c: Point2d<T>
) -> Point2d<T> {
    let (v1, v2) = (b-a, c-a);
    let (v11, v22) = (v1 * v2.dot(v2), v2 * v1.dot(v1));
    
    a + Vector2d::new(v22.y - v11.y, v11.x - v22.x)
        / (T::from(2.0).unwrap() * v1.cross(v2))
}

//    TRIANGLES AND RAYS
// *************************************************************


// *************************************************************
//    GEOMETRY
// *************************************************************

#[inline(always)]
pub fn tri_area<T: Float>(a: Point2d<T>, b: Point2d<T>, c: Point2d<T>) -> T {
    (b - a).cross(c - a)
}

#[inline(always)]
pub fn counter_clock_wise<T: Float>(
    a: Point2d<T>,
    b: Point2d<T>,
    c: Point2d<T>
) -> bool {
    (b - a).cross(c - a) > T::zero()
}

#[inline(always)]
pub fn on_parabola<T: Float>(v: Vector2d<T>) -> Vector3d<T> {
    Vector3d::new(v.x, v.y, v.x * v.x + v.y * v.y)
}

#[inline(always)]
pub fn in_circle<T: Float>(
    a: Point2d<T>,
    b: Point2d<T>,
    c: Point2d<T>,
    d: Point2d<T>
) -> bool {
    let ad = on_parabola(a - d);
    let bd = on_parabola(b - d);
    let cd = on_parabola(c - d);
    (ad.cross(bd)).dot(cd) > T::zero()
}
