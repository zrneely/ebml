
#![deny(missing_docs, missing_debug_implementations,
        trivial_casts, unsafe_code)]

#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![allow(unused)]

//! Uses `nom` to parse an EDTD (an EBML Document Type Definition), and generate types for use with
//! the `ebml` crate.
//!
//! ##Errata:
//!
//! * The specification gives no upper bound on the value of integer literals. We limit them to the
//!   values representable by a signed 64-bit number.
//! * The specification states that dates are always given in "ISO short form", referring to ISO 8601.
//!   However, it then goes on to replicate part of the specification, with two mistakes: the EBML
//!   specification says that times must always be given in HH:MM:SS format, but ISO 8601 specifies
//!   that either the seconds or both the seconds and the minutes may be left off. The EBML
//!   specification states that years must always be 4 digits; ISO 8601 provides a mechanism for
//!   representing years after 9999. We follow the EBML specification (and use time-zone naive
//!   datetimes).
//! * One limitation of the EDTD specification is that string defaults may only take ASCII values
//!   between 0x20 (` `) and 0x7E (`~`), despite strings being defined as UTF-8 encoded Unicode. We
//!   remove this limitation, allowing the text between quotes in a string default to take any
//!   valid UTF-8 value.
//! * The specification is not self-consistent when describing the syntax for type aliases. In some
//!   examples, it shows them ending with semicolons; in others, it shows them without. We elect to
//!   make the semicolon optional (but recommended).
//! * There is a line in the specification which says that properties of elements and new types
//!   must be enclosed in angle brackets, the BNF it gives specifies parentheses, and every example
//!   uses square brackets. We accept square brackets only.

extern crate chrono;
extern crate ebml;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate quote;

mod parsers;

use chrono::NaiveDateTime;

type Header<'a> = Vec<HeaderStatement<'a>>;

#[derive(Debug, PartialEq)]
enum HeaderStatement<'a> {
    Int {
        name: &'a str,
        value: i64,
    },
    Uint {
        name: &'a str,
        value: u64,
    },
    Float {
        name: &'a str,
        value: f64,
    },
    Date {
        name: &'a str,
        value: NaiveDateTime,
    },
    String {
        name: &'a str,
        value: String,
    },
    Binary {
        name: &'a str,
        value: Vec<u8>,
    },
    Named {
        name: &'a str,
        value: &'a str,
    },
}

