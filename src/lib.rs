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

#[derive(Debug)]
pub struct Queue<const N: usize> {
    data: Box<[usize; N]>,
    read: usize,
    write: usize,
}

impl<const N: usize> Queue<N> {
    pub fn new() -> Self {
        Self {
            data: Box::new([0; N]),
            read: 0,
            write: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        (self.write + 1) % N == self.read
    }

    pub fn is_empty(&self) -> bool {
        self.read == self.write
    }

    pub fn remaining(&self) -> usize {
        if self.read < self.write {
            N - self.write + self.read - 1
        } else {
            self.write - self.read - 1
        }
    }

    pub fn push(&mut self, value: usize) -> usize {
        assert!(
            !self.is_full(),
            "Attempt to get new entry when buffer is full"
        );
        let index = self.write;
        self.data[index] = value;
        self.write = (self.write + 1) % N;
        index
    }

    /// Updates the value for a given index.
    pub fn update(&mut self, index: usize, value: usize) {
        self.data[index] = value;
    }

    /// Pops a completed result from the front of the buffer if it's ready.
    pub fn try_shift_prime(&mut self) -> Option<usize> {
        loop {
            if self.is_empty() {
                return None;
            }
            let value = self.data[self.read];
            if value == 0 {
                return None; // Value not ready.
            }
            self.read = (self.read + 1) % N;
            if value == 1 {
                continue; // Value was not prime, check next index.
            }
            return Some(value); // A prime value
        }
    }
}

impl<const N: usize> Default for Queue<N> {
    fn default() -> Self {
        Self::new()
    }
}
