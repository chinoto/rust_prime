use std::{sync::LazyLock, thread::available_parallelism};

pub const TOTAL_WORK_LIMIT: usize = 200_000;
pub const THREAD_WORK_LIMIT: usize = 10_000;
pub static THREAD_COUNT: LazyLock<usize> = LazyLock::new(|| {
    available_parallelism().map_or_else(
        |_| {
            let count = 16;
            println!("Failed to get available_parallelism, defaulting to {count}");
            count
        },
        |x| x.get(),
    )
});

pub fn get_halt_arg() -> usize {
    std::env::args()
        .nth(1)
        .expect("Provide a limit.")
        .parse::<f64>()
        .expect("Failed to parse limit") as usize
}

pub fn check_primality(test: usize, primes: &[usize]) -> bool {
    // The largest prime factor of a number is potentially its square root
    let max = test.isqrt();
    { primes.iter() }
        .take_while(|&&i| i <= max)
        // If test is not divisible by all values of i, it is prime.
        .all(|&i| !test.is_multiple_of(i))
}
