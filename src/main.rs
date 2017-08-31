#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]

extern crate rand;
extern crate test;

mod voxel;
mod voxel_priority;
mod field;
mod tfce;

use ::field::generate_1d_field;
use ::tfce::tfce;
use ::tfce::approximate_tfce::approximate_tfce;

fn main() {
    let voxels = generate_1d_field(100, 0.0, 10.0);

    println!("from matplotlib import pyplot");
    {
        let mut voxels = voxels.clone();
        approximate_tfce(&mut voxels, 50);
        println!("pyplot.plot({:?}, 'r')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    }
    {
        let mut voxels = voxels.clone();
        tfce(&mut voxels);
        println!("pyplot.plot({:?}, 'b')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    }
    println!("pyplot.show()");
}
