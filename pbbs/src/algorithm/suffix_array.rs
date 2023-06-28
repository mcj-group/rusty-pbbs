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

use std::slice::from_raw_parts_mut;

use rayon::prelude::*;
use enhanced_rayon::prelude::*;

use crate::{DefInt, DefChar, DefAtomInt, ORDER};
use parlay::internal::sequence_ops::scan_inplace;
use parlay::{Timer, maybe_uninit_vec};


#[derive(Clone, Copy)]
struct Seg { s: DefInt, l: DefInt }


#[allow(dead_code)]
fn split_segment(
    seg_out: &mut [Seg],
    start: DefInt,
    ranks: &mut [DefInt],
    cs: &[(DefInt, DefInt)]
) {
    let l = seg_out.len();
    if l < 5000 { // sequential version
        let mut name = 0;
        ranks[cs[0].1 as usize] = name + start + 1;
        for i in 1..l {
            if cs[i-1].0 != cs[i].0 { name = i as DefInt; }
            ranks[cs[i].1 as usize] = name + start + 1;
        }
        name = 0;
        for i in 1..l {
            if cs[i-1].0 != cs[i].0 {
                seg_out[i-1] = Seg { s: name+start, l: i as DefInt-name };
                name = i as DefInt;
            } else { seg_out[i-1] = Seg { s: 0, l: 0 } }
        }
        seg_out[l-1] = Seg { s: name+start, l: l as DefInt-name };
    } else {
        let mut names = (0..1)
            .into_par_iter()
            .chain(
                (1..l)
                    .into_par_iter()
                    .map(|i| if cs[i-1].0 != cs[i].0 { i as DefInt } else { 0 })
            ).collect::<Vec<_>>();

        scan_inplace( &mut names, true, |a, b| a.max(b) );
        
        ranks
            .par_ind_iter_mut_by(|i| cs[i].1 as usize, l)
            .enumerate()
            .for_each(|(i, rank)| *rank = names[i] + start + 1);

        seg_out[..l-1]
            .par_iter_mut()
            .zip((1..l).into_par_iter())
            .for_each(|(s, i)| {
                if names[i] == i as DefInt {
                    *s = Seg { s: names[i-1] + start, l: i as DefInt - names[i-1] };
                } else { *s = Seg { s: 0, l: 0 }; }
            });
        seg_out[l-1] = Seg { s: names[l-1] + start, l: l as DefInt - names[l-1] };
    }
}

#[allow(dead_code)]
fn split_segment_top(
    seg_out: &mut [Seg],
    ranks: &mut [DefInt],
    cs: &[u128]
) -> Vec<(DefInt, DefInt)>
{
    let n = seg_out.len();
    let mask: u128 = (1 << 32) - 1;

    // mark start of each segment with equal keys
    let mut names: Vec<_> = (0..1).into_par_iter().chain((1..n)
        .into_par_iter()
        .map(|i|
            if cs[i] >> 32 != cs[i-1] >> 32 { i as DefInt } else { 0 }
        )).collect();

    // scan start i across each segment
    scan_inplace(&mut names, true, |a, b| a.max(b));

    // write new rank into original location
    let c: Vec<_> = ranks
        .par_ind_iter_mut_by(|i| (cs[i] & mask) as usize, n)
        .zip(names.par_iter())
        .zip(cs.par_iter())
        .map(|((r, n), c)| {
            *r = *n + 1;
            (0, (*c & mask) as DefInt)
        }).collect();

    // get starts and lengths of new segments
    seg_out[..n-1]
        .par_iter_mut()
        .zip((1..n).into_par_iter())
        .for_each(|(s, i)| {
        if names[i] == i as DefInt {
            *s = Seg { s: names[i-1], l: i as DefInt - names[i-1] }
        } else { *s = Seg { s: 0, l: 0 } };
    });
    seg_out[n-1] = Seg { s: names[n-1], l: n as DefInt - names[n-1] };

    c
}

