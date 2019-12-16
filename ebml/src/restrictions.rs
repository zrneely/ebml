
//! Restrictions on the values an `Element` may contain.

use {EbmlValue, IntValue, UintValue, DateValue, FloatValue, FloatValueRepr,
     StringValue};

#[cfg(feature = "chrono")]
use chrono::{DateTime, TimeZone, Utc};

use std::marker::PhantomData;

/// An additional restriction on the values an element type may contain.
pub trait Restriction<V: EbmlValue>: ::std::fmt::Debug {
    /// Checks to see if the value satisfies the restriction.
    fn matches(&self, value: &V) -> bool;
}

/// An intersection of multiple restrictions; a value which matches all of the restrictions matches
/// the entire restriction.
#[derive(Debug)]
pub struct Intersecton<V: EbmlValue> {
    restrictions: Vec<Box<Restriction<V>>>,
    _value: PhantomData<V>,
}
impl<V: EbmlValue> Restriction<V> for Intersecton<V> {
    fn matches(&self, value: &V) -> bool {
        self.restrictions.iter().all(|r| r.matches(value))
    }
}

/// A union of multiple restrictions; a value which matches any of the restrictions matches the
/// entire restriction.
#[derive(Debug)]
pub struct Union<V: EbmlValue> {
    restrictions: Vec<Box<Restriction<V>>>,
    _v: PhantomData<V>,
}
impl<V: EbmlValue> Restriction<V> for Union<V> {
    fn matches(&self, value: &V) -> bool {
        self.restrictions.iter().any(|r| r.matches(value))
    }
}
impl<V: EbmlValue> From<Vec<Box<Restriction<V>>>> for Union<V> {
    fn from(restrictions: Vec<Box<Restriction<V>>>) -> Self {
        Union {
            restrictions,
            _v: PhantomData,
        }
    }
}

/// An inclusive range element which can be open on one end.
#[derive(Debug, Clone)]
pub enum IntRangeRestriction {
    /// The range consists of a single value.
    Single(i64),
    /// The range is unbounded on the left, and has a maximum.
    OpenLeft {
        /// The maximum value.
        max: i64,
    },
    /// The range is unbounded on the right, and has a minimum.
    OpenRight {
        /// The minimum value.
        min: i64,
    },
    /// The range is bounded on both sides.
    Closed {
        /// The minimum value.
        min: i64,
        /// The maximum value.
        max: i64,
    },
}
impl Restriction<IntValue> for IntRangeRestriction {
    fn matches(&self, value: &IntValue) -> bool {
        use self::IntRangeRestriction::*;

        let value = value.to_repr();
        match *self {
            Single(allowed) => value == allowed,
            Closed { min, max } => min <= value && value <= max,
            OpenLeft { max } => value <= max,
            OpenRight { min } => min <= value,
        }
    }
}

/// An inclusive range element which must be closed.
#[derive(Debug, Clone)]
pub enum UintRangeRestriction {
    /// The range consists of a single value.
    Single(u64),
    /// The range is closed on both ends.
    Closed {
        /// The minimum value.
        min: u64,
        /// The maximum value.
        max: u64,
    },
    /// The range is unbounded on the right, and has a minimum value.
    OpenRight {
        /// The minimum value.
        min: u64,
    },
}
impl Restriction<UintValue> for UintRangeRestriction {
    fn matches(&self, value: &UintValue) -> bool {
        use self::UintRangeRestriction::*;

        let value = value.to_repr();
        match *self {
            Single(allowed) => value == allowed,
            Closed { min, max } => min <= value && value <= max,
            OpenRight { min } => min <= value,
        }
    }
}

/// A range which may be open or closed, inclusive or inclusive. Note that Float10's will always
/// fail this check, even when their value would be in the range, since there is no `f80` type in
/// Rust.
#[derive(Debug, Clone)]
pub enum FloatRangeRestriction {
    /// The range is unbounded on the left, and has a maximum.
    OpenLeft {
        /// The maximum value.
        max: f64,
        /// True if the maximum is included in the range.
        inclusive: bool,
    },
    /// The range in unbounded on the right, and has a minimum.
    OpenRight {
        /// The minimum value.
        min: f64,
        /// True if the minimum is included in the range.
        inclusive: bool,
    },
    /// The range is bounded on both sides.
    Closed {
        /// The minimum value.
        min: f64,
        /// True if the minimum is included in the range.
        min_inclusive: bool,
        /// The maximum value.
        max: f64,
        /// True if the maximum is included in the range.
        max_inclusive: bool,
    },
}
impl Restriction<FloatValue> for FloatRangeRestriction {
    fn matches(&self, value: &FloatValue) -> bool {
        use self::FloatRangeRestriction::*;

        match value.to_repr() {
            FloatValueRepr::F64(x) => {
                match *self {
                    OpenLeft {
                        max,
                        inclusive: true,
                    } => x <= max,
                    OpenLeft {
                        max,
                        inclusive: false,
                    } => x < max,
                    OpenRight {
                        min,
                        inclusive: true,
                    } => min <= x,
                    OpenRight {
                        min,
                        inclusive: false,
                    } => min < x,
                    Closed {
                        min,
                        min_inclusive: true,
                        max,
                        max_inclusive: true,
                    } => min <= x && x <= max,
                    Closed {
                        min,
                        min_inclusive: false,
                        max,
                        max_inclusive: true,
                    } => min < x && x <= max,
                    Closed {
                        min,
                        min_inclusive: true,
                        max,
                        max_inclusive: false,
                    } => min <= x && x < max,
                    Closed {
                        min,
                        min_inclusive: false,
                        max,
                        max_inclusive: false,
                    } => min < x && x < max,
                }
            }
            FloatValueRepr::F80(_) => false,
        }
    }
}

