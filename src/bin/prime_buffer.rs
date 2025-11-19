use rust_prime::TOTAL_WORK_LIMIT;

fn main() {
    let mut primes: Vec<usize> = vec![2];
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    // This is the last number that can be checked before the buffer needs to be drained into the primes list.
    let mut test_limit = primes.last().unwrap().pow(2);
    // This is a ring buffer that hold all the primes found. Make it huge to avoid flushing often.
    let mut buffer = [0; TOTAL_WORK_LIMIT];
    let mut buffer_read = 0;
    let mut buffer_write = 0;

    while test < test_halt {
        let max = (test as f64).sqrt() as usize;
        if primes
            .iter()
            .take_while(|&&i| i <= max)
            // If test is not divisible by all values of i, it is prime.
            .all(|&i| (test % i) != 0)
        {
            // Write to the current cell and advance the cursor.
            buffer[buffer_write] = test;
            buffer_write = (buffer_write + 1) % TOTAL_WORK_LIMIT;
        }
        test += 2;
        if
        // If the next cell is the read cell, flush the contents now, otherwise the buffer breaks.
        (buffer_write+1)%TOTAL_WORK_LIMIT==buffer_read
			// If the next test is past the limit and the buffer has content, flush.
			|| test>=test_limit && buffer_read!=buffer_write
			// If we're halting, flush now because there won't be another chance.
			|| test>=test_halt
        {
            // The buffer is empty when these are equal.
            while buffer_read != buffer_write {
                primes.push(buffer[buffer_read]);
                println!("{}", buffer[buffer_read]);
                buffer_read = (buffer_read + 1) % TOTAL_WORK_LIMIT;
            }
            // Now that the buffer is drained into the primes list, a new limit can be set.
            test_limit = primes.last().unwrap().pow(2);
        }
    }
}
