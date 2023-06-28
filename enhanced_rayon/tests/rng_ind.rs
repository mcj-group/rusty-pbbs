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


mod iter {
    use super::*;
    #[test]
    fn one_chunk() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind(&[0])
            .collect();
        assert_eq!(v, vec![(0..100).collect::<Vec<usize>>()]);
    }

    #[test]
    fn empty_chunks() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind(&[0, 0, 0, 0, 0])
            .collect();
        assert_eq!(v, vec![
            vec![], vec![], vec![], vec![],
            (0..100).collect::<Vec<usize>>()
            ]);
    }

    #[test]
    fn five_chunks() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind(&[0, 15, 70, 80])
            .collect();
        assert_eq!(v,
            vec![
                (0..15).collect::<Vec<usize>>(),
                (15..70).collect::<Vec<usize>>(),
                (70..80).collect::<Vec<usize>>(),
                (80..100).collect::<Vec<usize>>(),
            ]);
    }

    #[test]
    fn can_map() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind(&[0, 15, 70, 70])
            .enumerate()
            .map(|(i, chunk)| {
                chunk.into_iter().map(|ci| ci * i).collect()
            }).collect();
        assert_eq!(v,
            vec![
                vec![0; 15],
                (15..70).collect::<Vec<usize>>(),
                vec![],
                (210..300).step_by(3).collect::<Vec<usize>>(),
            ]
        );
    }

    #[test]
    fn can_mutate() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind(&[0, 15, 70, 70])
            .enumerate()
            .for_each(|(i, chunk)| {
                for ci in chunk { *ci *= i; }
            });
        assert_eq!(v,
            (vec![
                vec![0; 15],
                (15..70).collect::<Vec<usize>>(),
                vec![],
                (210..300).step_by(3).collect::<Vec<usize>>(),
            ]).into_iter().flatten().collect::<Vec<usize>>()
        );
    }

    #[test]
    #[should_panic]
    fn can_offset_back() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind(&[0, 15, 70, 60])
            .enumerate()
            .for_each(|_| {});
    }

    #[test]
    #[should_panic]
    fn can_overflow() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind(&[0, 15, 70, 120])
            .enumerate()
            .for_each(|_| {});
    }




    #[test]
    fn one_chunk_by() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind_by(|_i| 0, 1)
            .collect();
        assert_eq!(v, vec![(0..100).collect::<Vec<usize>>()]);
    }

    #[test]
    fn empty_chunks_by() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind_by(|_i| 0, 5)
            .collect();
        assert_eq!(v, vec![
            vec![], vec![], vec![], vec![],
            (0..100).collect::<Vec<usize>>()
            ]);
    }

    #[test]
    fn five_chunks_by() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind_by(|i| i*i, 5)
            .collect();
        assert_eq!(v,
            vec![
                vec![0],
                (1..4).collect::<Vec<usize>>(),
                (4..9).collect::<Vec<usize>>(),
                (9..16).collect::<Vec<usize>>(),
                (16..100).collect::<Vec<usize>>()
            ]);
    }

    #[test]
    fn can_map_by() {
        let v: Vec<Vec<usize>> = (0..100)
            .into_par_iter()
            .with_gran(1)
            .rng_ind_by(|i| i*i, 5)
            .enumerate()
            .map(|(i, chunk)| {
                chunk.into_iter().map(|ci| ci * i).collect()
            }).collect();
        assert_eq!(v,
            vec![
                vec![0],
                (1*1..4*1).step_by(1).collect::<Vec<usize>>(),
                (4*2..9*2).step_by(2).collect::<Vec<usize>>(),
                (9*3..16*3).step_by(3).collect::<Vec<usize>>(),
                (16*4..100*4).step_by(4).collect::<Vec<usize>>()
            ]
        );
    }

    #[test]
    fn can_mutate_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind_by(|i| i*i, 6)
            .enumerate()
            .for_each(|(i, chunk)| {
                for ci in chunk { *ci *= i; }
            });
        assert_eq!(v,
            (vec![
                vec![0],
                (1*1..4*1).step_by(1).collect::<Vec<usize>>(),
                (4*2..9*2).step_by(2).collect::<Vec<usize>>(),
                (9*3..16*3).step_by(3).collect::<Vec<usize>>(),
                (16*4..25*4).step_by(4).collect::<Vec<usize>>(),
                (25*5..100*5).step_by(5).collect::<Vec<usize>>(),
            ]).into_iter().flatten().collect::<Vec<usize>>()
        );
    }

    #[test]
    #[should_panic]
    fn can_offset_back_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind_by(|i| { if i==3 { 80 } else { i*10 } }, 5)
            .enumerate()
            .for_each(|_| {});
    }

    #[test]
    #[should_panic]
    fn can_overflow_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_iter_mut()
            .with_gran(1)
            .rng_ind_by(|i| i * 20, 10)
            .enumerate()
            .for_each(|_| {});
    }
}



