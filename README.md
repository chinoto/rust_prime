# Rusty Prime Generator

**Now with more oxidizers!**

My first task in learning a language is to make a prime generator because it's pretty simple, but more complex than "Hello World". This time I did it in a language capable of multi-threading, so I had to try it.

My big problem for awhile was figuring out how to keep the output of the workers in order so that the prime list is sequential and not missing any primes when updated, then I remembered reading _[The ryg blog: Ring buffers and queues]_, which solved the issue quite nicely (though I'm probably not using it in a typical way in _prime_buffer_mt_ptc.rs_).

After that it was mostly just banging my head against the wall trying to figure out the intricacies of Rust, such as using `&*` to dereference the underlying value of a (read|write)guard and then getting a reference to it (rather than the guard). Thanks to mbrubeck of ##rust on [freenode](http://freenode.net/) for helping with that, I couldn't find anything at all via search engine.

## Files

It is best to read _prime.rs_, _prime_buffer.rs_, and then _prime_buffer_mt_ptc.rs_ because I only comment on new aspects to the code in each file.

I tried to keep my file names short, these are what the abbreviations mean:

- mt = multi-threaded
- ptc = per thread channel
- sc = single channel (probably came after ptc)

## TODO

- Break this into main and worker pieces that can be compiled to WASM and spun up in Web Workers.

[VecDeque]: https://doc.rust-lang.org/1.12.1/std/collections/struct.VecDeque.html
[Learning Rust With Entirely Too Many Linked Lists]: http://cglab.ca/~abeinges/blah/too-many-lists/book/
[The ryg blog: Ring buffers and queues]: https://fgiesen.wordpress.com/2010/12/14/ring-buffers-and-queues/
