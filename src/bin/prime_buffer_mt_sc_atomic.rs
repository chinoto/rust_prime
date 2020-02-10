#![feature(integer_atomics)]
use std::collections::VecDeque;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::{env, thread};

use std::sync::atomic::{AtomicU64, Ordering};

const CHECK_BUFFER_SIZE: usize = 2000;
const MAIN_CHECK_SIZE: usize = 16;
const WORKER_CAP: usize = 100;

fn main() {
    let primes = Arc::new(RwLock::new(vec![2u64]));
    let mut insert_buffer = Vec::new();
    let mut test = 3;
    let test_halt = env::args()
        .nth(1)
        .expect("Provide a limit.")
        .parse::<f64>()
        .expect("Failed to parse limit") as u64;
    let mut test_limit = primes.read().unwrap().last().unwrap().pow(2);
    let mut check_buffer = VecDeque::with_capacity(CHECK_BUFFER_SIZE);

    //Channels for between buffer and worker threads.
    //The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));

    for _ in 0..4 {
        let check_rx = check_rx.clone();
        let primes = primes.clone();
        thread::spawn(|| worker(check_rx, primes));
    }

    loop {
        while test <= test_limit && test <= test_halt && check_buffer.len() < CHECK_BUFFER_SIZE {
            let result_a = Arc::new(AtomicU64::new(1));
            check_tx.send((result_a.clone(), test)).unwrap();
            check_buffer.push_back(result_a);
            loop {
                test += 2;
                if primes
                    .read()
                    .unwrap()
                    .iter()
                    .take(MAIN_CHECK_SIZE)
                    .all(|&i| (test % i) != 0)
                {
                    break;
                }
            }
        }
        thread::yield_now();

        let mut ran = false;
        // This used to require a hack before non-lexical lifetimes
        // https://stackoverflow.com/questions/50251487/what-are-non-lexical-lifetimes
        while let Some(result_a) = check_buffer.front() {
            let result = result_a.load(Ordering::Relaxed);
            if result == 1 {
                //Only try again if we didn't get a new prime added to the list.
                if ran {
                    break;
                }
                thread::yield_now();
                continue;
            }
            if result != 0 {
                insert_buffer.push(result);
                println!("{:?}", result);
            }
            ran = true;
            check_buffer.pop_front();
        }

        if (test >= test_halt || test >= test_limit) && !insert_buffer.is_empty() {
            let mut primes_w = primes.write().unwrap();
            primes_w.append(&mut insert_buffer);
            test_limit = primes_w.last().unwrap().pow(2);
        }

        if test >= test_halt && check_buffer.is_empty() {
            break;
        }
    }
}

type Work = (Arc<AtomicU64>, u64); // Just for clippy...

fn worker(check_rx: Arc<Mutex<mpsc::Receiver<Work>>>, primes: Arc<RwLock<Vec<u64>>>) {
    let mut len = 0;
    let mut work = Vec::with_capacity(WORKER_CAP);
    loop {
        //Give main() time to fill the channel.
        thread::yield_now();

        let check_rx = check_rx.lock().unwrap();
        while let Ok(recv) = check_rx.try_recv() {
            work.push(recv);
            len += 1;
            if len >= WORKER_CAP {
                break;
            }
        }
        len = 0;
        drop(check_rx);

        for (result_a, test) in work.drain(..) {
            let max = (test as f64).sqrt() as u64;
            let is_prime = primes
                .read()
                .unwrap()
                .iter()
                .skip(MAIN_CHECK_SIZE)
                .take_while(|&&i| i <= max)
                .all(|&i| (test % i) != 0);
            result_a.store(if is_prime { test } else { 0 }, Ordering::Relaxed);
        }
    }
}
