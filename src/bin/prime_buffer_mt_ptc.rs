use std::sync::{Arc, RwLock, mpsc};
use std::thread;

use rust_prime::{THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT};

fn main() {
    let primes = Arc::new(RwLock::new(vec![2usize]));
    let mut insert_buffer = [0; THREAD_WORK_LIMIT];
    let mut insert_buffer_len = 0;
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = (*primes.read().unwrap().last().unwrap()).pow(2);

    /*
    This time the buffer holds:
    1 for a test that is being checked for primality by a worker and should be waited on.
    0 for a test that was found not to be prime and should be skipped.
    Any number greater than 1 is a prime and should be added to the prime list.
    */
    let mut buffer = [1; TOTAL_WORK_LIMIT];
    let mut buffer_read = 0;
    let mut buffer_write = 0;

    // Channel for sending data back to the main thread (this one).
    let (result_tx, result_rx) = mpsc::channel();
    let mut workers = (0..*THREAD_COUNT)
        .map(|_| {
            let (check_tx, check_rx) = mpsc::channel();
            let result_tx = result_tx.clone();
            let primes = primes.clone();
            thread::spawn(move || worker(&check_rx, &result_tx, &primes));
            check_tx
        })
        .collect::<Vec<_>>();

    loop {
        // Loop until the inner loop decides the workers have enough.
        'pumper: loop {
            for check_tx in &mut workers {
                if test >= test_halt
                    || test >= test_limit
                    || (buffer_write + 1) % TOTAL_WORK_LIMIT == buffer_read
                {
                    break 'pumper;
                }

                // Set the current cell to 1 to signify that a worker is busy with it.
                buffer[buffer_write] = 1;
                // Send the number to be checked as well as the cell number so that the main thread
                // knows where to put the result once the worker has submitted its work.
                check_tx.send((buffer_write, test)).unwrap();

                buffer_write = (buffer_write + 1) % TOTAL_WORK_LIMIT;
                test += 2;
            }
        }
        thread::yield_now();

        // Find how many tasks have been queued up, then receive that many times.
        while let Ok((cell, test)) = result_rx.try_recv() {
            buffer[cell] = test;
        }

        while buffer_read != buffer_write
            && insert_buffer_len < THREAD_WORK_LIMIT
            && buffer[buffer_read] != 1
        {
            // 0 means the number tested was not prime, skip this branch if that is the case.
            if buffer[buffer_read] != 0 {
                insert_buffer[insert_buffer_len] = buffer[buffer_read];
                insert_buffer_len += 1;
                println!("{:?}", buffer[buffer_read]);
            }
            buffer_read = (buffer_read + 1) % TOTAL_WORK_LIMIT;
        }

        if test >= test_halt || test >= test_limit || insert_buffer_len >= THREAD_WORK_LIMIT {
            let mut primes_w = primes.write().unwrap();
            primes_w.extend_from_slice(&insert_buffer[..insert_buffer_len]);
            insert_buffer_len = 0;
            test_limit = primes_w.last().unwrap().pow(2);
        }

        if test >= test_halt && buffer_read == buffer_write && insert_buffer_len == 0 {
            break;
        }
    }
}

fn worker(
    check_rx: &mpsc::Receiver<(usize, usize)>,
    result_tx: &mpsc::Sender<(usize, usize)>,
    primes: &Arc<RwLock<Vec<usize>>>,
) {
    while let Ok((cell, test)) = check_rx.recv() {
        // Get a read lock each iteration. The main thread has a chance to get a write lock between
        // each iteration while attempting to receive work.
        let max = (test as f64).sqrt() as usize;
        let is_prime = primes
            .read()
            .unwrap()
            .iter()
            .take_while(|&&i| i <= max)
            .all(|&i| (test % i) != 0);
        if result_tx
            .send((cell, if is_prime { test } else { 0 }))
            .is_err()
        {
            break;
        }
    }
}