#[allow(dead_code)]
pub fn suffix_array(ss: &[DefChar], ranks: &mut [DefInt]) {
    let mut t = Timer::new("sa"); //t.start();
    let n = ss.len();

    // renumber characters densely
    // start numbering at 1 leaving 0 to indicate end-of-string
    let pad = 48;
    let mut flags = vec![DefInt::default(); 256];
    #[cfg(feature = "AW_safe")]
    {
        let atom_flags = (0..256)
            .map(|_| DefAtomInt::new(0))
            .collect::<Vec<_>>();
        let f = |i| {
            if atom_flags[ss[i] as usize].load(ORDER)==0 {
                atom_flags[ss[i] as usize].store(1, ORDER);
            }
        };
        (0..n).into_par_iter().with_gran(1024).for_each(f);
        flags.iter_mut().enumerate().for_each(
            |(i, f)| *f = atom_flags[i].load(ORDER)
        );
    }
    #[cfg(not(feature = "AW_safe"))]
    {
        let flags_ptr = flags.as_ptr() as usize;
        let f = |i| {
            if flags[ss[i] as usize]==0 {
                unsafe { (flags_ptr as *mut DefInt).add(ss[i] as usize).write(1); }
            }
        };
        (0..n).into_par_iter().with_gran(1024).for_each(f);
    }

    let m = scan_inplace(&mut flags, false, |a, b| a + b);


    // pad the end of string with 0s
    let s: Vec<_> = (0..n+pad)
        .into_par_iter()
        .map(|i| if i < n { flags[ss[i] as usize] as u8 } else { 0 })
        .collect();

    // pack characters into 128-bit word, along with the location i
    // 96 bits for characters, and 32 for location
    let logm = (m as f64).log2();
    let nchars = (96.0/logm).floor() as DefInt;
    let mut cl: Vec<_> = (0..n)
            .into_par_iter()
            .map(|i| {
            let mut r = s[i] as u128;
            for j in 1..nchars {
                r = r * (m as u128) + (s[i+j as usize] as u128);
            }
            (r << 32) + i as u128
        }).collect();
    t.next("copy into 128bit int");

    // sort based on packed words
    cl.par_sort_unstable();
    t.next("sort");

    // identify segments of equal values
    let mut seg_outs = maybe_uninit_vec![Seg { s: 0, l: 0 }; n];
    let mut c = split_segment_top(&mut seg_outs, ranks, &cl);
    cl.clear();
    t.next("split top");

    let mut round = 0;
    let mut n_keys = n;
    let mut offset = nchars;

    loop {
        if round > 40 { panic!("SA: too many rounds!"); }
        else { round += 1; }

        let is_seg = |s: &Seg| s.l > 1;
        // only keep segments that are longer than 1 (otherwise already sorted)
        let segs: Vec<_> = seg_outs[..n_keys]
            .par_iter()
            .cloned()
            .filter(is_seg)
            .collect();
        let n_segs = segs.len();
        if n_segs == 0 { break; }
        t.next("filter and scan");

        let mut offsets: Vec<DefInt> = maybe_uninit_vec![DefInt::default(); n_segs];

        #[cfg(feature = "rng_ind_safe")]
        {
            c
                .par_ind_chunks_mut_by(
                    |i| segs[i].s as usize,
                    n_segs
                ).zip(offsets.par_iter_mut())
                .enumerate()
                .for_each(|(i, (ci, o))| {
                    let l = segs[i].l;
                    *o = l;
                    let ci = &mut ci[..l as usize];
                    ci
                        .par_iter_mut()
                        .with_gran(128)
                        .for_each(|c| {
                            let o = (c.1 + offset) as usize;
                            c.0 = if o >= n { 0 } else { ranks[o] };
                        });

                        ci.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                });
        }
        #[cfg(not(feature = "rng_ind_safe"))]
        {
            (0..n_segs)
                .into_par_iter()
                .for_each(|i| {
                    let start = segs[i].s as usize;
                    let l = segs[i].l as usize;
                    let ci = unsafe { from_raw_parts_mut(
                        (c.as_ptr() as *mut (DefInt, DefInt)).add(start),
                        l
                    )};
                    unsafe {
                        (offsets.as_ptr() as *mut DefInt)
                        .add(i)
                        .write(l as DefInt);
                    }

                    // grab rank from offset locations ahead
                    ci
                        .par_iter_mut()
                        .with_gran(128)
                        .for_each(|cj| {
                            let o = (cj.1 + offset) as usize;
                            cj.0 = if o >= n { 0 } else { ranks[o] };
                        });

                    ci.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                });
        }
        t.next("sort");

        // starting offset for each segment
        n_keys = scan_inplace(&mut offsets, false, |a, b| a+b) as usize;

        // Split each segment into subsegments if neighbors differ.
        #[cfg(feature = "rng_ind_safe")]
        {
            seg_outs
                .par_ind_chunks_mut(&offsets)
                .zip(segs.par_iter())
                .for_each(|(seg_out, seg)| {
                    let start = seg.s as usize;
                    let l = seg.l as usize;
                    split_segment(
                        &mut seg_out[..l],
                        start as DefInt,
                        unsafe {from_raw_parts_mut(
                            ranks.as_ptr() as *mut _,
                            ranks.len())},
                        &c[start..start+l]
                    );
                });
        }
        #[cfg(not(feature = "rng_ind_safe"))]
        {
            let f = |i: usize| {
                let start = segs[i].s as usize;
                let l = segs[i].l as usize;
                let o = offsets[i] as usize;
                unsafe {
                    split_segment(
                        from_raw_parts_mut((seg_outs.as_ptr() as *mut Seg).add(o),l),
                        start as DefInt,
                        from_raw_parts_mut(ranks.as_ptr() as *mut _, ranks.len()),
                        &c[start..start+l]);
                    }
            };
            (0..n_segs).into_par_iter().with_gran(128).for_each(f);
        }
        t.next("split");

        offset *= 2;
    }

    ranks
        .par_iter_mut()
        .zip(c.par_iter())
        .for_each(|(q, ci)| *q = ci.1);

    t.next("rank update");
}


