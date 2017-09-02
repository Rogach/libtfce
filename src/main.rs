#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]
#![allow(dead_code)]

extern crate rand;
extern crate test;
extern crate num_cpus;
extern crate chan;

mod voxel;
mod voxel_priority;
mod field;
mod tfce;
mod ttest;
mod permutation;

use ::field::generate_1d_field;
use ::field::generate_2d8c_field;
use ::field::set_random_values;
use ::tfce::tfce;
use ::tfce::approximate_tfce::approximate_tfce;
use rand::{Rng, StdRng, SeedableRng};
use std::thread;

fn main() {
    let cpus = num_cpus::get();
    println!("Available CPUs: {}", cpus);

    let (send_result, recv_result) = chan::async();
    let (send_work, recv_work) = chan::sync(512);

    let generator_thread = thread::spawn(move || {
        for x in 0..10 {
            send_work.send(x);
        }
    });

    let result_thread = thread::spawn(move || {
        for (w, r) in recv_result.iter() {
            println!("got result: {} {}", w, r);
        }
    });

    let mut worker_threads = Vec::new();
    for _ in 0..cpus {
        let send_result = send_result.clone();
        let recv_work = recv_work.clone();
        worker_threads.push(thread::spawn(move || {
            while let Some(w) = recv_work.recv() {
                thread::sleep(::std::time::Duration::from_millis(1000));
                send_result.send((w, w*w));
            }
        }));
    }

    for t in worker_threads.into_iter() {
        t.join().unwrap();
    }
    std::mem::drop(send_result);

    result_thread.join().unwrap();

    generator_thread.join().unwrap();
}

fn example_permutation() {
    let nsubj = 10;
    let n = 100;

    let mut voxels = generate_1d_field(n);

    let mut a = Vec::with_capacity(nsubj);
    let mut b = Vec::with_capacity(nsubj);

    let mut rng = StdRng::from_seed(&[17556, 31771, 29830, 29830]);

    for _ in 0..nsubj {
        let mut sa = Vec::with_capacity(n);
        let mut sb = Vec::with_capacity(n);
        for _ in 0..n {
            sa.push(rng.next_f64());
            sb.push(rng.next_f64());
        }
        a.push(sa);
        b.push(sb);
    }

    for x in 5..15 {
        for s in 0..nsubj {
            b[s][x] += 0.8;
        }
    }

    let result = permutation::significant_indices(
        &permutation::run_permutation(
            &a, &b, 200,
            &mut |a, b| {
                for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                    v.value = tv.abs();
                }
                tfce(&mut voxels);
                voxels.iter().map(|v| v.tfce_value).collect()
            }
        )
    );

    println!("{:?}", result);
}

fn tfce_fuzztest() {
    for x in 29830..29900 {
        println!("x = {}", x);
        let n = 10;
        let mut voxels = generate_2d8c_field(n);
        set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, x]);

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

fn example_tfce_2d8c() {
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

fn example_tfce_1d() {
    let n = 100;
    let mut voxels = generate_1d_field(n);
    set_random_values(&mut voxels, 0.0, 1.0, &[17556, 31771, 29830, 29832]);

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
