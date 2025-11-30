use rust_prime::{Queue, THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT, check_primality};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

fn main() {
    let mut primes = Arc::new(vec![2usize]);
    let shared_primes = Arc::new(RwLock::new(primes.clone()));
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = primes.last().unwrap().pow(2);

    let mut check_buffer = Queue::<TOTAL_WORK_LIMIT>::new();

    // Channels for between buffer and worker threads.
    // The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));
    let (result_tx, result_rx) = mpsc::channel();

    for _ in 0..*THREAD_COUNT {
        let check_rx = check_rx.clone();
        let result_tx = result_tx.clone();
        let shared_primes = shared_primes.clone();
        thread::spawn(move || worker(&check_rx, &result_tx, &shared_primes));
    }

    loop {
        while test < test_limit && !check_buffer.is_full() {
            check_tx.send((check_buffer.push(0), test)).unwrap();
            test += 2;
        }
        thread::yield_now();

        for (index, result) in result_rx.try_iter() {
            check_buffer.update(index, result);
        }

        // Where Copy-on-Write takes place.
        let primes_inner: &mut Vec<usize> = Arc::make_mut(&mut primes);
        while let Some(result) = check_buffer.try_shift_prime() {
            primes_inner.push(result);
            println!("{result}");
        }

        if test >= test_limit {
            *shared_primes.write().unwrap() = primes.clone();
            test_limit = primes.last().unwrap().pow(2).min(test_halt);
        }

        if test >= test_halt && check_buffer.is_empty() {
            break;
        }
    }
}

fn worker(
    check_rx: &Arc<Mutex<mpsc::Receiver<(usize, usize)>>>,
    result_tx: &mpsc::Sender<(usize, usize)>,
    primes_shared: &Arc<RwLock<Arc<Vec<usize>>>>,
) {
    let mut work: Vec<(usize, usize)> = Vec::with_capacity(THREAD_WORK_LIMIT);
    let mut primes = primes_shared.read().unwrap().clone();
    let mut last = *primes.last().unwrap();
    'work: loop {
        // Give main() time to fill the channel.
        thread::yield_now();

        work.extend(check_rx.lock().unwrap().try_iter().take(THREAD_WORK_LIMIT));

        for (index, test) in work.drain(..) {
            let max = test.isqrt();
            while last < max {
                thread::yield_now();
                primes_shared.read().unwrap().clone_into(&mut primes);
                last = *primes.last().unwrap();
            }
            let is_prime = check_primality(test, &primes);
            if result_tx
                .send((index, if is_prime { test } else { 1 }))
                .is_err()
            {
                break 'work;
            }
        }
    }
}
