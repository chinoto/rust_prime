use rust_prime::{Queue, THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT, check_primality};
use std::sync::{Arc, RwLock, mpsc};
use std::thread;

fn main() {
    let primes = Arc::new(RwLock::new(vec![2usize]));
    let mut insert_buffer = Vec::new();
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = (*primes.read().unwrap().last().unwrap()).pow(2);
    let mut check_buffer = Queue::<TOTAL_WORK_LIMIT>::new();

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
                if test >= test_limit || !check_buffer.is_empty() {
                    break 'pumper;
                }

                // Send the number to be checked as well as the cell number so that the main thread
                // knows where to put the result once the worker has submitted its work.
                check_tx.send((check_buffer.push(0), test)).unwrap();
                test += 2;
            }
        }
        thread::yield_now();

        for (cell, test) in result_rx.try_iter() {
            check_buffer.update(cell, test);
        }

        while let Some(prime) = check_buffer.try_shift_prime() {
            insert_buffer.push(prime);
            println!("{prime:?}");
        }

        if test >= test_limit || insert_buffer.len() >= 100 {
            let mut primes_w = primes.write().unwrap();
            primes_w.append(&mut insert_buffer);
            test_limit = primes_w.last().unwrap().pow(2).min(test_halt);
        }

        if test >= test_halt && check_buffer.is_empty() && insert_buffer.is_empty() {
            break;
        }
    }
}

fn worker(
    check_rx: &mpsc::Receiver<(usize, usize)>,
    result_tx: &mpsc::Sender<(usize, usize)>,
    primes: &Arc<RwLock<Vec<usize>>>,
) {
    let mut work = Vec::with_capacity(THREAD_WORK_LIMIT);
    loop {
        // Give main() time to fill the channel.
        thread::yield_now();
        work.extend(check_rx.try_iter().take(THREAD_WORK_LIMIT));
        let primes_guard = primes.read().unwrap();
        for (cell, test) in work.drain(..) {
            let is_prime = check_primality(test, &primes_guard);
            if result_tx
                .send((cell, if is_prime { test } else { 1 }))
                .is_err()
            {
                return;
            }
        }
    }
}
