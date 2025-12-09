# Rusty Prime Generator

**Now with more oxidizers!**

## 2018-05-06 Initial Commit

My first task in learning a language is to make a prime generator because it's pretty simple, but more complex than "Hello World". This time I did it in a language capable of multi-threading, so I had to try it.

My big problem for awhile was figuring out how to keep the output of the workers in order so that the prime list is sequential and not missing any primes when updated, then I remembered reading _[The ryg blog: Ring buffers and queues]_, which solved the issue quite nicely (though I'm probably not using it in a typical way in `prime_buffer_mt_ptc.rs`).

After that it was mostly just banging my head against the wall trying to figure out the intricacies of Rust, such as using `&*` to dereference the underlying value of a (read|write)guard and then getting a reference to it (rather than the guard). Thanks to mbrubeck of ##rust on [freenode](http://freenode.net/) for helping with that, I couldn't find anything at all via search engine.

### Files

It is best to read `prime.rs`, `prime_buffer.rs`, and then `prime_buffer_mt_ptc.rs` because I only comment on new aspects to the code in each file.

I tried to keep my file names short, these are what the abbreviations mean:

- `mt` = multi-threaded
- `ptc` = per thread channel
- `sc` = single channel (probably came after ptc)

## 2025-11-29 Update

The results of my previous multithreading implementations were garbage mainly because they were sending each individual candidate and result over channels instead of amortizing channel costs with batching, which could be improved even more by sending the candidates to check as a range instead of collection.

With that idea in mind, I created `prime_mt_range_map.rs` based on `prime_buffer_mt_sc.rs` and used a `BTreeMap` to keep the batches in order instead of fussing with a manually implemented ring-buffer/queue.

## 2025-12-09 Update

Added more multi-threading library implementations:

- Heartbeat scheduling based (workers offer a piece of work to a shared collection every "heartbeat" to minimize synchronization overhead):
  - chili: based on the Zig library, spice; only offers a join API.
    - Several data structures were tried, but any performance difference is so marginal that criterion would be needed to choose one.
  - par-iter: Fork of rayon that can switch to using chili internally or disable parallelization.
  - forte: general purpose thread pool with spawn and join APIs.
- orx-parallel: Utilizes work-stealing like rayon.

par-iter beats every multi-threading implementation in overall CPU time, while only being slightly slower in real time than the fastest implementation. Since par-iter uses chili internally, I'm not sure why my direct usage of chili is faster in real time, yet uses more CPU time.

## Benchmark

```sh
./get_timings.sh 1e7
```

| (Kernel + User) / Real Seconds | CPU % | Binary                    |
| -----------------------------: | ----: | :------------------------ |
|           (0.13 + 0.61) / 0.74 |  100% | prime                     |
|           (0.11 + 0.62) / 0.74 |   99% | prime_buffer              |
|           (0.13 + 1.13) / 0.38 |  326% | prime_mt_range_map        |
|           (0.13 + 1.00) / 0.39 |  290% | prime_mt_orx              |
|           (0.15 + 1.01) / 0.39 |  293% | prime_mt_forte_ll         |
|           (0.12 + 1.07) / 0.41 |  290% | prime_mt_rayon            |
|           (0.13 + 0.88) / 0.46 |  221% | prime_mt_chili_ll         |
|           (0.13 + 0.90) / 0.46 |  225% | prime_mt_chili_mutex      |
|           (0.15 + 0.87) / 0.46 |  221% | prime_mt_chili_enum       |
|           (0.15 + 0.89) / 0.46 |  227% | prime_mt_chili_vec        |
|           (0.14 + 0.69) / 0.49 |  169% | prime_mt_par_iter         |
|           (2.99 + 5.06) / 0.58 | 1376% | prime_buffer_mt_sc        |
|           (3.52 + 4.83) / 0.60 | 1377% | prime_buffer_mt_sc_cow    |
|           (5.23 + 4.82) / 0.82 | 1213% | prime_buffer_mt_sc_atomic |
|         (57.93 + 55.21) / 7.12 | 1588% | prime_buffer_mt_ptc       |

## TODO

- Break this into main and worker pieces that can be compiled to WASM and spun up in Web Workers.

[VecDeque]: https://doc.rust-lang.org/1.12.1/std/collections/struct.VecDeque.html
[Learning Rust With Entirely Too Many Linked Lists]: http://cglab.ca/~abeinges/blah/too-many-lists/book/
[The ryg blog: Ring buffers and queues]: https://fgiesen.wordpress.com/2010/12/14/ring-buffers-and-queues/
