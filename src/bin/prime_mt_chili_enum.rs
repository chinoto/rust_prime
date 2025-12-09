use chili::Scope;
use rust_prime::{THREAD_WORK_LIMIT, check_primality};
use std::{iter::Chain, ops::Range};

fn main() {
    let mut primes = vec![2usize];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut scope = Scope::global();

    while test < test_halt {
        let test_limit = primes.last().unwrap().pow(2).min(test_halt);
        let range = test..test_limit;
        let new_primes_iter = find_primes_in_range(&mut scope, &primes, range).flatten();
        primes.extend(new_primes_iter.inspect(|p| println!("{p}")));
        test = test_limit;
    }
}

fn find_primes_in_range(s: &mut Scope, primes: &[usize], range: Range<usize>) -> PrimeChiliTree {
    if range.len() < THREAD_WORK_LIMIT {
        let new_primes = range
            .filter(|&test| check_primality(test, primes))
            .collect::<Vec<usize>>();
        return PrimeChiliTree::base(new_primes);
    }
    let Range { start, end } = range;
    let mid = start.midpoint(end);
    let range_a = start..mid;
    let range_b = mid..end;
    let (branch_a, branch_b) = s.join(
        |s| find_primes_in_range(s, primes, range_a),
        |s| find_primes_in_range(s, primes, range_b),
    );
    PrimeChiliTree::recurse(branch_a, branch_b)
}

enum PrimeChiliTree {
    Base(Option<Vec<usize>>),
    Recurse(Box<Chain<PrimeChiliTree, PrimeChiliTree>>),
}

impl PrimeChiliTree {
    fn base(a: Vec<usize>) -> Self {
        Self::Base(Some(a))
    }
    fn recurse(a: Self, b: Self) -> Self {
        Self::Recurse(Box::new(a.chain(b)))
    }
}

impl Iterator for PrimeChiliTree {
    // More efficient to return Vec, then `.flatten()`, otherwise the tree has
    // to be traversed for every usize.
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PrimeChiliTree::Recurse(chain) => chain.next(),
            PrimeChiliTree::Base(items) => items.take(),
        }
    }
}
