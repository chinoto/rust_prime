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

## Benchmark

```sh
./get_timings.sh 1e7
```

| (Kernel + User) / Real Seconds | CPU % | Binary                    |
| -----------------------------: | ----: | :------------------------ |
|           (0.12 + 0.64) / 0.76 |  100% | prime                     |
|           (0.12 + 0.64) / 0.76 |  100% | prime_buffer              |
|           (2.93 + 5.13) / 0.60 | 1342% | prime_buffer_mt_sc        |
|           (5.12 + 5.07) / 0.87 | 1159% | prime_buffer_mt_sc_atomic |
|           (3.52 + 4.69) / 0.62 | 1322% | prime_buffer_mt_sc_cow    |
|         (65.25 + 60.45) / 8.19 | 1534% | prime_buffer_mt_ptc       |
|           (0.12 + 1.15) / 0.39 |  320% | prime_mt_range_map        |
|           (0.16 + 1.03) / 0.41 |  291% | prime_mt_rayon            |

## TODO

- Break this into main and worker pieces that can be compiled to WASM and spun up in Web Workers.

[VecDeque]: https://doc.rust-lang.org/1.12.1/std/collections/struct.VecDeque.html
[Learning Rust With Entirely Too Many Linked Lists]: http://cglab.ca/~abeinges/blah/too-many-lists/book/
[The ryg blog: Ring buffers and queues]: https://fgiesen.wordpress.com/2010/12/14/ring-buffers-and-queues/