#[allow(dead_code)]
fn atomic_split_segment(
    seg_out: &mut [Seg],
    start: DefInt,
    ranks: &[DefAtomInt],
    cs: &[(DefInt, DefInt)]
) {
    let l = seg_out.len();
    if l < 5000 { // sequential version
        let mut name = 0;
        ranks[cs[0].1 as usize].store(name + start + 1, ORDER);
        for i in 1..l {
            if cs[i-1].0 != cs[i].0 { name = i as DefInt; }
            ranks[cs[i].1 as usize].store(name + start + 1, ORDER);
        }
        name = 0;
        for i in 1..l {
            if cs[i-1].0 != cs[i].0 {
                seg_out[i-1] = Seg { s: name+start, l: i as DefInt-name };
                name = i as DefInt;
            } else { seg_out[i-1] = Seg { s: 0, l: 0 } }
        }
        seg_out[l-1] = Seg { s: name+start, l: l as DefInt-name };
    } else {
        let mut names = (0..1)
            .into_par_iter()
            .chain(
                (1..l)
                    .into_par_iter()
                    .map(|i| if cs[i-1].0 != cs[i].0 { i as DefInt } else { 0 })
            ).collect::<Vec<_>>();

        scan_inplace( &mut names, true, |a, b| a.max(b) );
        
        (0..l)
            .into_par_iter()
            .for_each(|i| {
                ranks[cs[i].1 as usize].store(names[i] + start + 1, ORDER);
            });

        seg_out[..l-1]
            .par_iter_mut()
            .zip((1..l).into_par_iter())
            .for_each(|(s, i)| {
                if names[i] == i as DefInt {
                    *s = Seg { s: names[i-1] + start, l: i as DefInt - names[i-1] };
                } else { *s = Seg { s: 0, l: 0 }; }
            });
        seg_out[l-1] = Seg { s: names[l-1] + start, l: l as DefInt - names[l-1] };
    }
}

