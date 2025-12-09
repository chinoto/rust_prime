use chili::Scope;
use rust_prime::{THREAD_WORK_LIMIT, check_primality};
use std::ops::Range;

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut scope = Scope::global();

    while test < test_halt {
        let test_limit = primes.last().unwrap().pow(2).min(test_halt);
        let range = test..test_limit;
        let new_primes = find_primes_in_range(&mut scope, &primes, range);
        let new_primes_iter = new_primes.into_iter().flatten();
        primes.extend(new_primes_iter.inspect(|p| println!("{p}")));
        test = test_limit;
    }
}

fn find_primes_in_range(s: &mut Scope, primes: &[usize], range: Range<usize>) -> Vec<Vec<usize>> {
    if range.len() < THREAD_WORK_LIMIT {
        let new_primes = range
            .filter(|&test| check_primality(test, primes))
            .collect::<Vec<usize>>();
        // A small allocation to wrap this batch is likely faster than using single layer
        // Vectors that will need to be repeatedly grown and have their contents moved.
        return vec![new_primes];
    }
    let Range { start, end } = range;
    let mid = start.midpoint(end);
    let range_a = start..mid;
    let range_b = mid..end;
    let (mut vec_a, vec_b) = s.join(
        |s| find_primes_in_range(s, primes, range_a),
        |s| find_primes_in_range(s, primes, range_b),
    );
    vec_a.extend(vec_b);
    vec_a
}