#[cfg(feature = "chrono")]
/// A date range using `chrono::DateTime`.
#[derive(Debug, Clone)]
pub enum DateRangeRestriction<Tz: TimeZone> {
    /// The range is unbounded on the left, and has a maximum value.
    OpenLeft {
        /// The maximum value.
        max: DateTime<Tz>,
    },
    /// The range is unbounded on the right, and has a minimum value.
    OpenRight {
        /// The minimum value.
        min: DateTime<Tz>,
    },
    /// The range is closed, and has both a minimum and a maximum. Both times must use the same
    /// time zone.
    Closed {
        /// The minimum value.
        min: DateTime<Tz>,
        /// The maximum value.
        max: DateTime<Tz>,
    },
}
#[cfg(feature = "chrono")]
impl<Tz: TimeZone> Restriction<DateValue> for DateRangeRestriction<Tz> {
    fn matches(&self, value: &DateValue) -> bool {
        use self::DateRangeRestriction::*;

        let value = value.to_repr();
        match *self {
            OpenLeft { ref max } => value <= max.with_timezone(&Utc),
            OpenRight { ref min } => min.with_timezone(&Utc) <= value,
            Closed { ref min, ref max } => {
                min.with_timezone(&Utc) <= value && value <= max.with_timezone(&Utc)
            }
        }
    }
}

#[cfg(not(feature = "chrono"))]
/// A date range using nanoseconds since the millennium (NOT the Unix Epoch!).
#[derive(Debug, Clone)]
pub enum DateRangeRestriction {
    /// The range is unbounded on the left, and has a maximum value.
    OpenLeft {
        /// The maximum value.
        max: i64,
    },
    /// The range is unbounded on the right, and has a minimum value.
    OpenRight {
        /// The minimum value.
        min: i64,
    },
    /// The range is closed, and has both a minimum and a maximum.
    Closed {
        /// The minimum value.
        min: i64,
        /// The maximum value.
        max: i64,
    },
}
#[cfg(not(feature = "chrono"))]
impl Restriction<DateValue> for DateRangeRestriction {
    fn matches(&self, value: &DateValue) -> bool {
        use self::DateRangeRestriction::*;

        let value = value.to_repr();
        match *self {
            OpenLeft { max } => value <= max,
            OpenRight { min } => min <= value,
            Closed { min, max } => min <= value && value <= max,
        }
    }
}

/// A range of legal values for a `String`. The values are given as Unicode scalar values, not
/// bytes or ASCII characters (scalar values are a subset of codepoints, excluding high- and
/// low-surrogates).
#[derive(Debug, Clone)]
pub enum StringRangeRestriction {
    /// The range consists of a single value.
    Single(char),
    /// The range is closed on both ends.
    Closed {
        /// The minimum value.
        min: char,
        /// The maximum value.
        max: char,
    },
    /// The range is unbounded on the right, and has a minimum value.
    OpenRight {
        /// The minimum value.
        min: char,
    },
}
impl Restriction<StringValue> for StringRangeRestriction {
    fn matches(&self, value: &StringValue) -> bool {
        use self::StringRangeRestriction::*;

        let value = value.to_repr();
        value.chars().all(|c| {
            match *self {
                Single(allowed) => c == allowed,
                Closed { min, max } => min <= c && c <= max,
                OpenRight { min } => min <= c,
            }
        })

    }
}
impl Restriction<StringValue> for [StringRangeRestriction] {
    fn matches(&self, value: &StringValue) -> bool {
        use self::StringRangeRestriction::*;

        let value = value.to_repr();
        value.chars().all(|c| {
            self.iter().any(|s| {
                match *s {
                    Single(allowed) => c == allowed,
                    Closed { min, max } => min <= c && c <= max,
                    OpenRight { min } => min <= c,
                }
            })
        })
    }
}