#[allow(dead_code)]
fn atomic_split_segment_top(
    seg_out: &mut [Seg],
    ranks: &[DefAtomInt],
    cs: &[u128]
) -> Vec<(DefInt, DefInt)>
{
    let n = seg_out.len();
    let mask: u128 = (1 << 32) - 1;

    // mark start of each segment with equal keys
    let mut names: Vec<_> = (0..1).into_par_iter().chain((1..n)
        .into_par_iter()
        .map(|i|
            if cs[i] >> 32 != cs[i-1] >> 32 { i as DefInt } else { 0 }
        )).collect();

    // scan start i across each segment
    scan_inplace(&mut names, true, |a, b| a.max(b));

    // write new rank into original location
    let c: Vec<_> = names
        .par_iter()
        .zip(cs.par_iter())
        .map(|(&n, &c)| {
            ranks[(c & mask) as usize].store(n + 1, ORDER);
            (0, (c & mask) as DefInt)
        }).collect();

    // get starts and lengths of new segments
    seg_out[..n-1]
        .par_iter_mut()
        .zip((1..n).into_par_iter())
        .for_each(|(s, i)| {
        if names[i] == i as DefInt {
            *s = Seg { s: names[i-1], l: i as DefInt - names[i-1] }
        } else { *s = Seg { s: 0, l: 0 } };
    });
    seg_out[n-1] = Seg { s: names[n-1], l: n as DefInt - names[n-1] };

    c
}