mod slice {
    use super::*;
    #[test]
    fn one_chunk() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offs = vec![0];
        v
            .par_ind_chunks_mut(&offs)
            .with_gran(1)
            .for_each(|v| v.iter_mut().for_each(|vi| *vi = 1));
        assert_eq!(v, vec![1; 100]);
    }

    #[test]
    fn empty_chunks() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offs = vec![0, 0, 0, 0, 0];
        v
            .par_ind_chunks_mut(&offs)
            .with_gran(1)
            .enumerate()
            .for_each(|(i, v)| v.iter_mut().for_each(|vi| *vi = i));
        assert_eq!(v, vec![4; 100]);
    }

    #[test]
    fn five_chunks() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offs = vec![0, 15, 70, 80];
        v
            .par_ind_chunks_mut(&offs)
            .with_gran(1)
            .enumerate()
            .for_each(|(i, v)| v.iter_mut().for_each(|vi| *vi = i));
        assert_eq!(v,
            vec![
                vec![0; 15],
                vec![1; 55],
                vec![2; 10],
                vec![3; 20],
            ].into_iter().flatten().collect::<Vec<usize>>());
    }

    #[test]
    #[should_panic]
    fn can_offset_back() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_ind_chunks_mut(&[0, 15, 70, 60])
            .with_gran(1)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic]
    fn can_overflow() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_ind_chunks_mut(&[0, 15, 70, 120])
            .with_gran(1)
            .for_each(|_| {});
    }


    #[test]
    fn one_chunk_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_ind_chunks_mut_by(|_| 0, 1)
            .with_gran(1)
            .for_each(|v| v.iter_mut().for_each(|vi| *vi = 1));
        assert_eq!(v, vec![1; 100]);
    }

    #[test]
    fn one_sub_chunk_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_ind_chunks_mut_by(|_| 20, 1)
            .with_gran(1)
            .for_each(|v| {
                println!("{:?}", v.len());
                v.iter_mut().for_each(|vi| *vi = 1);
            });
        assert_eq!(
            v,
            (0..20)
                .into_iter()
                .chain((20..100).into_iter().map(|_| 1))
                .collect::<Vec<_>>()
            );
    }

    #[test]
    fn empty_chunks_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        v
            .par_ind_chunks_mut_by(|_| 0, 5)
            .with_gran(1)
            .enumerate()
            .for_each(|(i, v)| v.iter_mut().for_each(|vi| *vi = i));
        assert_eq!(v, vec![4; 100]);
    }

    #[test]
    fn five_chunks_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offs = vec![0, 15, 70, 80];
        v
            .par_ind_chunks_mut_by(|i| offs[i], 4)
            .with_gran(1)
            .enumerate()
            .for_each(|(i, v)| v.iter_mut().for_each(|vi| *vi = i));
        assert_eq!(v,
            vec![
                vec![0; 15],
                vec![1; 55],
                vec![2; 10],
                vec![3; 20],
            ].into_iter().flatten().collect::<Vec<usize>>());
    }

    #[test]
    #[should_panic]
    fn can_offset_back_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offsets = vec![0, 15, 70, 60];
        v
            .par_ind_chunks_mut_by(|i| offsets[i], 4)
            .with_gran(1)
            .for_each(|_| {});
    }

    #[test]
    #[should_panic]
    fn can_overflow_by() {
        let mut v = (0..100).collect::<Vec<usize>>();
        let offsets = vec![0, 15, 70, 120];
        v
            .par_ind_chunks_mut_by(|i| offsets[i], 4)
            .with_gran(1)
            .for_each(|_| {});
    }
}
