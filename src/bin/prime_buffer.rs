const BUFFER_SIZE: usize=81920;

fn main() {
	let mut primes: Vec<u64>=vec![2];
	let mut test=3;
	let test_halt=1e7 as u64;
	//This is the last number that can be checked before the buffer needs to be drained into the primes list.
	let mut test_limit=vl0(&primes);
	//This is a ring buffer that hold all the primes found. Make it huge to avoid flushing often.
	let mut buffer=[0;BUFFER_SIZE];
	let mut buffer_read=0;
	let mut buffer_write=0;

	while test<test_halt {
		let mut is_prime=true;
		let max=(test as f64).sqrt() as u64;
		for i in &primes {
			if *i>max {break;}
			if (test%*i)==0 {
				is_prime=false;
				break;
			}
		}
		if is_prime {
			//Write to the current cell and advance the cursor.
			buffer[buffer_write]=test;
			buffer_write=(buffer_write+1)%BUFFER_SIZE;
		}
		test+=2;
		if
			//If the next cell is the read cell, flush the contents now, otherwise the buffer breaks.
			(buffer_write+1)%BUFFER_SIZE==buffer_read
			//If the next test is past the limit and the buffer has content, flush.
			|| test>=test_limit && buffer_read!=buffer_write
			//If we're halting, flush now because there won't be another chance.
			|| test>=test_halt
		{
			//The buffer is empty when these are equal.
			while buffer_read!=buffer_write {
				primes.push(buffer[buffer_read]);
				println!("{}", buffer[buffer_read]);
				buffer_read=(buffer_read+1)%BUFFER_SIZE;
			}
			//Now that the buffer is drained into the primes list, a new limit can be set.
			test_limit=vl0(&primes);
		}
	}
}

///Vec.last() or 0
fn vl0(v: &Vec<u64>) -> u64 {
	match v.last() {
		Some(x) => x.pow(2),
		None => 0
	}
}
