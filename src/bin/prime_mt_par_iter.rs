use par_iter::prelude::*;
use rust_prime::check_primality;

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();

    while test < test_halt {
        let test_limit = primes.last().unwrap().pow(2).min(test_halt);
        // Copy the known primes, rather than collecting new primes and appending
        // because known primes length is likely less than the new primes length.
        // k+(k+n) < n+(k+n)
        let primes_copy = primes.clone();
        primes.par_extend(
            { (test..test_limit).into_par_iter() }
                .filter(|&test| check_primality(test, &primes_copy)),
        );
        test = test_limit;
        { primes[primes_copy.len()..].iter() }.for_each(|p| println!("{p}"));
    }
}
