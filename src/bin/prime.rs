use std::env;

fn main() {
	//This is the list of primes found, which are used to
	//determine if the current test value is prime as well.
	let mut primes: Vec<u64>=vec![2];
	//The current value to be tested for primality.
	let mut test=3;
	//The number at which finding primes stops.
	let test_halt=env::args()
		.nth(1).expect("Provide a limit.")
		.parse::<f64>().expect("Failed to parse limit") as u64;

	while test<test_halt {
		//Any value beyond the square root of the test is not a factor.
		let max=(test as f64).sqrt() as u64;
		if primes.iter()
			.take_while(|&&i| i<=max)
			//If test is not divisible by all values of i, it is prime.
			.all(|&i| (test%i)!=0)
		{
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