#[allow(dead_code)]
pub fn atomic_suffix_array(ss: &[DefChar], ranks: &[DefAtomInt]) {
    let mut t = Timer::new("sa"); //t.start();
    let n = ss.len();

    // renumber characters densely
    // start numbering at 1 leaving 0 to indicate end-of-string
    let pad = 48;
    let mut flags = vec![DefInt::default(); 256];
    #[cfg(feature = "AW_safe")]
    {
        let atom_flags = (0..256)
            .map(|_| DefAtomInt::new(0))
            .collect::<Vec<_>>();
        let f = |i| {
            if atom_flags[ss[i] as usize].load(ORDER)==0 {
                atom_flags[ss[i] as usize].store(1, ORDER);
            }
        };
        (0..n).into_par_iter().with_gran(1024).for_each(f);
        flags.iter_mut().enumerate().for_each(
            |(i, f)| *f = atom_flags[i].load(ORDER)
        );
    }
    #[cfg(not(feature = "AW_safe"))]
    {
        let flags_ptr = flags.as_ptr() as usize;
        let f = |i| {
            if flags[ss[i] as usize]==0 {
                unsafe { (flags_ptr as *mut DefInt).add(ss[i] as usize).write(1); }
            }
        };
        (0..n).into_par_iter().with_gran(1024).for_each(f);
    }

    let m = scan_inplace(&mut flags, false, |a, b| a + b);


    // pad the end of string with 0s
    let s: Vec<_> = (0..n+pad)
        .into_par_iter()
        .map(|i| if i < n { flags[ss[i] as usize] as u8 } else { 0 })
        .collect();

    // pack characters into 128-bit word, along with the location i
    // 96 bits for characters, and 32 for location
    let logm = (m as f64).log2();
    let nchars = (96.0/logm).floor() as DefInt;
    let mut cl: Vec<_> = (0..n)
            .into_par_iter()
            .map(|i| {
            let mut r = s[i] as u128;
            for j in 1..nchars {
                r = r * (m as u128) + (s[i+j as usize] as u128);
            }
            (r << 32) + i as u128
        }).collect();
    t.next("copy into 128bit int");

    // sort based on packed words
    cl.par_sort_unstable();
    t.next("sort");

    // identify segments of equal values
    let mut seg_outs = maybe_uninit_vec![Seg { s: 0, l: 0 }; n];
    let mut c = atomic_split_segment_top(&mut seg_outs, ranks, &cl);
    cl.clear();
    t.next("split top");

    let mut round = 0;
    let mut n_keys = n;
    let mut offset = nchars;

    loop {
        if round > 40 { panic!("SA: too many rounds!"); }
        else { round += 1; }

        let is_seg = |s: &Seg| s.l > 1;
        // only keep segments that are longer than 1 (otherwise already sorted)
        let segs: Vec<_> = seg_outs[..n_keys]
            .par_iter()
            .cloned()
            .filter(is_seg)
            .collect();
        let n_segs = segs.len();
        if n_segs == 0 { break; }
        t.next("filter and scan");

        let mut offsets: Vec<DefInt> = maybe_uninit_vec![DefInt::default(); n_segs];

        #[cfg(feature = "rng_ind_safe")]
        {
            c
                .par_ind_chunks_mut_by(
                    |i| segs[i].s as usize,
                    n_segs
                ).zip(offsets.par_iter_mut())
                .enumerate()
                .for_each(|(i, (ci, o))| {
                    let l = segs[i].l;
                    *o = l;
                    let ci = &mut ci[..l as usize];
                    ci
                        .par_iter_mut()
                        .with_gran(128)
                        .for_each(|c| {
                            let o = (c.1 + offset) as usize;
                            c.0 = if o >= n { 0 } else { ranks[o].load(ORDER) };
                        });

                        ci.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                });
        }
        #[cfg(not(feature = "rng_ind_safe"))]
        {
            (0..n_segs)
                .into_par_iter()
                .for_each(|i| {
                    let start = segs[i].s as usize;
                    let l = segs[i].l as usize;
                    let ci = unsafe { from_raw_parts_mut(
                        (c.as_ptr() as *mut (DefInt, DefInt)).add(start),
                        l
                    )};
                    unsafe {
                        (offsets.as_ptr() as *mut DefInt)
                        .add(i)
                        .write(l as DefInt);
                    }

                    // grab rank from offset locations ahead
                    ci
                        .par_iter_mut()
                        .with_gran(128)
                        .for_each(|cj| {
                            let o = (cj.1 + offset) as usize;
                            cj.0 = if o >= n { 0 } else { ranks[o].load(ORDER) };
                        });

                    ci.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                });
        }
        t.next("sort");

        // starting offset for each segment
        n_keys = scan_inplace(&mut offsets, false, |a, b| a+b) as usize;

        // Split each segment into subsegments if neighbors differ.
        #[cfg(feature = "rng_ind_safe")]
        {
            seg_outs
                .par_ind_chunks_mut(&offsets)
                .zip(segs.par_iter())
                .for_each(|(seg_out, seg)| {
                    let start = seg.s as usize;
                    let l = seg.l as usize;
                    atomic_split_segment(
                        &mut seg_out[..l],
                        start as DefInt,
                        ranks,
                        &c[start..start+l]
                    );
                });
        }
        #[cfg(not(feature = "rng_ind_safe"))]
        {
            let f = |i: usize| {
                let start = segs[i].s as usize;
                let l = segs[i].l as usize;
                let o = offsets[i] as usize;
                unsafe {
                    atomic_split_segment(
                        from_raw_parts_mut(
                            (seg_outs.as_ptr() as *mut Seg).add(o),
                            l
                        ),
                        start as DefInt,
                        ranks,
                        &c[start..start+l]);
                    }
            };
            (0..n_segs).into_par_iter().with_gran(128).for_each(f);
        }
        t.next("split");

        offset *= 2;
    }

    ranks
        .par_iter()
        .zip(c.par_iter())
        .for_each(|(q, ci)| q.store(ci.1, ORDER));

    t.next("rank update");
}
