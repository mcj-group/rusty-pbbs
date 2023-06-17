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

use std::time::{ Instant, Duration };

/// A timer that can be used to time regions of code.
pub struct Timer<'a> {
    total_so_far: Duration,
    last: Instant,
    on: bool,
    name: &'a str,
}

impl<'a> Timer<'a> {
    /// prints `d` in seconds
    pub fn report(&self, d: Duration, name: &str) {
        if name.is_empty() {
            println!("{}:\t{:.6}", self.name, d.as_secs_f64());
        } else {
            println!("{}:{}:\t{:.6}", self.name, name, d.as_secs_f64());
        }
    }

    /// Creates a new timer with the given name.
    pub fn new(name: &'a str) -> Self {
        Timer {
            total_so_far: Duration::ZERO,
            last: Instant::now(),
            on: false,
            name,
        }
    }

    /// Starts the timer.
    pub fn start(&mut self) {
        self.on = true;
        self.last = Instant::now();
    }

    /// Stops the timer and returns the time since the last `start` or `next`.
    pub fn stop(&mut self) -> Duration {
        self.on = false;
        let d = Instant::now() - self.last;
        self.total_so_far += d;
        d
    }

    /// Resets and turns off the timer.
    pub fn reset(&mut self) {
        self.total_so_far = Duration::ZERO;
        self.on = false;
    }

    /// Returns the time since the last `start` or `next`.
    pub fn next_time(&mut self) -> Duration {
        if !self.on {
            return Duration::ZERO;
        }
        let t = Instant::now();
        let td = t - self.last;
        self.total_so_far += td;
        self.last = t;
        td
    }

    /// Returns the total time when timer was on since the last `new` or `reset`.
    pub fn total_time(&self) -> Duration {
        if self.on {
            self.total_so_far + (Instant::now() - self.last)
        } else {
            self.total_so_far
        }
    }

    /// Prints the time since the last `start` or `next`.
    pub fn next(&mut self, name: &'a str) {
        let nt = self.next_time();
        if self.on {
            self.report(nt, name);
        }
    }

    /// Prints the total time when timer was on since the last `new` or `reset`.
    pub fn total(&self) {
        let tt = self.total_time();
        self.report(tt, "total");
    }
}
