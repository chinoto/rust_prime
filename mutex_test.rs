use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::thread;

fn main() {
    let a = Mutex::new(1);
    let t = unsafe { thread::spawn(|| *a.lock().unwrap() *= 2) };
    println!("{:?}", a.lock().unwrap());
    t.join().unwrap();
    println!("{:?}", a.lock().unwrap());
}
