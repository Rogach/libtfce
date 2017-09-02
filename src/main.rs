#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]
#![allow(dead_code)]

extern crate rand;
extern crate test;

mod voxel;
mod voxel_priority;
mod field;
mod tfce;
mod ttest;

use ::field::generate_1d_field;
use ::field::generate_2d8c_field;
use ::tfce::tfce;
use ::tfce::approximate_tfce::approximate_tfce;

fn main() {
    test_2d8c();
}

fn fuzztest() {
    for x in 29830..29900 {
        println!("x = {}", x);
        let n = 10;
        let voxels = generate_2d8c_field(n, 0.0, 1.0, &[17556, 31771, 29830, x]);

        let mut approx_voxels = voxels.clone();
        approximate_tfce(&mut approx_voxels, 10000);

        let mut exact_voxels = voxels.clone();
        tfce(&mut exact_voxels);

        for n in 0..voxels.len() {
            if (approx_voxels[n].tfce_value - exact_voxels[n].tfce_value).abs() > 1e-3 {
                panic!("difference at x = {}", x);
            }
        }
    }
}

fn test_2d8c() {
    let n = 20;
    let voxels = generate_2d8c_field(n, 0.0, 1.0, &[17556, 31771, 29830, 29832]);

    println!("import numpy");
    println!("from matplotlib import pyplot");

    let orig_data = voxels.iter().map(|v| v.value).collect::<Vec<f64>>();
    println!("pyplot.subplot2grid((3, 2), (0,0))");
    println!("pyplot.title('orig data')");
    println!(
        "pyplot.imshow(numpy.array({:?}).reshape({},{}), cmap='Greys')",
        orig_data, n, n
    );

    let mut approx_voxels = voxels.clone();
    approximate_tfce(&mut approx_voxels, 5000);
    let approx_data = approx_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();
    println!("pyplot.subplot2grid((3,2), (0, 1))");
    println!("pyplot.title('approximate')");
    println!(
        "pyplot.imshow(numpy.array({:?}).reshape({},{}), cmap='Greys')",
        approx_data, n, n
    );

    let mut exact_voxels = voxels.clone();
    tfce(&mut exact_voxels);
    let exact_data = exact_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();
    println!("pyplot.subplot2grid((3,2), (1,0))");
    println!("pyplot.title('exact')");
    println!(
        "pyplot.imshow(numpy.array({:?}).reshape({},{}), cmap='Greys')",
        exact_data, n, n
    );

    println!("pyplot.subplot2grid((3,2), (1,1))");
    println!("pyplot.title('diff')");
    println!(
        "pyplot.imshow((numpy.array({:?})-numpy.array({:?})).reshape({},{}), cmap='coolwarm', vmin=-0.1, vmax=0.1)",
        exact_data, approx_data, n, n
    );

    println!("pyplot.subplot2grid((3,2), (2,0), 1, 2)");
    println!("pyplot.title('raw')");
    println!("pyplot.plot({:?}, 'r')", approx_data);
    println!("pyplot.plot({:?}, 'b')", exact_data);

    println!("pyplot.show()");
}

fn test_1d() {
    let n = 100;
    let voxels = generate_1d_field(n, 0.0, 1.0, &[17556, 31771, 29830, 29832]);

    let mut approx_voxels = voxels.clone();
    approximate_tfce(&mut approx_voxels, 5000);
    let approx_data = approx_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();

    let mut exact_voxels = voxels.clone();
    tfce(&mut exact_voxels);
    let exact_data = exact_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();

    println!("from matplotlib import pyplot");
    println!("pyplot.plot({:?}, 'r')", approx_data);
    println!("pyplot.plot({:?}, 'b')", exact_data);
    println!("pyplot.show()");
}
