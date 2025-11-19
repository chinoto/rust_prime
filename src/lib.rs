use std::{sync::LazyLock, thread::available_parallelism};

pub const TOTAL_WORK_LIMIT: usize = 200_000;
pub const THREAD_WORK_LIMIT: usize = 10_000;
pub static THREAD_COUNT: LazyLock<usize> = LazyLock::new(|| {
    available_parallelism().map_or_else(
        |_| {
            let count = 16;
            println!("Failed to get available_parallelism, defaulting to {count}");
            count
        },
        |x| x.get(),
    )
});

pub fn get_halt_arg() -> usize {
    std::env::args()
        .nth(1)
        .expect("Provide a limit.")
        .parse::<f64>()
        .expect("Failed to parse limit") as usize
}
