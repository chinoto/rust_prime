use std::sync::{Arc, RwLock, mpsc, Mutex};
use std::thread;

const BUFFER_SIZE: usize=81920;

fn main() {
	let primes=Arc::new(RwLock::new(vec![2]));
	let mut test=3;
	let test_halt=1e7 as u64;
	let mut test_limit=vl0(&*primes.read().unwrap());

	let mut buffer=[1;BUFFER_SIZE];
	let mut buffer_read=0;
	let mut buffer_write=0;

	//Channels for between buffer and worker threads.
	//The workers share the check receiver using a mutex, would be better to use a proper mpmc instead.
	let (check_tx,check_rx)=mpsc::channel();
	let check_rx=Arc::new(Mutex::new(check_rx));
	let (result_tx,result_rx)=mpsc::channel();

	for _ in 0..4 {
		let check_rx=check_rx.clone();
		let result_tx=result_tx.clone();
		let primes=primes.clone();
		thread::spawn(move || worker(check_rx, result_tx, primes));
	}

	loop {
		while
			test<=test_limit
			&& test<=test_halt
			&& (buffer_write+1)%BUFFER_SIZE!=buffer_read
		{
			check_tx.send((buffer_write,test)).unwrap();
			test+=2;
			buffer_write=(buffer_write+1)%BUFFER_SIZE;
		}
		thread::yield_now();


		let mut primes_o=None;
		while let Ok((cell,result))=result_rx.try_recv() {
			buffer[cell]=result;
			if buffer_read==cell {
				while buffer[buffer_read]!=1 {
					if buffer[buffer_read]!=0 {
						if primes_o.is_none() {primes_o=Some(primes.write().unwrap());}
						println!("{:?}", buffer[buffer_read]);
						primes_o.as_mut().unwrap().push(buffer[buffer_read]);
					}
					buffer[buffer_read]=1;
					buffer_read=(buffer_read+1)%BUFFER_SIZE;
				}
			}
		}

		if primes_o.is_some() {
			test_limit=vl0(primes_o.as_ref().unwrap());
		}

		if test>=test_halt && buffer_read==buffer_write {
			break;
		}
	}
}

fn worker(
	check_rx: Arc<Mutex<mpsc::Receiver<(usize,u64)>>>,
	result_tx: mpsc::Sender<(usize,u64)>,
	primes: Arc<RwLock<Vec<u64>>>
) {
	const CAP: usize=500;
	let mut len=0;
	let mut work:[(usize,u64); CAP]=[(0,0); CAP];
	loop {
		//Give main() time to fill the channel.
		thread::yield_now();

		let check_rx=check_rx.lock().unwrap();
		while let Ok(recv) = check_rx.try_recv() {
			//work.push(recv);
			work[len]=recv;
			len+=1;
			if len>=CAP {break;}
		}
		drop(check_rx);

		//Why is this slower?! D;
		//if work.len()==0 {continue;}

		let primes=primes.read().unwrap();
		for &(cell,test) in work.iter().take(len) {
			let mut is_prime=true;
			let max=(test as f64).sqrt() as u64;
			for i in &*primes {
				if *i>max {break;}
				if (test%*i)==0 {
					is_prime=false;
					break;
				}
			}
			result_tx.send((cell,if is_prime {test} else {0})).unwrap();
		}
		len=0;
	}
}

fn vl0(v: &Vec<u64>) -> u64 {
	match v.last() {
		Some(x) => x.pow(2),
		None => 0
	}
}
