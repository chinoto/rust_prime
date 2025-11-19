pub fn get_halt_arg() -> u64 {
    std::env::args()
        .nth(1)
        .expect("Provide a limit.")
        .parse::<f64>()
        .expect("Failed to parse limit") as u64
}
