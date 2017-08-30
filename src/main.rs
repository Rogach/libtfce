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

fn main() {
    let mut voxels = generate_1d_field(1000000, 0.0, 10.0);
    tfce(&mut voxels);
}
