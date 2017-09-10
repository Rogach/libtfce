extern crate libtfce;

use libtfce::field::generate_1d_field;
use libtfce::tfce;
use libtfce::ttest;
use std::env;

fn main() {
    let data_file = env::args().nth(1).expect("expected input filename as first argument");

    let n = 1000;
    let mut voxels = generate_1d_field(n);

    let mut a = Vec::new();
    let mut b = Vec::new();

    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(data_file).expect("failed to open input file");
    let mut buf = [0u8; 8];
    for _ in 0..27 {
        let mut sa = Vec::new();
        for _ in 0..n {
            file.read(&mut buf).unwrap();
            sa.push(unsafe { std::mem::transmute(buf) });
        }
        a.push(sa);

        let mut sb = Vec::new();
        for _ in 0..n {
            file.read(&mut buf).unwrap();
            sb.push(unsafe { std::mem::transmute(buf) });
        }
        b.push(sb);
    }

    // let mut k = 2.0;
    // while k >= 0.1 {
    //     let mut e = 0.0;
    //     while e <= 2.0 {
    //         let result = permutation::get_periods(permutation::significant_indices(
    //             &permutation::run_permutation(
    //                 &a, &b, 1000,
    //                 &mut |a, b| {
    //                     for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
    //                         v.value = tv.abs();
    //                     }
    //                     tfce(&mut voxels, k, e);
    //                     voxels.iter().map(|v| v.tfce_value).collect()
    //                 }
    //             )
    //         ));

    //         eprintln!("result: {:?}, k = {}, e = {}", result, k, e);

    //         e += 0.1;
    //     }

    //     k -= 0.1;
    // }


    // for (v, tv) in voxels.iter_mut().zip(ttest::ttest_rel_vec(&a.iter().collect(), &b.iter().collect()).into_iter()) {
    //     v.value = tv.abs();
    // }
    // tfce(&mut voxels, 1.9, 0.0);

    // println!("import numpy");
    // println!("from matplotlib import pyplot");
    // println!("pyplot.plot({:?}, 'b')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    // println!("pyplot.show()")

    let result = permutation::get_periods(permutation::significant_indices(
        &permutation::run_permutation(
            &a, &b, 1000,
            &mut |a, b| {
                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                    v.value = tv.abs();
                }
                tfce(&mut voxels, 1.9, 0.0);
                voxels.iter().map(|v| v.tfce_value).collect()
            }
        )
    ));

    eprintln!("result: {:?}", result);

}