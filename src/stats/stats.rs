use crate::stats::timing::{ResultsType, HOSTCALL_RESULTS, SYSCALL_RESULTS};
use statistical::mean;
use statistical::univariate::geometric_mean;

//Elk is 2.1 GHz
pub fn output_hostcall_perf_results() {
    HOSTCALL_RESULTS.with(|r| {
        // println!("results: {:?}", r);
        for (k, v) in r.borrow().iter() {
            if !v.is_empty() {
                let mean = mean(v);
                println!("{:?}: num_samples = {:?} mean = {:?} ns", k, v.len(), mean)
            }
        }
    });
}

pub fn output_syscall_perf_results() {
    SYSCALL_RESULTS.with(|r| {
        // println!("results: {:?}", r);
        for (k, v) in r.borrow().iter() {
            if !v.is_empty() {
                let mean = mean(v);
                println!("{:?}: num_samples = {:?} mean = {:?} ns", k, v.len(), mean)
            }
        }
    });
}
