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
    k_min: f64, k_step: f64, k_max: f64,
    e_min: f64, e_step: f64, e_max: f64
) {

    let n_cpu = num_cpus::get();
    eprintln!("using {} threads to explore TFCE parameters", n_cpu);
    let mut pool = jobsteal::make_pool(n_cpu).unwrap();

    pool.scope(|scope| {
        let mut k = k_min;
        while k <= k_max {

            let mut e = e_min;
            while e <= e_max {

                let mut voxels = voxels.clone();

                scope.submit(move || {
                    let result = permutation::get_periods(permutation::significant_indices(
                        &permutation::run_permutation(
                            a, b, n,
                            &mut |ap, bp| {
                                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&ap, &bp).into_iter()) {
                                    v.value = tv.abs();
                                }
                                tfce(&mut voxels, k, e);
                                voxels.iter().map(|v| v.tfce_value).collect()
                            }
                        )
                    ));

                    if result.len() > 0 {
                        println!("k = {:.4}, e = {:.4}, {:?}", k, e, result);
                    }
                });

                e += e_step;
            }

            k += k_step;
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
