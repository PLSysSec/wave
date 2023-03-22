#![flux::ignore]

use crate::stats::timing::{ResultsType, HOSTCALL_RESULTS, SYSCALL_RESULTS};
use statistical::mean;
use statistical::univariate::geometric_mean;
use std::fs::File;
use std::io::Write;

// better implementation of geomean
// every rust library screws this up by not controlling for overflow using the log trick
pub fn geomean(arr: &[f64]) -> f64 {
    f64::exp(arr.iter().map(|x| f64::ln(*x)).sum())
}

//Elk is 2.1 GHz
pub fn output_hostcall_perf_results() {
    let mut f = File::create("./hostcall_results.txt").expect("Unable to open file");

    HOSTCALL_RESULTS.with(|r| {
        for (k, v) in r.borrow().iter() {
            if !v.is_empty() {
                let mean = mean(v);
                let geomean = geometric_mean(v);
                writeln!(f, "{:?},{:?},{:?},{:?}", k, v.len(), mean, geomean);
            }
        }
    });
}

pub fn output_syscall_perf_results() {
    let mut f = File::create("./syscall_results.txt").expect("Unable to open file");

    SYSCALL_RESULTS.with(|r| {
        for (k, v) in r.borrow().iter() {
            if !v.is_empty() {
                let mean = mean(v);
                let geomean = geometric_mean(v);
                writeln!(f, "{:?},{:?},{:?},{:?}", k, v.len(), mean, geomean);
            }
        }
    });
}
