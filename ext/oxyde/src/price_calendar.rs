use std::cmp;
use std::cmp::Ordering;

pub type Price = u16; // Up to 65535 - enough for total rental prices in all currencies?
pub type Timestamp = u32; // Seconds since epoch, good for at least another 50 years

#[derive(Debug, PartialEq, Copy, Clone)]
#[magnus::wrap(class = "PeriodPrice")]
pub struct PeriodPrice {
    pub begin: Timestamp,
    pub price: Price,
}

impl PeriodPrice {
    #[allow(dead_code)]
    pub fn begin(&self) -> Timestamp {
        self.begin
    }

    pub fn end(&self) -> Timestamp {
        self.begin + PERIOD_LENGTH // Actually the start of the next period
    }

    #[allow(dead_code)]
    pub fn price(&self) -> Price {
        self.price
    }
}

#[derive(Debug)]
pub struct PriceCalendar {
    pub data: Vec<PeriodPrice>,
}

pub const PERIOD_LENGTH: u32 = 60 * 60 * 24; // Seconds in a day
const DEFAULT_PRICE: Price = 1;

impl PriceCalendar {
    pub fn add(&mut self, period_price: PeriodPrice) {
        self.data.push(period_price)
    }

    // Data must be ordered and not overlapping
    pub fn proportional_period_prices(&self, begin: Timestamp, end: Timestamp) -> &[PeriodPrice] {
        // Get begin period index, or following index if no match
        let result =
            self.data
                .binary_search_by(|period_price| match period_price.begin.cmp(&begin) {
                    Ordering::Less => match (period_price.end() - 1).cmp(&begin) {
                        Ordering::Less => Ordering::Less,
                        Ordering::Equal => Ordering::Equal,
                        Ordering::Greater => Ordering::Equal,
                    },
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Greater => Ordering::Greater,
                });

        let start_index = match result {
            Ok(index) => index,
            Err(index) => index,
        };

        // Include all other applicable periods up to end time
        let rest = &self.data[start_index..];
        &rest[..rest
            .iter()
            .position(|pp| pp.begin >= end)
            .unwrap_or(rest.len())]
    }

    pub fn proportional_sum(&self, begin: Timestamp, end: Timestamp) -> Price {
        let pps = self.proportional_period_prices(begin, end);
        let period_price_sum: Price = pps.iter().map(|&pp| pp.price).sum();

        // Sum of parts of first and last period prices outside pricing range
        let period_price_discard = pps.first().map_or_else(
            || 0.0,
            |pp| (begin.saturating_sub(pp.begin) as f32 / PERIOD_LENGTH as f32) * pp.price as f32,
        ) + pps.last().map_or_else(
            || 0.0,
            |pp| (pp.end().saturating_sub(end) as f32 / PERIOD_LENGTH as f32) * pp.price as f32,
        );

        let period_price_part = period_price_sum as f32 - period_price_discard;

        let duration_with_periods = cmp::max(end, pps.last().map_or_else(|| end, |pp| pp.end()))
            - cmp::min(begin, pps.first().map_or_else(|| begin, |pp| pp.begin));

        let default_price_periods =
            (duration_with_periods as f32 / PERIOD_LENGTH as f32) - pps.len() as f32;

        (period_price_part + default_price_periods * (DEFAULT_PRICE as f32)).round() as Price
    }
}

#[test]
fn proportional_sum_test() {
    let index = PriceCalendar {
        data: vec![
            PeriodPrice {
                begin: 4 * PERIOD_LENGTH,
                price: 10,
            },
            PeriodPrice {
                begin: 6 * PERIOD_LENGTH,
                price: 20,
            },
        ],
    };

    let sum = index.proportional_sum(
        5 * PERIOD_LENGTH + (PERIOD_LENGTH / 2),
        6 * PERIOD_LENGTH + PERIOD_LENGTH / 2,
    );
    assert!(sum == 11, "sum = {}, should be 11", sum);

    let sum = index.proportional_sum(4 * PERIOD_LENGTH, 5 * PERIOD_LENGTH);
    assert!(sum == 10, "sum = {}, should be 10", sum);

    let sum = index.proportional_sum(2 * PERIOD_LENGTH, 5 * PERIOD_LENGTH);
    assert!(sum == 12, "sum = {}, should be 12", sum);

    let sum = index.proportional_sum(5 * PERIOD_LENGTH, 6 * PERIOD_LENGTH);
    assert!(sum == 1, "sum = {}, should be 1", sum);

    // FIXME
    let sum = index.proportional_sum(5 * PERIOD_LENGTH, 6 * PERIOD_LENGTH + 1);
    assert!(sum == 1, "sum = {}, should be 1", sum);

    let sum = index.proportional_sum(4 * PERIOD_LENGTH, 7 * PERIOD_LENGTH);
    assert!(sum == 31, "sum = {}, should be 31", sum);

    let sum = index.proportional_sum(4 * PERIOD_LENGTH, (4.5 * PERIOD_LENGTH as f32) as u32);
    assert!(sum == 5, "sum = {}, should be 5", sum);
}

#[test]
fn single_match() {
    let mut index = PriceCalendar { data: vec![] };
    index.add(PeriodPrice {
        price: 10,
        begin: 1 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 20,
        begin: 7 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 30,
        begin: 9 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 40,
        begin: 12 * PERIOD_LENGTH,
    });
    assert!(
        index.proportional_period_prices(9 * PERIOD_LENGTH, 10 * PERIOD_LENGTH)
            == &index.data[2..3]
    );
}

#[test]
fn multiple_matches() {
    let mut index = PriceCalendar { data: vec![] };
    index.add(PeriodPrice {
        price: 10,
        begin: 1 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 20,
        begin: 7 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 30,
        begin: 9 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 40,
        begin: 12 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 50,
        begin: 20 * PERIOD_LENGTH,
    });
    assert!(
        index.proportional_period_prices(8 * PERIOD_LENGTH, 12 * PERIOD_LENGTH)
            == &index.data[2..3]
    );
}

#[test]
fn no_match_all_after() {
    let mut index = PriceCalendar { data: vec![] };
    index.add(PeriodPrice {
        price: 10,
        begin: 4 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 20,
        begin: 7 * PERIOD_LENGTH,
    });
    assert!(index.proportional_period_prices(0, 3 * PERIOD_LENGTH) == &[]);
}

#[test]
fn no_match_all_before() {
    let mut index = PriceCalendar { data: vec![] };
    index.add(PeriodPrice {
        price: 10,
        begin: 4 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 22,
        begin: 7 * PERIOD_LENGTH,
    });
    assert!(index.proportional_period_prices(9 * PERIOD_LENGTH, 9 * PERIOD_LENGTH) == &[]);
}

#[test]
fn no_match_all_around() {
    let mut index = PriceCalendar { data: vec![] };
    index.add(PeriodPrice {
        price: 10,
        begin: 4 * PERIOD_LENGTH,
    });
    index.add(PeriodPrice {
        price: 22,
        begin: 7 * PERIOD_LENGTH,
    });
    assert!(index.proportional_period_prices(5 * PERIOD_LENGTH, 6 * PERIOD_LENGTH) == &[]);
}
