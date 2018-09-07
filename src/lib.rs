#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]

extern crate rand;
extern crate test;
extern crate byteorder;
extern crate jobsteal;
extern crate num_cpus;

pub mod voxel;
pub mod field;

pub mod tfce;
pub use self::tfce::tfce;
pub use self::tfce::approximate_tfce;

pub mod prob_bin_search;
pub use self::prob_bin_search::probabilistic_binary_search;

pub mod ttest;
pub mod permutation;
mod voxel_priority;

pub mod fiff;
pub mod freesurfer;
pub mod stc;

use voxel::Voxel;
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};

pub fn explore_tfce_permutation(
    a: &Vec<Vec<f64>>,
    b: &Vec<Vec<f64>>,
    n: i32,
    voxels: &mut Vec<Voxel>,
    e_min: f64, e_step: f64, e_max: f64,
    h_min: f64, h_step: f64, h_max: f64,
    negative: bool,
    positive: bool
) {

    let n_cpu = num_cpus::get();
    eprintln!("using {} threads to explore TFCE parameters", n_cpu);
    let mut pool = jobsteal::make_pool(n_cpu).unwrap();

    pool.scope(|scope| {
        let mut e = e_min;
        while e <= e_max {

            let mut h = h_min;
            while h <= h_max {

                let mut voxels = voxels.clone();

                scope.submit(move || {
                    let result = permutation::significant_indices(
                        &permutation::run_permutation(
                            a, b, n,
                            &mut |ap, bp| {
                                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&ap, &bp).into_iter()) {
                                    if negative {
                                        v.value = tv.min(0.0).abs();
                                    } else if positive {
                                        v.value = tv.max(0.0);
                                    } else {
                                        v.value = tv.abs();
                                    }
                                }
                                tfce(&mut voxels, e, h);
                                voxels.iter().map(|v| v.tfce_value).collect()
                            }
                        )
                    );

                    if result.len() > 0 {
                        println!("e = {:.4}, h = {:.4}, {:?} significant voxels", e, h, result.len());
                    }
                });

                h += h_step;
            }

            e += e_step;
        }

    });
}

pub fn read_data_file(filename: String) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut a = Vec::new();
    let mut b = Vec::new();

    let mut file = File::open(&filename).expect(&format!("failed to open input file {}", &filename));
    let subject_count = file.read_u32::<LittleEndian>().unwrap();
    for _ in 0..subject_count {
        let sa_len = file.read_u32::<LittleEndian>().unwrap();
        let mut sa = Vec::new();
        for _ in 0..sa_len {
            sa.push(file.read_f64::<LittleEndian>().unwrap());
        }
        a.push(sa);

        let sb_len = file.read_u32::<LittleEndian>().unwrap();
        let mut sb = Vec::new();
        for _ in 0..sb_len {
            sb.push(file.read_f64::<LittleEndian>().unwrap());
        }
        b.push(sb);
    }

    (a, b)
}