#[derive(Debug, PartialEq)]
enum NewType<'a> {
    Int {
        name: &'a str,
        default: Option<i64>,
        range: Option<IntRange>,
    },
    Uint {
        name: &'a str,
        default: Option<u64>,
        range: Option<UintRange>,
    },
    Float {
        name: &'a str,
        default: Option<f64>,
        range: Option<FloatRange>,
    },
    Date {
        name: &'a str,
        default: Option<NaiveDateTime>,
        range: Option<DateRange>,
    },
    String {
        name: &'a str,
        default: Option<String>,
        range: Option<StringRange>,
    },
    Binary {
        name: &'a str,
        default: Option<Vec<u8>>,
        range: Option<BinaryRange>,
    },
}
impl<'a> NewType<'a> {
    fn update<'b>(&mut self, val: Property<'b>) {
        match val {
            Property::IntDefault(x) => match self {
                &mut NewType::Int { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::IntRange(x) => match self {
                &mut NewType::Int { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },
            Property::UintDefault(x) => match self {
                &mut NewType::Uint { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::UintRange(x) => match self {
                &mut NewType::Uint { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },
            Property::FloatDefault(x) => match self {
                &mut NewType::Float { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::FloatRange(x) => match self {
                &mut NewType::Float { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },
            Property::DateDefault(x) => match self {
                &mut NewType::Date { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::DateRange(x) => match self {
                &mut NewType::Date { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },
            Property::StringDefault(x) => match self {
                &mut NewType::String { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::StringRange(x) => match self {
                &mut NewType::String { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },
            Property::BinaryDefault(x) => match self {
                &mut NewType::Binary { ref mut default, .. } => *default = Some(x),
                _ => unreachable!(),
            },
            Property::BinaryRange(x) => match self {
                &mut NewType::Binary { ref mut range, .. } => *range = Some(x),
                _ => unreachable!(),
            },

            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum Property<'a> {
    Parent(Vec<&'a str>),
    Level(Level),
    Cardinality(Cardinality),
    Size(SizeList),
    Ordered(bool),

    IntDefault(i64),
    IntRange(IntRange),

    UintDefault(u64),
    UintRange(UintRange),

    FloatDefault(f64),
    FloatRange(FloatRange),

    DateDefault(NaiveDateTime),
    DateRange(DateRange),

    StringDefault(String),
    StringRange(StringRange),

    BinaryDefault(Vec<u8>),
    BinaryRange(BinaryRange),
}

#[derive(Debug, Eq, PartialEq)]
enum Type<'a> {
    Int,
    Uint,
    Float,
    String,
    Date,
    Binary,
    Container,
    Name(&'a str),
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Level {
    Bounded {
        start: u64,
        end: u64,
    },
    Open {
        start: u64,
    },
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum IntRangeItem {
    Single(i64),
    From {
        start: i64,
    },
    To {
        end: i64,
    },
    Bounded {
        start: i64,
        end: i64,
    },
}
type IntRange = Vec<IntRangeItem>;

#[derive(Debug, Eq, PartialEq, Clone)]
enum UintRangeItem {
    Single(u64),
    From {
        start: u64,
    },
    // There is no To for unsigned integers
    Bounded {
        start: u64,
        end: u64,
    },
}
impl UintRangeItem {
    // binary range items must only think of a single byte
    fn to_binary_range_item(&self) -> Option<BinaryRangeItem> {
        use UintRangeItem::*;

        match *self {
            Single(x @ 0...0xFF) => {
                Some(BinaryRangeItem::Single(x as u8))
            }
            From { start: start @ 0...0xFF } => {
                Some(BinaryRangeItem::From { start: start as u8 })
            }
            Bounded { start: start @ 0...0xFF, end: end @ 0...0xFF } => {
                Some(BinaryRangeItem::Bounded {
                    start: start as u8,
                    end: end as u8
                })
            }
            _ => None
        }
    }

    // string range items operate on Unicode code points directly
    fn to_string_range_item(&self) -> Option<StringRangeItem> {
        use UintRangeItem::*;

        match *self {
            Single(x @ 0...0x10_FFFF) => {
                Some(StringRangeItem::Single(x as u32))
            }
            From { start: start @ 0...0x10_FFFF } => {
                Some(StringRangeItem::From { start: start as u32 })
            }
            Bounded { start: start @ 0...0x10_FFFF, end: end @ 0...0x10_FFFF } => {
                Some(StringRangeItem::Bounded {
                    start: start as u32,
                    end: end as u32
                })
            }
            _ => None
        }
    }
}
type UintRange = Vec<UintRangeItem>;
type SizeList = Vec<UintRangeItem>;

#[derive(Debug, PartialEq, Clone)]
enum FloatRangeItem {
    From {
        start: f64,
        include_start: bool,
    },
    To {
        end: f64,
        include_end: bool,
    },
    Bounded {
        start: f64,
        include_start: bool,
        end: f64,
        include_end: bool,
    },
}
type FloatRange = Vec<FloatRangeItem>;

#[derive(Debug, Eq, PartialEq, Clone)]
enum DateRangeItem {
    From {
        start: NaiveDateTime,
    },
    To {
        end: NaiveDateTime,
    },
    Bounded {
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
}
type DateRange = Vec<DateRangeItem>;

// This uses u32 since the values are Unicode code points, not bytes.
#[derive(Debug, Eq, PartialEq, Clone)]
enum StringRangeItem {
    Single(u32),
    From {
        start: u32,
    },
    Bounded {
        start: u32,
        end: u32,
    },
}
type StringRange = Vec<StringRangeItem>;

#[derive(Debug, Eq, PartialEq, Clone)]
enum BinaryRangeItem {
    Single(u8),
    From {
        start: u8,
    },
    Bounded {
        start: u8,
        end: u8,
    },
}
type BinaryRange = Vec<BinaryRangeItem>;

#[derive(Debug, Eq, PartialEq, Clone)]
enum Cardinality {
    ZeroOrMany,
    ZeroOrOne,
    ExactlyOne,
    OneOrMany,
}
