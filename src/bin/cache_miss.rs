use std::time::{Duration, Instant};

const N: usize = 2048;

fn time(target: fn() -> Duration) {
    let start = Instant::now();
    let calculation = target();
    let end = Instant::now();
    let duration = end - start;
    println!("calculation took: {}", calculation.as_nanos());
    println!(" allocation took: {}", (duration - calculation).as_nanos());
    println!("      total took: {}", duration.as_nanos());
}

fn cache_friendly() -> Duration {
    #[allow(clippy::useless_conversion)]
    let mut matrix: Vec<Vec<usize>> = (0..N)
        .into_iter()
        .map(|_| Vec::from_iter((0..N).into_iter()))
        .collect();

    let start = Instant::now();
    (0..N).for_each(|row| {
        (0..N).for_each(|col| {
            matrix[row][col] = row + col;
        });
    });
    let end = Instant::now();
    end - start
}

fn cache_infriendly() -> Duration {
    #[allow(clippy::useless_conversion)]
    let mut matrix: Vec<Vec<usize>> = (0..N)
        .into_iter()
        .map(|_| Vec::from_iter((0..N).into_iter()))
        .collect();

    let start = Instant::now();
    (0..N).for_each(|col| {
        (0..N).for_each(|row| {
            matrix[row][col] = row + col;
        });
    });
    let end = Instant::now();
    end - start
}

fn main() {
    time(cache_friendly);
    time(cache_infriendly);
}
