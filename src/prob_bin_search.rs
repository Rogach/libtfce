use std::f64::consts::PI;
use rand::distributions::{Normal, IndependentSample};
use rand;

#[derive(Debug)]
pub struct NormDistr {
    mean: f64,
    stddev: f64
}
impl NormDistr {
    pub fn new(mean: f64, stddev: f64) -> NormDistr {
        NormDistr { mean, stddev }
    }
    fn sample(&self) -> f64 {
        Normal::new(self.mean, self.stddev).ind_sample(&mut rand::thread_rng())
    }
    fn var(&self) -> f64 {
        self.stddev.powi(2)
    }
    fn pdf(&self, x: f64) -> f64 {
        1.0 / (2.0 * PI * self.var()).sqrt() * (- (x-self.mean).powi(2)/(2.0*self.var())).exp()
    }
    fn cdf(&self, x: f64) -> f64 {
        let b0 = 0.2316419;
        let b1 = 0.319381530;
        let b2 = -0.356563782;
        let b3 = 1.781477937;
        let b4 = -1.821255978;
        let b5 = 1.330274429;
        let x = (x - self.mean) / self.stddev;
        if x > 0.0 {
            let t = 1.0 / (1.0 + b0 * x);
            let ts = t * (b1 + t * (b2 + t * (b3 + t * (b4 + t * b5))));
            1.0 - NormDistr::new(0.0, 1.0).pdf(x) * ts
        } else {
            let x = -x;
            let t = 1.0 / (1.0 + b0 * x);
            let ts = t * (b1 + t * (b2 + t * (b3 + t * (b4 + t * b5))));
            NormDistr::new(0.0, 1.0).pdf(x) * ts
        }
    }
}

pub fn probabilistic_binary_search(
    mut prior: NormDistr,
    n: i32,
    test_is_greater_than_x: &mut FnMut(f64) -> bool
) -> NormDistr {
    let mut tests = Vec::new();
    for _ in 0..n {
        let test_x = NormDistr::new(prior.mean, prior.stddev + 1.0).sample();
        tests.push(BinaryTest { x: test_x, is_greater_than_x: test_is_greater_than_x(test_x) });
        prior = compute_posterior(&tests);
    }
    prior
}

fn compute_posterior(tests: &Vec<BinaryTest>) -> NormDistr {
    let mut fuzzy_upper_bound = 2.0;
    let mut fuzzy_lower_bound = -2.0;
    if let Some(gt_mean) = mean(tests.iter().filter(|t| t.is_greater_than_x).map(|t| t.x).collect()) {
        if let Some(lt_mean) = mean(tests.iter().filter(|t| !t.is_greater_than_x).map(|t| t.x).collect()) {
            fuzzy_lower_bound = gt_mean;
            fuzzy_upper_bound = lt_mean;
        }
    }

    let start_mean = (fuzzy_lower_bound + fuzzy_upper_bound) / 2.0;
    let start_stddev = (0.3f64).max((fuzzy_upper_bound - fuzzy_lower_bound).abs() / 4.0);
    let start_speed = 0.1;

    let (max_mean, max_stddev) = hill_climb(
        start_mean, start_stddev, start_speed, 0.001,
        &|mean, stddev| {
            let mut pt = 0.0;
            let d = NormDistr::new(mean, stddev);
            for &BinaryTest { x, is_greater_than_x } in tests.iter() {
                if is_greater_than_x {
                    pt *= 1.0 - d.cdf(x);
                } else {
                    pt *= d.cdf(x);
                }
            }
            pt
        }
    );

    NormDistr::new(max_mean, max_stddev)
}

fn hill_climb(
    mut x: f64,
    mut y: f64,
    mut d: f64,
    min_d: f64,
    op: &Fn(f64, f64) -> f64
) -> (f64, f64) {
    let mut z = op(x, y);

    while d >= min_d {
        let mut changed = false;

        let mut on_x = true;
        while on_x {
            let z_x_m = op(x - d, y);
            let z_x_p = op(x + d, y);
            if z_x_p > z {
                if z_x_m <= z || z_x_p > z_x_m {
                    x += d;
                    z = z_x_p;
                    changed = true;
                } else {
                    x -= d;
                    z = z_x_m;
                    changed = true;
                }
            } else if z_x_m > z {
                if z_x_p <= z || z_x_m > z_x_p {
                    x -= d;
                    z = z_x_m;
                    changed = true;
                } else {
                    x += d;
                    z = z_x_p;
                    changed = true;
                }
            } else {
                on_x = false;
            }
        }

        let mut on_y = true;
        while on_y {
            let z_y_m = op(x, y - d);
            let z_y_p = op(x, y + d);
            if z_y_p > z {
                if z_y_p <= z || z_y_p > z_y_m {
                    y += d;
                    z = z_y_p;
                    changed = true;
                } else {
                    y -= d;
                    z = z_y_m;
                    changed = true;
                }
            } else if z_y_m > z {
                if z_y_p <= z || z_y_m > z_y_p {
                    y -= d;
                    z = z_y_m;
                    changed = true;
                } else {
                    y += d;
                    z = z_y_p;
                    changed = true;
                }
            } else {
                on_y = false;
            }
        }

        if !changed {
            d /= 2.0;
        }
    }

    (x, y)
}

fn mean(values: Vec<f64>) -> Option<f64> {
    let n = values.len();
    let sum: f64 = values.iter().sum();
    if n > 0 {
        Some(sum / (n as f64))
    } else {
        None
    }
}

struct BinaryTest {
    x: f64,
    is_greater_than_x: bool
}
