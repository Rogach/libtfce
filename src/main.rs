#![feature(exact_chunks)]

extern crate libtfce;
extern crate clap;
extern crate byteorder;

use libtfce::field::generate_1d_field;
use libtfce::tfce;
use libtfce::permutation;
use libtfce::read_data_file;
use libtfce::explore_tfce_permutation;
use libtfce::freesurfer;
use libtfce::stc;

use clap::{Arg, App};
use std::fs::File;
use byteorder::{LittleEndian, WriteBytesExt};

fn main() {
    let args = App::new("libtfce")
        .arg(Arg::with_name("type").long("type").value_name("tp").required(true).takes_value(true)
             .possible_values(&["1d", "mesh-time"])
             .display_order(1)
             .help("TFCE graph configuration"))

        .arg(Arg::with_name("permutation-count").long("permutation-count").short("n").value_name("N").required(true).takes_value(true)
             .help("Number of permutations"))
        .arg(Arg::with_name("h").short("h").value_name("value").required(true).takes_value(true)
             .help("TFCE parameter H, intensity weighting (2 is recommended)"))
        .arg(Arg::with_name("e").short("e").value_name("value").required(true).takes_value(true)
             .help("TFCE parameter E, cluster extent weighting (0.666 is recommended)"))

        .arg(Arg::with_name("explore").long("explore").takes_value(false)
             .help("run in exploration mode: try all H values from --h to --h-max (increment by --h-step), try all E values from --e to --e-max (increment by --e-step)"))
        .arg(Arg::with_name("h-max").long("h-max").value_name("value").takes_value(true)
             .help("max value for H in --explore mode"))
        .arg(Arg::with_name("h-step").long("h-step").value_name("value").takes_value(true)
             .help("increment for H in --explore mode"))
        .arg(Arg::with_name("e-max").long("e-max").value_name("value").takes_value(true)
             .help("max value for E in --explore mode"))
        .arg(Arg::with_name("e-step").long("e-step").value_name("value").takes_value(true)
             .help("increment for E in --explore mode"))

        .arg(Arg::with_name("input-file").long("input-file").value_name("filename").takes_value(true)
             .help("Input file for (type=1d)"))
        .arg(Arg::with_name("output-file").long("output-file").value_name("filename").takes_value(true)
             .help("Output file (type=1d)"))

        .arg(Arg::with_name("source-space").long("source-space").value_name("filename").takes_value(true)
             .help("Freesurfer source space .fif file, used to extract mesh data (type=mesh-time)"))
        .arg(Arg::with_name("input-stcs").long("input-stcs").value_name("filenames...").takes_value(true).multiple(true)
             .help("Input stc files, 4 per subject. Files must be in order: subj1-condA-lh.stc, subj1-condA-rh.stc, subj1-condB-lh.stc, subj1-condB-rh.stc, subj2-condA-lh.stc, etc. (type=mesh-time)"))
        .arg(Arg::with_name("output-stcs").long("output-stcs").value_name("lh.stc rh.stc").takes_value(true).number_of_values(2)
             .help("Output stc files, lh and rh stc filenames (type=mesh-time)"))

        .get_matches();

    let permutation_count =
        args.value_of("permutation-count").unwrap().parse::<i32>()
        .expect("failed to parse permutation-cost");
    let h =
        args.value_of("h").unwrap().parse::<f64>()
        .expect("failed to parse h");
    let e =
        args.value_of("e").unwrap().parse::<f64>()
        .expect("failed to parse e");

    let explore = args.is_present("explore");

    match args.value_of("type") {
        Some("1d") => {
            let data_file = args.value_of("input-file").expect("--input-file not provided").into();

            let (a, b) = read_data_file(data_file);
            let mut voxels = generate_1d_field(a[0].len());

            if !explore {
                let result = tfce::run_permutation(
                    &mut voxels,
                    &a, &b,
                    permutation_count,
                    e, h
                );

                eprintln!("Statistically significant periods: {:?}", permutation::get_periods(permutation::significant_indices(&result)));

                let mut output_file =
                    File::create(args.value_of("output-file").expect("--output-file not provided"))
                        .expect("failed to create output file");

                output_file.write_u32::<LittleEndian>(result.len() as u32).unwrap();
                for b in result.into_iter() {
                    output_file.write_u8(if b { 1 } else { 0 }).unwrap();
                }
            } else {
                let h_max =
                    args.value_of("h-max").expect("--h-max is required for explore mode")
                    .parse::<f64>().expect("failed to parse h-max");
                let h_step =
                    args.value_of("h-step").expect("--h-step is required for explore mode")
                    .parse::<f64>().expect("failed to parse h-step");
                let e_max =
                    args.value_of("e-max").expect("--e-max is required for explore mode")
                    .parse::<f64>().expect("failed to parse e-max");
                let e_step =
                    args.value_of("e-step").expect("--e-step is required for explore mode")
                    .parse::<f64>().expect("failed to parse e-step");

                explore_tfce_permutation(
                    &a, &b,
                    permutation_count,
                    &mut voxels,
                    e, e_step, e_max,
                    h, h_step, h_max
                );
            }
        },
        Some("mesh-time") => {
            let source_space_filename =
                args.value_of("source-space")
                .expect("--source-space is required for type=mesh-time");

            let input_stc_filenames =
                args.values_of("input-stcs")
                .expect("--input-stcs is required for type=mesh-time")
                .collect::<Vec<&str>>();

            if input_stc_filenames.len() % 4 != 0 {
                panic!("--input-stcs must contain 4 files per subject");
            }

            let output_stc_filenames =
                args.values_of("output-stcs")
                .expect("--output-stcs is required for type=mesh-time")
                .collect::<Vec<&str>>();

            if output_stc_filenames.len() != 2 {
                panic!("--output-stcs must contain 2 filenames");
            }

            let mut stcs_a = Vec::new();
            let mut stcs_b = Vec::new();

            let mut a = Vec::new();
            let mut b = Vec::new();

            for subj_stcs in input_stc_filenames.exact_chunks(4) {
                let (a_lh, a_rh) = (stc::read(subj_stcs[0]), stc::read(subj_stcs[1]));
                let (b_lh, b_rh) = (stc::read(subj_stcs[2]), stc::read(subj_stcs[3]));

                a.push(stc::concat_pair(&a_lh, &a_rh));
                b.push(stc::concat_pair(&b_lh, &b_rh));

                stcs_a.push((a_lh, a_rh));
                stcs_b.push((b_lh, b_rh));
            }

            let mut voxels =
                freesurfer::extend_graph_into_time(
                    freesurfer::read_source_space_to_graph(source_space_filename),
                    stcs_a[0].0.time_count
                );

            if !explore {
                let result = tfce::run_permutation(
                    &mut voxels,
                    &a, &b,
                    permutation_count,
                    e, h
                );

                eprintln!("Statistically significant periods: {:?}", permutation::get_periods(permutation::significant_indices(&result)).len());

                let mut output_stc_lh = stc::read(input_stc_filenames[0]);
                let mut output_stc_rh = stc::read(input_stc_filenames[1]);
                let mut output_data_lh = Vec::new();
                let mut output_data_rh = Vec::new();
                for t in 0..output_stc_lh.time_count {
                    let mut time_slice_lh = Vec::new();
                    for i in 0..output_stc_lh.vertex_count {
                        time_slice_lh.push(if result[(output_stc_lh.vertex_count + output_stc_rh.vertex_count) * t + i] { 1.0 } else { 0.0 });
                    }
                    output_data_lh.push(time_slice_lh);

                    let mut time_slice_rh = Vec::new();
                    for i in 0..output_stc_rh.vertex_count {
                        time_slice_rh.push(if result[(output_stc_lh.vertex_count + output_stc_rh.vertex_count) * t + output_stc_lh.vertex_count + i] { 1.0 } else { 0.0 });
                    }
                    output_data_rh.push(time_slice_rh);
                }

                output_stc_lh.data = output_data_lh;
                output_stc_rh.data = output_data_rh;
                stc::write(output_stc_filenames[0], output_stc_lh);
                stc::write(output_stc_filenames[1], output_stc_rh);

            } else {
                let h_max =
                    args.value_of("h-max").expect("--h-max is required for explore mode")
                    .parse::<f64>().expect("failed to parse h-max");
                let h_step =
                    args.value_of("h-step").expect("--h-step is required for explore mode")
                    .parse::<f64>().expect("failed to parse h-step");
                let e_max =
                    args.value_of("e-max").expect("--e-max is required for explore mode")
                    .parse::<f64>().expect("failed to parse e-max");
                let e_step =
                    args.value_of("e-step").expect("--e-step is required for explore mode")
                    .parse::<f64>().expect("failed to parse e-step");

                explore_tfce_permutation(
                    &a, &b,
                    permutation_count,
                    &mut voxels,
                    e, e_step, e_max,
                    h, h_step, h_max
                );
            }
        },
        _ => panic!("unknown operation type: {}", args.value_of("type").unwrap())
    };
}
