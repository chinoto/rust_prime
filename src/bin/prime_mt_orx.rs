use orx_parallel::*;
use rust_prime::check_primality;

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();

    while test < test_halt {
        let last_square = primes.last().unwrap().pow(2);
        let test_limit = last_square.min(test_halt);
        let new_primes = { (test..test_limit).into_par() }
            .filter(|&test| check_primality(test, &primes))
            .collect::<Vec<usize>>();
        new_primes.iter().for_each(|p| println!("{p}"));
        primes.extend(new_primes);
        test = test_limit;
    }
}
