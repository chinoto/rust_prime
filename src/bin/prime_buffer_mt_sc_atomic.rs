#![feature(integer_atomics)]
use std::sync::{Arc, RwLock, mpsc, Mutex};
use std::thread;
use std::collections::VecDeque;

use std::sync::atomic::{AtomicU64, Ordering};

const BUFFER_SIZE: usize=81920;

fn main() {
	let primes=Arc::new(RwLock::new(vec![2]));
	let mut test=3;
	let test_halt=1e7 as u64;
	let mut test_limit=vl0(&*primes.read().unwrap());

	let mut buffer=VecDeque::with_capacity(BUFFER_SIZE);

	//Channels for between buffer and worker threads.
	//The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
	let (check_tx,check_rx)=mpsc::channel();
	let check_rx=Arc::new(Mutex::new(check_rx));

	for _ in 0..4 {
		let check_rx=check_rx.clone();
		let primes=primes.clone();
		thread::spawn(move || worker(check_rx, primes));
	}

	loop {
		while
			test<=test_limit
			&& test<=test_halt
		{
			let result_a=Arc::new(AtomicU64::new(1));
			check_tx.send((result_a.clone(),test)).unwrap();
			buffer.push_back(result_a);
			test+=2;
		}
		thread::yield_now();


		let mut primes_o=None;
		let mut ran=false;
		loop {
			//The compiler won't let me just do a `while let` and drop(result_a) before
			//buffer.pop_front(), so we have a little nesting instead to work around scope issues.
			if let Some(result_a)=buffer.front() {
				let result=result_a.load(Ordering::Relaxed);
				if result==1 {
					//Only try again if we didn't get a new prime added to the list.
					if ran {break;}
					thread::yield_now();
					continue;
				}
				if result!=0 {
					if primes_o.is_none() {primes_o=Some(primes.write().unwrap());}
					println!("{:?}", result);
					primes_o.as_mut().unwrap().push(result);
				}
				ran=true;
			}
			else {break;}
			buffer.pop_front();
		}

		if primes_o.is_some() {
			test_limit=vl0(primes_o.as_ref().unwrap());
		}

		if test>=test_halt && buffer.len()==0 {
			break;
		}
	}
}

fn worker(
	check_rx: Arc<Mutex<mpsc::Receiver<(Arc<AtomicU64>,u64)>>>,
	primes: Arc<RwLock<Vec<u64>>>
) {
	const CAP: usize=500;
	let mut len=0;
	let mut work=Vec::with_capacity(CAP);
	loop {
		//Give main() time to fill the channel.
		thread::yield_now();

		let check_rx=check_rx.lock().unwrap();
		while let Ok(recv) = check_rx.try_recv() {
			work.push(recv);
			len+=1;
			if len>=CAP {break;}
		}
		len=0;
		drop(check_rx);

		let primes=primes.read().unwrap();
		for (result_a,test) in work.drain(..) {
			let mut is_prime=true;
			let max=(test as f64).sqrt() as u64;
			for i in &*primes {
				if *i>max {break;}
				if (test%*i)==0 {
					is_prime=false;
					break;
				}
			}
			result_a.store(if is_prime {test} else {0}, Ordering::Relaxed);
		}
	}
}

fn vl0(v: &Vec<u64>) -> u64 {
	match v.last() {
		Some(x) => x.pow(2),
		None => 0
	}
}
