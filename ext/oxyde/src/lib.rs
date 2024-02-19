mod price_calendar;
mod yield_index_builder;

use crate::price_calendar::*;
use magnus::{function, method, prelude::*, Error, Ruby};
use once_cell::sync::OnceCell;

static INSTANCE: OnceCell<yield_index_builder::YieldIndices> = OnceCell::new();

struct PriceCalendarAccessor {}

impl PriceCalendarAccessor {
    pub fn global() -> &'static yield_index_builder::YieldIndices {
        INSTANCE.get().expect("Yield index is not initialized")
    }

    pub fn build() {
        let index = yield_index_builder::build();
        INSTANCE
            .set(index)
            .expect("Couldn't set global calendar index");
    }
}

fn hello(subject: String) -> String {
    format!("Hello from Rust, {subject}!")
}

fn build_index() {
    PriceCalendarAccessor::build();
}

// For some reason required as per https://github.com/matsadler/magnus/issues/66
unsafe impl magnus::IntoValueFromNative for PeriodPrice {}

fn search_index(
    item_id: yield_index_builder::Identifier,
    begin: Timestamp,
    end: Timestamp,
) -> Vec<PeriodPrice> {
    PriceCalendarAccessor::global()
        .get(&item_id)
        .expect("Item ID missing")
        .proportional_period_prices(begin, end)
        .to_vec()
}

fn single_price(
    item_id: yield_index_builder::Identifier,
    begin: Timestamp,
    end: Timestamp,
) -> Price {
    PriceCalendarAccessor::global()
        .get(&item_id)
        .expect("Item ID missing")
        .proportional_sum(begin, end)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Oxyde")?;
    module.define_singleton_method("hello", function!(hello, 1))?;
    module.define_singleton_method("build_index", function!(build_index, 0))?;
    module.define_singleton_method("search_index", function!(search_index, 3))?;
    module.define_singleton_method("single_price", function!(single_price, 3))?;

    let rb_yield_period = ruby.define_class("PeriodPrice", ruby.class_object())?;
    rb_yield_period.define_method("begin", method!(PeriodPrice::begin, 0))?;
    rb_yield_period.define_method("price", method!(PeriodPrice::price, 0))?;

    env_logger::init();

    Ok(())
}
