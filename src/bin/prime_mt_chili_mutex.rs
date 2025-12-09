use chili::Scope;
use rust_prime::{THREAD_WORK_LIMIT, check_primality};
use std::{collections::BTreeSet, ops::Range, sync::Mutex};

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut scope = Scope::global();

    let insert_mutex = Mutex::default();
    while test < test_halt {
        let test_limit = primes.last().unwrap().pow(2).min(test_halt);
        let range = test..test_limit;
        find_primes_in_range(&mut scope, &primes, range, &insert_mutex);
        let insert_buffer = std::mem::take(&mut *insert_mutex.lock().unwrap());
        let insert_iter = insert_buffer.into_iter().flatten();
        primes.extend(insert_iter.inspect(|p| println!("{p}")));
        test = test_limit;
    }
}

fn find_primes_in_range(
    s: &mut Scope,
    primes: &[usize],
    range: Range<usize>,
    insert_buffer: &Mutex<BTreeSet<Vec<usize>>>,
) {
    if range.len() < THREAD_WORK_LIMIT {
        let new_primes = range
            .filter(|&test| check_primality(test, primes))
            .collect::<Vec<usize>>();
        let mut guard = insert_buffer.lock().unwrap();
        guard.insert(new_primes);
        return;
    }
    let Range { start, end } = range;
    let mid = start.midpoint(end);
    let range_a = start..mid;
    let range_b = mid..end;
    s.join(
        |s| find_primes_in_range(s, primes, range_a, insert_buffer),
        |s| find_primes_in_range(s, primes, range_b, insert_buffer),
    );
}
