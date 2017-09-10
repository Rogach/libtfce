extern crate libtfce;

use libtfce::tfce;
use libtfce::ttest;
use libtfce::field::generate_1d_field;
use libtfce::read_data_file;
use std::env;

fn main() {
    let data_file = env::args().nth(1).expect("expected input filename as first argument");

    let (a, b) = read_data_file(data_file);
    let mut voxels = generate_1d_field(a[0].len());

    for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a.iter().collect(), &b.iter().collect()).into_iter()) {
        v.value = tv.abs();
    }

    tfce(&mut voxels, 1.9, 0.0);

    println!("from matplotlib import pyplot");
    println!("pyplot.plot({:?}, 'b')", voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>());
    println!("pyplot.show()");
}
