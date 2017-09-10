extern crate libtfce;

use libtfce::field::{generate_1d_field, set_random_values};
use libtfce::tfce;
use libtfce::approximate_tfce;

fn main() {
    let n = 100;
    let mut voxels = generate_1d_field(n);
    set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29832]);

    let mut approx_voxels = voxels.clone();
    approximate_tfce(&mut approx_voxels, 5000);
    let approx_data = approx_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();

    let mut exact_voxels = voxels.clone();
    tfce(&mut exact_voxels, 2.0/3.0, 2.0);
    let exact_data = exact_voxels.iter().map(|v| v.tfce_value).collect::<Vec<f64>>();

    println!("from matplotlib import pyplot");
    println!("pyplot.plot({:?}, 'r')", approx_data);
    println!("pyplot.plot({:?}, 'b')", exact_data);
    println!("pyplot.show()");
}
