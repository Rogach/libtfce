extern crate libtfce;
extern crate clap;
extern crate byteorder;

use libtfce::field::generate_1d_field;
use libtfce::tfce;
use libtfce::ttest;
use libtfce::permutation;
use libtfce::read_data_file;

use clap::{Arg, App};
use std::fs::File;
use byteorder::{LittleEndian, WriteBytesExt};

fn main() {
    let args = App::new("libtfce")
        .arg(Arg::with_name("type")
             .long("type")
             .value_name("1d")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("input-file")
             .long("input-file")
             .value_name("filename")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("output-file")
             .long("output-file")
             .value_name("filename")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("k")
             .long("k")
             .short("k")
             .value_name("value")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("e")
             .long("e")
             .short("e")
             .value_name("value")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("permutation-count")
             .long("permutation-count")
             .value_name("N")
             .required(true)
             .takes_value(true))
        .get_matches();

    let permutation_count =
        args.value_of("permutation-count").unwrap().parse::<i32>()
        .expect("failed to parse permutation-cost");
    let k =
        args.value_of("k").unwrap().parse::<f64>()
        .expect("failed to parse k");
    let e =
        args.value_of("e").unwrap().parse::<f64>()
        .expect("failed to parse e");

    match args.value_of("type") {
        Some("1d") => {
            let data_file = args.value_of("input-file").unwrap().into();

            let (a, b) = read_data_file(data_file);
            let mut voxels = generate_1d_field(a[0].len());

            let result = permutation::run_permutation(
                &a, &b, permutation_count,
                &mut |a, b| {
                    for (v, tv) in voxels.iter_mut().zip(::ttest::ttest_rel_vec(&a, &b).into_iter()) {
                        v.value = tv.abs();
                    }
                    tfce(&mut voxels, k, e);
                    voxels.iter().map(|v| v.tfce_value).collect()
                }
            );

            eprintln!("Statistically significant periods: {:?}", permutation::get_periods(permutation::significant_indices(&result)));

            let mut output_file =
                File::create(args.value_of("output-file").unwrap())
                .expect("failed to create output file");

            output_file.write_u32::<LittleEndian>(result.len() as u32).unwrap();
            for b in result.into_iter() {
                output_file.write_u8(if b { 1 } else { 0 }).unwrap();
            }
        },
        _ => panic!("unknown operation type")
    };
}
