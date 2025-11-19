use rust_prime::check_primality;

fn main() {
    // This is the list of primes found, which are used to
    // determine if the current test value is prime as well.
    let mut primes: Vec<usize> = vec![2];
    // The current value to be tested for primality.
    let mut test = 3;
    // The number at which finding primes stops.
    let test_halt = rust_prime::get_halt_arg();

    while test < test_halt {
        if check_primality(test, &primes) {
            primes.push(test);
            println!("{test}");
        }
        /*
        Skip every number divisible by 2, waste of processing time.
        This could go further and alternate between adding 2 and 4 to avoid checking numbers
        divisible by 3, but really, even skipping even numbers doesn't make a significant
        difference because any test that has factors greater than 50 will cost at least 14 times
        as much as a test that is divisible by 2.
        */
        test += 2;
    }
}
