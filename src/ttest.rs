pub fn ttest_rel(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let nsubj = a.len();
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut i = 0;
    while i < nsubj {
        let v = a[i] - b[i];
        sum += v;
        sum2 += v*v;
        i += 1;
    }
    sum / ((sum2*(nsubj as f64) - sum*sum)/((nsubj - 1) as f64)).sqrt()
}

pub fn ttest_rel_vec(a: &Vec<&Vec<f64>>, b: &Vec<&Vec<f64>>) -> Vec<f64> {
    let nsubj = a.len();
    let n = a[0].len();

    let mut result = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut s = 0;
        let mut sum = 0.0;
        let mut sum2 = 0.0;
        while s < nsubj {
            let v = unsafe {
                a.get_unchecked(s).get_unchecked(i) - b.get_unchecked(s).get_unchecked(i)
            };
            sum += v;
            sum2 += v*v;
            s += 1;
        }
        result.push(sum / ((sum2*(nsubj as f64) - sum*sum)/((nsubj - 1) as f64)).sqrt());

        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use rand::{Rng, StdRng, SeedableRng};

    #[test]
    fn test_ttest_rel() {
        let a = vec![ 0.78410583,  0.7053225 ,  0.04590954,  0.47383383,  0.71913645];
        let b = vec![ 0.44289209,  0.60141457,  0.06801757,  0.23473256,  0.41713896];
        assert!((ttest_rel(&a, &b) - 2.8715290847363173).abs() < 1e-7);

        let a = vec![ 0.04378271,  0.26757753,  0.18631389,  0.38958731,  0.60529497];
        let b = vec![ 0.30417776,  0.88567478,  0.78889515,  0.5128209 ,  0.87737296];
        assert!((ttest_rel(&a, &b) - -3.7716505476992812).abs() < 1e-7);

        let a = vec![ 0.06973489,  0.10810649,  0.14355822,  0.1778686 ,  0.43980159];
        let b = vec![ 0.21664624,  0.7943964 ,  0.54497303,  0.42043867,  0.75062018];
        assert!((ttest_rel(&a, &b) - -3.8813890034033038).abs() < 1e-7);
    }

    #[bench]
    fn benchmark_ttest(bench: &mut Bencher) {
        let mut rng = StdRng::from_seed(&[17556, 31771, 29830, 29830]);
        let n = 10000;
        let nsubj = 20;

        let mut a = Vec::with_capacity(nsubj);
        for _ in 0..nsubj {
            let mut data = Vec::with_capacity(n);
            for _ in 0..n {
                data.push(rng.next_f64());
            }
            a.push(data);
        }

        let mut b = Vec::with_capacity(nsubj);
        for _ in 0..nsubj {
            let mut data = Vec::with_capacity(n);
            for _ in 0..n {
                data.push(rng.next_f64());
            }
            b.push(data);
        }

        bench.iter(|| {
            rng.reseed(&[17556, 31771, 29830, 29830]);
            let mut ap = Vec::new();
            let mut bp = Vec::new();
            for (a, b) in a.iter().zip(b.iter()) {
                if rng.gen() {
                    ap.push(a); bp.push(b);
                } else {
                    bp.push(a); ap.push(b);
                }
            }
            ttest_rel_vec(&ap, &bp)
        });
    }
}
