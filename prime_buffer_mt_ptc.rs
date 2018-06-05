use std::sync::{Arc, RwLock, mpsc};
use std::thread;

struct WorkerMeta {
	tx: mpsc::Sender<(usize,u64)>,
	tasks: u32
}

const BUFFER_SIZE: usize=81920;

fn main() {
	let primes=Arc::new(RwLock::new(vec![2]));
	let mut test=3;
	let test_halt=1e7 as u64;
	let mut test_limit=vl0(&*primes.read().unwrap());

	/*
	This time the buffer holds:
	1 for a test that is being checked for primality by a worker and should be waited on.
	0 for a test that was found not to be prime and should be skipped.
	Any number greater than 1 is a prime and should be added to the prime list.
	*/
	let mut buffer=[1;BUFFER_SIZE];
	let mut buffer_read=0;
	let mut buffer_write=0;

	//Each workers' transmitter and number of tasks is stored here.
	let mut workers=vec![];
	//Channel for sending data back to the main thread (this one).
	let (txm,rxm)=mpsc::channel();
	for id in 0..4 {
		let (tx,rx)=mpsc::channel();
		let txm=txm.clone();
		let primes=primes.clone();
		thread::spawn(move || worker(id,rx,txm,primes));
		workers.push(WorkerMeta {
			tx: tx,
			tasks:0
		});
	}

	loop {
		//Loop until the inner loop decides the workers have enough.
		'pumper: loop {
			for t in &mut workers {
				if
					test>=test_halt
					||test>=test_limit
					||(buffer_write+1)%BUFFER_SIZE==buffer_read
					{break 'pumper;}

				//Set the current cell to 1 to signify that a worker is busy with it.
				buffer[buffer_write]=1;
				//Send the number to be checked as well as the cell number so that the main thread
				//knows where to put the result once the worker has submitted its work.
				t.tx.send((buffer_write,test)).unwrap();

				t.tasks+=1;
				buffer_write=(buffer_write+1)%BUFFER_SIZE;
				test+=2;
			}
		}

		//Find how many tasks have been queued up, then receive that many times.
		for _ in 0..(workers.iter().map(|t|t.tasks).sum()) {
			let (id,cell,test)=rxm.recv().unwrap();
			buffer[cell]=test;
			workers[id].tasks-=1;
		}

		//Get a write lock, guaranteed to get immediately because the workers have no tasks.
		let mut primes_w=primes.write().unwrap();
		while buffer_read!=buffer_write {
			//None of the cells should be busy, this is just a sanity check.
			if buffer[buffer_read]==1 {unreachable!();}
			//0 means the number tested was not prime, skip this branch if that is the case.
			if buffer[buffer_read]!=0 {
				primes_w.push(buffer[buffer_read]);
				println!("{}", buffer[buffer_read]);
			}
			buffer_read=(buffer_read+1)%BUFFER_SIZE;
		}
		test_limit=vl0(&*primes_w);
		if test>=test_halt {break;}
	}
}

fn worker(
	id: usize,
	rx: mpsc::Receiver<(usize,u64)>,
	txm: mpsc::Sender<(usize,usize,u64)>,
	primes: Arc<RwLock<Vec<u64>>>
) {
	while let Ok((cell,test)) = rx.recv() {
		//Get a read lock each iteration. The main thread has a chance to get a write lock between
		//each iteration while attempting to receive work.
		let primes=primes.read().unwrap();
		let mut is_prime=true;
		let max=(test as f64).sqrt() as u64;
		for i in &*primes {
			if *i>max {break;}
			if (test%*i)==0 {
				is_prime=false;
				break;
			}
		}
		txm.send((id,cell,if is_prime {test} else {0})).unwrap();
	}
}

fn vl0(v: &Vec<u64>) -> u64 {
	match v.last() {
		Some(x) => x.pow(2),
		None => 0
	}
}
