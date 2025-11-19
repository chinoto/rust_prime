use rust_prime::{Queue, THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT, check_primality};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

fn main() {
    let primes = Arc::new(RwLock::new(vec![2usize]));
    let mut insert_buffer = Vec::new();
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = (*primes.read().unwrap().last().unwrap()).pow(2);

    let mut check_buffer = Queue::<TOTAL_WORK_LIMIT>::new();

    // Channels for between buffer and worker threads.
    // The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));
    let (result_tx, result_rx) = mpsc::channel();

    for _ in 0..*THREAD_COUNT {
        let check_rx = check_rx.clone();
        let result_tx = result_tx.clone();
        let primes = primes.clone();
        thread::spawn(move || worker(&check_rx, &result_tx, &primes));
    }

    loop {
        while test <= test_limit && test <= test_halt && !check_buffer.is_full() {
            check_tx.send((check_buffer.push(0), test)).unwrap();
            test += 2;
        }
        thread::yield_now();

        for (cell, result) in result_rx.try_iter() {
            check_buffer.update(cell, result);
        }

        while let Some(prime) = check_buffer.try_shift_prime() {
            insert_buffer.push(prime);
            println!("{prime:?}");
        }

        if (test >= test_limit.min(test_halt)) && !insert_buffer.is_empty() {
            let mut primes_w = primes.write().unwrap();
            primes_w.append(&mut insert_buffer);
            test_limit = primes_w.last().unwrap().pow(2);
        }

        if test >= test_halt && check_buffer.is_empty() {
            break;
        }
    }
}

fn worker(
    check_rx: &Arc<Mutex<mpsc::Receiver<(usize, usize)>>>,
    result_tx: &mpsc::Sender<(usize, usize)>,
    primes: &Arc<RwLock<Vec<usize>>>,
) {
    let mut work: Vec<(usize, usize)> = Vec::with_capacity(THREAD_WORK_LIMIT);
    'work: loop {
        // Give main() time to fill the channel.
        thread::yield_now();

        work.extend(check_rx.lock().unwrap().try_iter().take(THREAD_WORK_LIMIT));

        let primes = primes.read().unwrap();
        for (cell, test) in work.drain(..) {
            let is_prime = check_primality(test, &primes);
            if result_tx
                .send((cell, if is_prime { test } else { 1 }))
                .is_err()
            {
                break 'work;
            }
        }
    }
}
