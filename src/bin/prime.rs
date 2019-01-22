fn main() {
	//This is the list of primes found, which are used to
	//determine if the current test value is prime as well.
	let mut primes: Vec<u64>=vec![2];
	//The current value to be tested for primality.
	let mut test=3;
	//The number at which finding primes stops.
	let test_halt=1e7 as u64;

	while test<test_halt {
		//Until proven otherwise, assume the value is prime.
		let mut is_prime=true;
		//Any value beyond the square root of the test is not a factor.
		let max=(test as f64).sqrt() as u64;
		for i in &primes {
			if *i>max {break;}
			//If test is divisible by i, it is not prime.
			if (test%*i)==0 {
				is_prime=false;
				break;
			}
		}
		if is_prime {
			primes.push(test);
			println!("{}", test);
		}
		/*
		Skip every number divisible by 2, waste of processing time.
		This could go further and alternate between adding 2 and 4 to avoid checking numbers
		divisible by 3, but really, even skipping even numbers doesn't make a significant
		difference because any test that has factors greater than 50 will cost at least 14 times
		as much as a test that is divisible by 2.
		*/
		test+=2;
	}
}
