use rust_prime::{Queue, TOTAL_WORK_LIMIT, check_primality};

fn main() {
    let mut primes: Vec<usize> = vec![2];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    // This is the last number that can be checked before the buffer needs to be drained into the primes list.
    let mut test_limit = primes.last().unwrap().pow(2);
    // This is a ring buffer that hold all the primes found. Make it huge to avoid flushing often.
    let mut check_buffer = Queue::<TOTAL_WORK_LIMIT>::new();

    while test < test_halt {
        if check_primality(test, &primes) {
            check_buffer.push(test);
        }
        test += 2;
        if
        // If the next cell is the read cell, flush the contents now, otherwise the buffer breaks.
        check_buffer.is_full()
			// If the next test is past the limit and the buffer has content, flush.
			|| test>=test_limit && !check_buffer.is_empty()
			// If we're halting, flush now because there won't be another chance.
			|| test>=test_halt
        {
            // The buffer is empty when these are equal.
            while let Some(prime) = check_buffer.try_shift_prime() {
                primes.push(prime);
                println!("{prime}");
            }
            // Now that the buffer is drained into the primes list, a new limit can be set.
            test_limit = primes.last().unwrap().pow(2);
        }
    }
}
