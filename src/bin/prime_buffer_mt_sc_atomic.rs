use rust_prime::{THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT, check_primality};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

fn main() {
    let primes = Arc::new(RwLock::new(vec![2usize]));
    let mut insert_buffer = Vec::new();
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = primes.read().unwrap().last().unwrap().pow(2);
    let mut check_buffer = VecDeque::with_capacity(TOTAL_WORK_LIMIT);

    // Channels for between buffer and worker threads.
    // The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));

    for _ in 0..*THREAD_COUNT {
        let check_rx = check_rx.clone();
        let primes = primes.clone();
        thread::spawn(move || worker(&check_rx, &primes));
    }

    loop {
        while test <= test_limit.min(test_halt) && check_buffer.len() < TOTAL_WORK_LIMIT {
            let result_a = Arc::new(AtomicUsize::new(0));
            check_tx.send((result_a.clone(), test)).unwrap();
            check_buffer.push_back(result_a);
            test += 2;
        }
        thread::yield_now();

        let mut ran = false;
        // This used to require a hack before non-lexical lifetimes
        // https://stackoverflow.com/questions/50251487/what-are-non-lexical-lifetimes
        while let Some(result_a) = check_buffer.front() {
            let result = result_a.load(Ordering::Relaxed);
            if result == 0 {
                // Only try again if we didn't get a new prime added to the list.
                if ran {
                    break;
                }
                thread::yield_now();
                continue;
            } else if result > 1 {
                insert_buffer.push(result);
                println!("{result:?}");
            }
            ran = true;
            check_buffer.pop_front();
        }

        if test >= test_limit.min(test_halt) && !insert_buffer.is_empty() {
            let mut primes_w = primes.write().unwrap();
            primes_w.append(&mut insert_buffer);
            test_limit = primes_w.last().unwrap().pow(2);
        }

        if test >= test_halt && check_buffer.is_empty() {
            break;
        }
    }
}

type Work = (Arc<AtomicUsize>, usize); // Just for clippy...

fn worker(check_rx: &Arc<Mutex<mpsc::Receiver<Work>>>, primes: &Arc<RwLock<Vec<usize>>>) {
    let mut work = Vec::with_capacity(THREAD_WORK_LIMIT);
    loop {
        // Give main() time to fill the channel.
        thread::yield_now();

        work.extend(check_rx.lock().unwrap().try_iter().take(THREAD_WORK_LIMIT));

        let primes_guard = primes.read().unwrap();
        for (result_a, test) in work.drain(..) {
            let is_prime = check_primality(test, &primes_guard);
            result_a.store(if is_prime { test } else { 1 }, Ordering::Relaxed);
        }
    }
}
