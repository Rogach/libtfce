#![feature(test)]
#![feature(type_ascription)]
#![feature(box_patterns)]

extern crate rand;
extern crate test;

pub mod voxel;
pub mod field;

pub mod tfce;
pub use self::tfce::tfce;
pub use self::tfce::approximate_tfce;

pub mod prob_bin_search;
pub use self::prob_bin_search::probabilistic_binary_search;

pub mod ttest;
pub mod permutation;
mod voxel_priority;
