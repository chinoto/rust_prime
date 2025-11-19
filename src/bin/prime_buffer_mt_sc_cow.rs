use rust_prime::{Queue, THREAD_COUNT, THREAD_WORK_LIMIT, TOTAL_WORK_LIMIT, check_primality};
use std::borrow::Cow;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

fn main() {
    let mut primes = Cow::from(vec![2usize]);
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
        while test <= test_limit.min(test_halt) && !check_buffer.is_full() {
            check_tx.send((check_buffer.push(0), test)).unwrap();
            test += 2;
        }
        thread::yield_now();

        for (index, result) in result_rx.try_iter() {
            check_buffer.update(index, result);
        }

        while let Some(result) = check_buffer.try_shift_prime() {
            primes.to_mut().push(result);
            println!("{result:?}");
        }

        if test >= test_limit {
            primes.clone_into(&mut *shared_primes.write().unwrap());
            test_limit = primes.last().unwrap().pow(2);
        }

        if test >= test_halt && check_buffer.is_empty() {
            break;
        }
    }
}

fn worker(
    check_rx: &Arc<Mutex<mpsc::Receiver<(usize, usize)>>>,
    result_tx: &mpsc::Sender<(usize, usize)>,
    primes_shared: &Arc<RwLock<Cow<[usize]>>>,
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
