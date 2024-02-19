use indicatif::{ProgressBar, ProgressStyle};
use mysql::prelude::*;
use mysql::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
use jemalloc_ctl::{epoch, stats};
use pretty_bytes::converter::convert;

use crate::price_calendar::*;

pub type Identifier = u32;

pub type YieldIndices = HashMap<Identifier, PriceCalendar>;

#[allow(dead_code)]
pub fn build() -> YieldIndices {
    let url: String = env::var("MYSQL_URL").expect("Please set the MYSQL_URL environment variable");
    let pool = Pool::new(Opts::from_url(&url).unwrap()).unwrap();
    let mut conn = pool.get_conn().unwrap();

    mem_debug("   Memory before index build");

    conn.query_drop("SET time_zone = '+00:00';")
        .expect("Couldn't set UTC time zone");

    let pv_count: u64 = conn
        .query_first("SELECT COUNT(*) FROM price_variations")
        .unwrap()
        .unwrap();

    let mut item_indices = HashMap::new();

    let pb = ProgressBar::new(pv_count);
    pb.set_style(
        ProgressStyle::with_template(
            "📈 Building index [{elapsed_precise}] {wide_bar:.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap(),
    );

    let mut count = 0;

    conn.query_iter(
        "SELECT car_id, UNIX_TIMESTAMP(date), price FROM price_variations ORDER BY `date` ASC",
    )
    .unwrap()
    .for_each(|row| {
        let pv: (Identifier, Timestamp, Price) = from_row(row.unwrap());

        let index = match item_indices.entry(pv.0) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(PriceCalendar { data: vec![] }),
        };

        index.add(PeriodPrice {
            begin: pv.1,
            price: pv.2,
        });

        count = count + 1;

        if count % 1000 == 0 {
            pb.inc(1000);
        }
    });

    pb.finish_with_message("done");

    mem_debug("   Memory after index build");

    println!("✅ Pricing indexed for {} items", item_indices.len());

    item_indices
}

pub fn mem_debug(msg: &str) {
    // many statistics are cached and only updated when the epoch is advanced.
    epoch::advance().unwrap();

    println!(
        "{}: {} allocated / {} resident",
        msg,
        convert(stats::allocated::read().unwrap() as f64),
        convert(stats::resident::read().unwrap() as f64)
    );
}
