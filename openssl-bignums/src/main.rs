use std::io;
use std::io::Write;
use cpu_time::ProcessTime;
use openssl::bn::BigNum;
use openssl::bn::BigNumRef;
use openssl::bn::BigNumContext;
// use openssl::error::ErrorStack;

fn compute_mean(data: &Vec<u128>) -> Option<f64> {
    let sum = data.iter().sum::<u128>() as f64;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

fn compute_std_deviation(data: &Vec<u128>) -> Option<f64> {
    match (compute_mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data.iter().map(|value| {
                let diff = data_mean - (*value as f64);

                diff * diff
            }).sum::<f64>() / count as f64;

            Some(variance.sqrt())
        },
        _ => None
    }
}

fn main() {

    /* Check number of params */
    if std::env::args().len() != 3 {
	println!("Usage: cargo run <n_attrs> <n_iters>");
	std::process::exit(1);
    }

    /* Get maximum number of "attributes" to simulate */
    let n_attrs = std::env::args().nth(1).
	expect("No number of attributes to simulate given").
	parse::<i32>().unwrap();

    /* Get number of iterations for benchmark */
    let n_iters = std::env::args().nth(2).
	expect("No number of times to iterate tests given").
	parse::<i32>().unwrap();

    let mut cpu_all_times: Vec<u128> = Vec::new();
    let mut cpu_primes_times: Vec<u128> = Vec::new();
    let mut cpu_qrs_times: Vec<u128> = Vec::new();

    let mut means_all: Vec<f64>  = Vec::new();
    let mut stddevs_all: Vec<f64>  = Vec::new();
    let mut means_primes: Vec<f64>  = Vec::new();
    let mut stddevs_primes: Vec<f64>  = Vec::new();
    let mut means_qrs: Vec<f64>  = Vec::new();
    let mut stddevs_qrs: Vec<f64>  = Vec::new();

    /* Generate OpenSSL context */
    let mut ctx = BigNumContext::new_secure().unwrap();

    for k in 1..n_attrs+1 {
    
	for _i in 0..n_iters {

	    print!("\r// n_attrs: {}; iter: {}", k, _i);
	    let _ = io::stdout().flush();

	    let cpu_iter_all_start = ProcessTime::try_now().
		expect("Getting CPU time failed");

	    let cpu_iter_primes_start = ProcessTime::try_now().
		expect("Getting CPU time failed");
		
	    /* Compute safe prime p of 1024 bits */
	    let mut p = BigNum::new().unwrap();
	    let _ = p.generate_prime(1024, true, None, None);

	    /* Compute safe prime q of 1024 bits */
	    let mut q = BigNum::new().unwrap();
	    let _ = q.generate_prime(1024, true, None, None);

	    let cpu_iter_primes_duration = cpu_iter_primes_start.elapsed();

	    /* Compute n=pq -- roughly 2048 bits */
	    let mut n = BigNum::new().unwrap();
	    let _ = BigNumRef::checked_mul(&mut n, &p, &q, &mut ctx);

	    /* Compute k random QRs mod n */
	    let cpu_iter_qrs_start = ProcessTime::try_now().
		expect("Getting CPU time failed");
	    
	    for _j in 1..k+1 {
		let mut qr = BigNum::new().unwrap();
		let _ = n.rand_range(&mut qr);
		let big_one = BigNum::from_u32(1).unwrap();
		let mut gcd = BigNum::new().unwrap();
		
		let _ = gcd.gcd(&qr, &n, &mut ctx).unwrap();
		
		while gcd != big_one {		    
		    let _ = n.rand_range(&mut qr);
		}
	    }

	    let cpu_iter_qrs_duration = cpu_iter_qrs_start.elapsed();
	    let cpu_iter_all_duration = cpu_iter_all_start.elapsed();

	    cpu_all_times.push(cpu_iter_all_duration.as_millis());
	    cpu_primes_times.push(cpu_iter_primes_duration.as_millis());
	    cpu_qrs_times.push(cpu_iter_qrs_duration.as_millis());
	    
	}

	let mean_all = compute_mean(&cpu_all_times).unwrap();
	let stddev_all = compute_std_deviation(&cpu_all_times).unwrap();
	means_all.push(mean_all);
	stddevs_all.push(stddev_all);

	let mean_primes = compute_mean(&cpu_primes_times).unwrap();
	let stddev_primes = compute_std_deviation(&cpu_qrs_times).unwrap();
	means_primes.push(mean_primes);
	stddevs_primes.push(stddev_primes);	
	
	let mean_qrs = compute_mean(&cpu_qrs_times).unwrap();
	let stddev_qrs = compute_std_deviation(&cpu_qrs_times).unwrap();
	means_qrs.push(mean_qrs);
	stddevs_qrs.push(stddev_qrs);	
	
    }

    /* Spit the data */
    println!("\n");
    println!("#Benchmarks for whole CredDef generation");
    println!("#Data obtained iterating {:?} times", n_iters);
    println!("#Num attrs\tMean time (ms)\tStd dev (ms)");
    let mut it_means_all = means_all.iter();
    let mut it_stdvs_all = stddevs_all.iter();
    let mut k = 1;
    loop {
	match (it_means_all.next(), it_stdvs_all.next()) {
	    (Some(x), Some(y)) => println!("{}\t\t{:.6}\t{:.6}", k, x, y),
	    (Some(_x), None) => break,
	    (None, Some(_y)) => break,
	    (None, None) => break,
	}
	k+=1;
    }
    
    println!("\n");
    println!("#Benchmarks for CredDef prime generation");
    println!("#Data obtained iterating {:?} times", n_iters);
    println!("#Num attrs\tMean time (ms)\tStd dev (ms)");
    let mut it_means_primes = means_primes.iter();
    let mut it_stdvs_primes = stddevs_primes.iter();
    k = 1;
    loop {
	match (it_means_primes.next(), it_stdvs_primes.next()) {
	    (Some(x), Some(y)) => println!("{}\t\t{:.6}\t{:.6}", k, x, y),
	    (Some(_x), None) => break,
	    (None, Some(_y)) => break,
	    (None, None) => break,
	}
	k+=1;
    }

    println!("\n");    
    println!("#Benchmarks for CredDef QRns generation");
    println!("#Data obtained iterating {:?} times", n_iters);
    println!("#Num attrs\tMean time (ms)\tStd dev (ms)");
    let mut it_means_qrs = means_qrs.iter();
    let mut it_stdvs_qrs = stddevs_qrs.iter();
    k = 1;
    loop {
	match (it_means_qrs.next(), it_stdvs_qrs.next()) {
	    (Some(x), Some(y)) => println!("{}\t\t{:.6}\t{:.6}", k, x, y),
	    (Some(_x), None) => break,
	    (None, Some(_y)) => break,
	    (None, None) => break,
	}
	k+=1;
    }


}
