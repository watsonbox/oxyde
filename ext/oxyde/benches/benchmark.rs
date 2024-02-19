use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "../src/price_calendar.rs"]
mod price_calendar;
use crate::price_calendar::*;

// #[path = "../src/yield_index_builder.rs"]
// mod yield_index_builder;
// use crate::yield_index_builder::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("proportional_sum", |b| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u32;

        let mut rng = rand::thread_rng();
        let item_count: usize = black_box(50_000);
        let calendar_periods: u32 = black_box(365);

        //mem_debug("Before index build");

        let mut item_indices = HashMap::with_capacity(item_count);

        for item_id in 0..item_count {
            let mut index = PriceCalendar { data: vec![] };

            for j in 0..calendar_periods {
                index.add(PeriodPrice {
                    begin: now + j * PERIOD_LENGTH,
                    price: rng.gen_range(10..255),
                })
            }

            item_indices.insert(item_id, index);
        }

        //mem_debug("After index build");

        b.iter(|| {
            // Pick an item and search start second at random
            let item_id: usize = rng.gen_range(0..item_count);
            let start_second: u32 = rng.gen_range(now..now + calendar_periods * PERIOD_LENGTH);

            // Expand to a range of 2 weeks. Performance is better the fewer periods covered.
            item_indices
                .get(&item_id)
                .expect("Item ID missing")
                .proportional_sum(
                    start_second - 7 * PERIOD_LENGTH,
                    start_second + 7 * PERIOD_LENGTH,
                );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
