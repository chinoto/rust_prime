use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

use rust_prime::{THREAD_COUNT, THREAD_WORK_LIMIT, check_primality};

fn main() {
    let primes = Arc::new(RwLock::new(vec![2usize]));
    let mut check_buffer = BTreeMap::<usize, Option<Vec<usize>>>::new();
    let mut test = 3;
    let test_halt = rust_prime::get_halt_arg();
    let mut test_limit = (*primes.read().unwrap().last().unwrap()).pow(2);
    // let batch_limit = TOTAL_WORK_LIMIT / THREAD_WORK_LIMIT; // 20
    // Use a larger batch limit in hopes that more cores will be used, but they aren't...
    let batch_limit = 1000;

    let (check_tx, check_rx) = mpsc::channel();
    let check_rx = Arc::new(Mutex::new(check_rx));
    let (result_tx, result_rx) = mpsc::channel();

    for _ in 0..*THREAD_COUNT {
        let check_rx = check_rx.clone();
        let result_tx = result_tx.clone();
        let primes = primes.clone();
        thread::spawn(move || worker(&check_rx, &result_tx, &primes));
    }

    while test < test_halt || !check_buffer.is_empty() {
        // Queue up work to be done by workers.
        while check_buffer.len() < batch_limit && test < test_limit {
            let end = test_limit.min(test + THREAD_WORK_LIMIT);
            let range = test..end;
            check_buffer.insert(test, None);
            check_tx.send(range).unwrap();
            test = end;
        }
        thread::yield_now();

        // Receive the batches in whatever order they come.
        // Force waiting for one batch.
        if let Ok((start, batch)) = result_rx.recv() {
            check_buffer.insert(start, Some(batch));
        }
        // Receive any other batches that happen to also be ready.
        for (start, batch) in result_rx.try_iter() {
            check_buffer.insert(start, Some(batch));
        }

        // Append batches to primes in order.
        // Only acquire a write guard if there are any batches to add and keep it until done.
        let mut write_guard_opt = None;
        while let Some(entry) = check_buffer.first_entry() {
            let Some(batch) = entry.get() else { break };
            batch.iter().for_each(|prime| println!("{prime}"));
            let write_guard = write_guard_opt.get_or_insert_with(|| primes.write().unwrap());
            write_guard.extend_from_slice(batch);
            check_buffer.pop_first().unwrap();
        }
        if let Some(write_guard) = write_guard_opt {
            test_limit = write_guard.last().unwrap().pow(2).min(test_halt);
        }
    }
}

fn worker(
    check_rx: &Arc<Mutex<Receiver<Range<usize>>>>,
    result_tx: &Sender<(usize, Vec<usize>)>,
    primes: &Arc<RwLock<Vec<usize>>>,
) {
    // The braces are necessary to drop the lock on check_rx, otherwise other threads get blocked.
    while let Ok(range) = { check_rx.lock().unwrap().recv() } {
        let primes = primes.read().unwrap();
        let start = range.start;
        let primes_in_range = range.filter(|&test| check_primality(test, &primes));
        result_tx.send((start, primes_in_range.collect())).unwrap();
    }
}
