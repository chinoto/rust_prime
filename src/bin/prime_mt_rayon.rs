use rayon::prelude::*;
use rust_prime::check_primality;

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();

    while test < test_halt {
        let test_limit = primes.last().unwrap().pow(2).min(test_halt);
        // Can't push into primes list while reading it because that might cause a reallocation,
        // thus breaking any readers. Also, the borrow checker doesn't like it...

        // The copy vector's allocation can't be reused because the source vector will always be
        // bigger and require a reallocation.

        // TODO: Potential solution is to pad the primes vector with 0s and use .split_at_mut() to
        // read the existing primes in the first slice and write the new primes in the second slice.
        let primes_copy = primes.clone();
        primes.par_extend(
            // rayon doesn't support `a..=b`, `a..b+1` is equivalent in this case.
            #[allow(clippy::range_plus_one)]
            { (test..test_limit + 1).into_par_iter() }
                .filter(|&test| check_primality(test, &primes_copy)),
        );
        test = test_limit + 1;
    }
    // Other impls only print primes added to the list, 2 was already there.
    primes[1..].iter().for_each(|p| println!("{p:?}"));
}
