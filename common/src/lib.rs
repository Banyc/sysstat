use std::time::Duration;

use strict_num::FiniteF64;

pub mod value;

pub fn change_per_second(m: i128, n: i128, p: Duration) -> Option<FiniteF64> {
    let change = (n - m) as f64 / p.as_secs_f64();
    FiniteF64::new(change)
}
