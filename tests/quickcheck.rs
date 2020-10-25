#![cfg(feature = "quickcheck")]

use quickcheck_dep::{quickcheck, Arbitrary, QuickCheck, StdGen, TestResult};
use rand::{rngs::StdRng, SeedableRng};
use std::convert::TryFrom;
use time::{Date, Duration, Time};

/// Returns a statically seeded generator to ensure tests are deterministic
fn make_generator(size: usize) -> StdGen<StdRng> {
    StdGen::new(StdRng::from_seed([0; 32]), size)
}

macro_rules! test_generator_size {
    ($type:ty,
     $($($method:ident()).+ $(min=$min_value:literal)?),+,
     $size:literal
    ) => {{
        let mut q = QuickCheck::with_gen(make_generator($size));
        let mut g = make_generator($size);

        $(
            // Check that size sets upper bound.
            // We check that the generated value is bounded by the requested
            // size. If $min_value is present, then that is also an accepted
            // value even if numerically greater than the size.
            q.quickcheck((|v: $type| {
                let value = v.$($method()).+;
                value <= $size $(|| value == $min_value)?
            }) as fn($type) -> bool);

            // Check that full range is used
            let mut found_small_value = false;
            let mut found_large_value = false;
            let mut iterations = 0;

            while !(found_small_value && found_large_value) && iterations <= $size * 2 {
                iterations += 1;

                let v = <$type>::arbitrary(&mut g);
                let value = v$(.$method())+;
                if value <= $size / 4 $(|| value == $min_value)? {
                    found_small_value = true;
                }
                if value >= $size / 2 {
                    found_large_value = true;
                }
            }

            assert!(
                found_small_value,
                "Found no small value for {} {} at size {}",
                stringify!($type),
                stringify!($(.$method())+),
                stringify!($size),
            );
            assert!(
                found_large_value,
                "Found no large value for {} {} at size {}",
                stringify!($type),
                stringify!($(.$method())+),
                stringify!($size),
            );
        )+
    }};
}

macro_rules! test_shrink {
    ($type:ty,
     $fn_name:ident,
     $($method:ident()).+
     $(, min=$min_value:literal)?
    ) => {
        quickcheck! {
            fn $fn_name(v: $type) -> TestResult {
                let method_value = v.$($method()).+;
                if method_value == 0 $(|| method_value == $min_value)? {
                    TestResult::discard()
                } else {
                    TestResult::from_bool(
                        v.shrink()
                            .any(|shrunk|
                                 shrunk.$($method()).+ < v.$($method()).+))
                }
            }
        }
    };
}

quickcheck! {
    fn date_supports_arbitrary(d: Date) -> bool {
        Date::from_ymd(d.year(), d.month(), d.day()) == Ok(d)
    }
}
test_shrink!(Date, date_can_shrink_year, year().abs());
test_shrink!(Date, date_can_shrink_ordinal, ordinal(), min = 1);

#[test]
fn arbitrary_date_respects_generator_size() {
    test_generator_size!(Date, year().abs(), 0);
    test_generator_size!(Date, year().abs(), 1);
    test_generator_size!(Date, year().abs(), 100);
    test_generator_size!(Date, year().abs(), 10_000);
    test_generator_size!(Date, year().abs(), 100_000);

    test_generator_size!(Date, ordinal() min=1, 1);
    test_generator_size!(Date, ordinal(), 10);
    test_generator_size!(Date, ordinal(), 100);
    test_generator_size!(Date, ordinal(), 366);
}

quickcheck! {
    fn duration_supports_arbitrary(d: Duration) -> bool {
        Duration::new(d.whole_seconds(), d.subsec_nanoseconds()) == d
    }
}
test_shrink!(Duration, duration_can_shrink_seconds, whole_seconds().abs());
test_shrink!(Duration, duration_can_shrink_ns, subsec_nanoseconds().abs());

#[test]
fn arbitrary_duration_respects_generator_size() {
    test_generator_size!(Duration, whole_seconds().abs(), 0);
    test_generator_size!(Duration, whole_seconds().abs(), 1);
    test_generator_size!(Duration, whole_seconds().abs(), 1000);
    test_generator_size!(Duration, whole_seconds().abs(), 1_000_000);
    test_generator_size!(Duration, whole_seconds().abs(), 1_000_000_000);

    test_generator_size!(Duration, subsec_nanoseconds().abs(), 0);
    test_generator_size!(Duration, subsec_nanoseconds().abs(), 1);
    test_generator_size!(Duration, subsec_nanoseconds().abs(), 1000);
    test_generator_size!(Duration, subsec_nanoseconds().abs(), 1_000_000);
    test_generator_size!(Duration, subsec_nanoseconds().abs(), 1_000_000_000);
}

quickcheck! {
    fn time_supports_arbitrary(t: Time) -> bool {
        Time::from_hms_nano(t.hour(), t.minute(), t.second(), t.nanosecond()) == Ok(t)
    }
}
test_shrink!(Time, time_can_shrink_hour, hour());
test_shrink!(Time, time_can_shrink_minute, minute());
test_shrink!(Time, time_can_shrink_second, second());
test_shrink!(Time, time_can_shrink_nanosecond, nanosecond());

#[test]
fn arbitrary_time_respects_generator_size() {
    test_generator_size!(Time, nanosecond(), second(), minute(), hour(), 0);
    test_generator_size!(Time, nanosecond(), second(), minute(), hour(), 1);
    test_generator_size!(Time, nanosecond(), second(), minute(), hour(), 10);
    test_generator_size!(Time, nanosecond(), second(), minute(), 100);
    test_generator_size!(Time, nanosecond(), 1000);
    test_generator_size!(Time, nanosecond(), 1_000_000);
    test_generator_size!(Time, nanosecond(), 1_000_000_000);
}
