extern crate libtfce;

use libtfce::field::{generate_2d8c_field, set_random_values};
use libtfce::tfce;
use libtfce::approximate_tfce;

fn main() {
    let n = 20;
    let mut voxels = generate_2d8c_field(n);
    set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29832]);

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
    tfce(&mut exact_voxels, 2.0/3.0, 2.0);
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
