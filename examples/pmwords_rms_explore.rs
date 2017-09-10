extern crate libtfce;

use libtfce::field::generate_1d_field;
use libtfce::read_data_file;
use std::env;

fn main() {
    let data_file = env::args().nth(1).expect("expected input filename as first argument");

    let (a, b) = read_data_file(data_file);
    let mut voxels = generate_1d_field(a[0].len());

    libtfce::explore_tfce_permutation(
        &a, &b,
        1000,
        &mut voxels,
        1.0, 0.1, 2.0,
        0.0, 0.1, 1.0
    );
}
