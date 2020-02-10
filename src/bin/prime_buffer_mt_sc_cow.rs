use std::borrow::Cow;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::{env, thread};

const CHECK_BUFFER_SIZE: usize = 2000;
const MAIN_CHECK_SIZE: usize = 16;
const WORKER_CAP: usize = 100;

fn main() {
    let mut primes = Cow::from(vec![2u64]);
    let shared_primes = Arc::new(RwLock::new(primes.clone()));
    let mut test = 3;
    let test_halt = env::args()
        .nth(1)
        .expect("Provide a limit.")
        .parse::<f64>()
        .expect("Failed to parse limit") as u64;
    let mut test_limit = primes.last().unwrap().pow(2);

    let mut buffer = [1; CHECK_BUFFER_SIZE];
    let mut buffer_read = 0;
    let mut buffer_write = 0;

    //Channels for between buffer and worker threads.
    //The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));
    let (result_tx, result_rx) = mpsc::channel();

    for _ in 0..4 {
        let check_rx = check_rx.clone();
        let result_tx = result_tx.clone();
        let shared_primes = shared_primes.clone();
        thread::spawn(move || worker(check_rx, result_tx, shared_primes));
    }

    loop {
        while test <= test_limit
            && test <= test_halt
            && (buffer_write + 1) % CHECK_BUFFER_SIZE != buffer_read
        {
            check_tx.send((buffer_write, test)).unwrap();
            buffer_write = (buffer_write + 1) % CHECK_BUFFER_SIZE;
            loop {
                test += 2;
                if primes
                    .iter()
                    .take(MAIN_CHECK_SIZE)
                    .all(|&i| (test % i) != 0)
                {
                    break;
                }
            }
        }
        thread::yield_now();

        while let Ok((cell, result)) = result_rx.try_recv() {
            buffer[cell] = result;
        }

        while buffer_read != buffer_write && buffer[buffer_read] != 1 {
            if buffer[buffer_read] != 0 {
                primes.to_mut().push(buffer[buffer_read]);
                println!("{:?}", buffer[buffer_read]);
            }
            buffer[buffer_read] = 1;
            buffer_read = (buffer_read + 1) % CHECK_BUFFER_SIZE;
        }

        if test >= test_limit {
            *shared_primes.write().unwrap() = primes.clone();
            test_limit = primes.last().unwrap().pow(2);
        }

        if test >= test_halt && buffer_read == buffer_write {
            break;
        }
    }
}

fn worker(
    check_rx: Arc<Mutex<mpsc::Receiver<(usize, u64)>>>,
    result_tx: mpsc::Sender<(usize, u64)>,
    primes_shared: Arc<RwLock<Cow<[u64]>>>,
) {
    let mut work: Vec<(usize, u64)> = Vec::with_capacity(WORKER_CAP);
    let mut primes = primes_shared.read().unwrap().clone();
    let mut last = *primes.last().unwrap();
    'work: loop {
        //Give main() time to fill the channel.
        thread::yield_now();

        let check_rx = check_rx.lock().unwrap();
        while let Ok(recv) = check_rx.try_recv() {
            work.push(recv);
            if work.len() >= WORKER_CAP {
                break;
            }
        }
        drop(check_rx);

        for (cell, test) in work.drain(..) {
            let max = (test as f64).sqrt() as u64;
            while last < max {
                thread::yield_now();
                primes = primes_shared.read().unwrap().clone();
                last = *primes.last().unwrap();
            }
            let is_prime = primes
                .iter()
                .skip(MAIN_CHECK_SIZE)
                .take_while(|&&i| i <= max)
                //If test is not divisible by all values of i, it is prime.
                .all(|&i| (test % i) != 0);
            if result_tx
                .send((cell, if is_prime { test } else { 0 }))
                .is_err()
            {
                break 'work;
            }
        }
    }
}
