
use std::str::{self, FromStr};
use std::num;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use ebml::Id;
use nom::{self, types::CompleteByteSlice, AsChar, ErrorKind, Needed};

use {BinaryRange, BinaryRangeItem, Cardinality, DateRange, DateRangeItem, FloatRange,
     FloatRangeItem, Header, HeaderStatement, IntRange, IntRangeItem, Level, NewType, Property,
     SizeList, StringRange, StringRangeItem, Type, UintRange, UintRangeItem};

const NANOS_PER_SEC: f64 = 1_000_000_000f64;

enum ParseError<P> {
    Utf8(str::Utf8Error),
    Parse(P),
}
fn parse_from_complete_slice<F: FromStr>(data: CompleteByteSlice) -> Result<F, ParseError<F::Err>> {
    match str::from_utf8(*data) {
        Ok(r) => F::from_str(r).map_err(|e| ParseError::Parse(e)),
        Err(e) => Err(ParseError::Utf8(e)),
    }
}

fn from_hex(s: CompleteByteSlice) -> Option<Vec<u8>> {
    let mut b = Vec::with_capacity(s.len() / 2);
    let mut modulus = 0;
    let mut buf = 0;

    for (idx, byte) in s.iter().enumerate() {
        buf <<= 4;

        match byte {
            b'A'...b'F' => buf |= byte - b'A' + 10,
            b'a'...b'f' => buf |= byte - b'a' + 10,
            b'0'...b'9' => buf |= byte - b'0',
            b' '|b'\r'|b'\n'|b'\t' => {
                buf >>= 4;
                continue
            }
            _ => return None
        }

        modulus += 1;
        if modulus == 2 {
            modulus = 0;
            b.push(buf);
        }
    }

    if modulus == 0 {
        Some(b)
    } else {
        None
    }
}

named!(lcomment<CompleteByteSlice, CompleteByteSlice>, preceded!(
    tag!("//"),
    take_until_and_consume!("\n")
));

named!(bcomment<CompleteByteSlice, CompleteByteSlice>, delimited!(
    tag!("/*"),
    take_until!("*/"),
    tag!("*/")
));

named!(comment<CompleteByteSlice, CompleteByteSlice>, alt_complete!(lcomment | bcomment));

named!(sep<CompleteByteSlice, ()>, value!((), many0!(alt!(nom::multispace | comment))));

// Sadly handwritten name parser.
fn name(input: CompleteByteSlice) -> Result<(CompleteByteSlice, &str), nom::Err<CompleteByteSlice>> {
    let len = input.len();
    if len == 0 {
        Err(nom::Err::Incomplete(Needed::Size(1)))
    } else {
        // The first character must be alpha or underscore
        let zeroth = input[0] as char;
        if !zeroth.is_alpha() && zeroth != '_' {
            Err(nom::Err::Error(error_position!(input, ErrorKind::AlphaNumeric)))
        } else {
            for (idx, item) in input[1..].iter().enumerate() {
                if !item.is_alphanum() && item.as_char() != '_' {
                    return Ok((
                        CompleteByteSlice(&input[idx + 1..]),
                        str::from_utf8(&input[0..idx + 1]).unwrap()
                    ))
                }
            }
            Ok((CompleteByteSlice(&input[len..]), str::from_utf8(&input[..]).unwrap()))
        }
    }
}

named!(id<CompleteByteSlice, Id>, map_opt!(
    map_res!(
        take_while!(nom::is_hex_digit),
        |r: CompleteByteSlice| match str::from_utf8(&r) {
            Ok(r) => u32::from_str_radix(r, 16).map_err(|e| ParseError::Parse(e)),
            Err(e) => Err(ParseError::Utf8(e)),
        }
    ),
    Id::from_encoded
));

named!(type_<CompleteByteSlice, Type>, alt_complete!(
    value!(Type::Int, tag!("int")) |
    value!(Type::Uint, tag!("uint")) |
    value!(Type::Float, tag!("float")) |
    value!(Type::String, tag!("string")) |
    value!(Type::Date, tag!("date")) |
    value!(Type::Binary, tag!("binary")) |
    value!(Type::Container, tag!("container")) |
    map!(name, |n| Type::Name(n))
));

named!(parent<CompleteByteSlice, Vec<&str>>, delimited!(
    tuple!(tag!("parent"), sep, tag!(":"), sep),
    parents,
    pair!(sep, tag!(";"))
));

named!(parents<CompleteByteSlice, Vec<&str>>, separated_nonempty_list_complete!(
    delimited!(sep, tag!(","), sep),
    name
));

named!(level<CompleteByteSlice, Level>, do_parse!(
    tag!("level") >> sep >> tag!(":") >> sep >>
    start: map_res!(
        take_while!(nom::is_digit),
        parse_from_complete_slice
    ) >>
    tag!("..") >>
    end: opt!(
        map_res!(
            take_while!(nom::is_digit),
            parse_from_complete_slice
        )
    ) >>
    sep >> tag!(";") >>

    (if let Some(end) = end {
        Level::Bounded { start, end }
    } else {
        Level::Open { start }
    })
));

named!(cardinality<CompleteByteSlice, Cardinality>, delimited!(
    tuple!(tag!("card"), sep, tag!(":"), sep),
    alt_complete!(
        value!(Cardinality::ZeroOrMany, tag!("*")) |
        value!(Cardinality::ZeroOrOne, tag!("?")) |
        value!(Cardinality::ExactlyOne, tag!("1")) |
        value!(Cardinality::OneOrMany, tag!("+"))
    ),
    pair!(sep, tag!(";"))
));

named!(int_v<CompleteByteSlice, i64>, map_res!(
    take_while!(|x| nom::is_digit(x) || x == b'-'),
    parse_from_complete_slice
));

named!(float_v<CompleteByteSlice, f64>, map_res!(
    take_while!(|x| nom::is_digit(x) || x == b'-' || x == b'+' || x == b'.' || x == b'e'),
    parse_from_complete_slice
));

named!(date_v<CompleteByteSlice, NaiveDateTime>, alt_complete!(
    do_parse!(
        year: map_res!(take!(4), parse_from_complete_slice) >>
        month: map_res!(take!(2), parse_from_complete_slice) >>
        day: map_res!(take!(2), parse_from_complete_slice) >>
        tag!("T") >>
        hour: map_res!(take!(2), parse_from_complete_slice) >>
        tag!(":") >>
        minute: map_res!(take!(2), parse_from_complete_slice) >>
        tag!(":") >>
        second: map_res!(take!(2), parse_from_complete_slice) >>
        fractional: opt!(map_res!(
            // Use recognize here to discard the pair itself, giving the input slice
            // containing the dot back.
            recognize!(
                pair!(
                    tag!("."),
                    take_while!(nom::is_digit)
                )
            ),
            parse_from_complete_slice::<f64>
        )) >>
        time: map_opt!(value!(()),
            |_| if let Some(part) = fractional {
                NaiveTime::from_hms_nano_opt(hour, minute, second, (part * NANOS_PER_SEC) as u32)
            } else {
                NaiveTime::from_hms_opt(hour, minute, second)
            }
        ) >>
        date: map_opt!(value!(()), |_| NaiveDate::from_ymd_opt(year, month, day)) >>
        (NaiveDateTime::new(date, time))
    ) |
    map!(int_v, |val| {
        // Numerical values are nanoseconds since the millennium
        let epoch = NaiveDateTime::new(
            NaiveDate::from_ymd(2001, 1, 1),
            NaiveTime::from_hms(0, 0, 0)
        );
        epoch + Duration::nanoseconds(val)
    })
));

// Not part of the spec, but helpful for implementing the string_def and binary_def things.
// This creates owned data (copies the input) since it must transform any input hex data.
named!(binary_v<CompleteByteSlice, Vec<u8>>, alt_complete!(
    preceded!(
        tag!("0x"),
        map_opt!(take_while!(nom::is_hex_digit), from_hex)
    ) |
    map!(
        delimited!(
            tag!("\""),
            recognize!(take_until!("\"")),
            tag!("\"")
        ),
        |slice| slice.to_vec()
    )
));


named!(int_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(int_v, Property::IntDefault),
    pair!(sep, tag!(";"))
));

named!(uint_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(
        map_res!(
            take_while!(nom::is_digit),
            parse_from_complete_slice
        ),
        Property::UintDefault
    ),
    pair!(sep, tag!(";"))
));

named!(float_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(float_v, Property::FloatDefault),
    pair!(sep, tag!(";"))
));

named!(date_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(date_v, Property::DateDefault),
    pair!(sep, tag!(";"))
));

named!(string_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(map_res!(binary_v, String::from_utf8), Property::StringDefault),
    pair!(sep, tag!(";"))
));

named!(binary_def<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("def"), sep, tag!(":"), sep),
    map!(binary_v, Property::BinaryDefault),
    pair!(sep, tag!(";"))
));

named!(int_range<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("range"), sep, tag!(":"), sep),
    map!(
        separated_nonempty_list_complete!(
            delimited!(sep, tag!(","), sep),
            alt_complete!(
                do_parse!(
                    start: int_v >>
                    tag!("..") >>
                    end: int_v >>
                    (IntRangeItem::Bounded { start, end })
                ) |
                map!(
                    terminated!(
                        int_v,
                        tag!("..")
                    ),
                    |start| IntRangeItem::From { start }
                ) |
                map!(
                    preceded!(
                        tag!(".."),
                        int_v
                    ),
                    |end| IntRangeItem::To { end }
                ) |
                map!(int_v, IntRangeItem::Single)
            )
        ),
        Property::IntRange
    ),
    pair!(sep, tag!(";"))
));

named!(uint_range<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("range"), sep, tag!(":"), sep),
    map!(
        separated_nonempty_list_complete!(
            delimited!(sep, tag!(","), sep),
            alt_complete!(
                do_parse!(
                    start: map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ) >>
                    tag!("..") >>
                    end: map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ) >>
                    (UintRangeItem::Bounded { start, end })
                ) |
                map!(
                    terminated!(
                        map_res!(
                            take_while!(nom::is_digit),
                            parse_from_complete_slice
                        ),
                        tag!("..")
                    ),
                    |start| UintRangeItem::From { start }
                ) |
                map!(
                    map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ),
                    UintRangeItem::Single
                )
            )
        ),
        Property::UintRange
    ),
    pair!(sep, tag!(";"))
));

named!(float_range<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("range"), sep, tag!(":"), sep),
    map!(
        separated_nonempty_list_complete!(
            delimited!(sep, tag!(","), sep),
            alt_complete!(
                do_parse!(
                    start: float_v >>
                    tag!("<") >>
                    include_start: map!(opt!(tag!("=")), |x| x.is_some()) >>
                    tag!("..") >>
                    tag!("<") >>
                    include_end: map!(opt!(tag!("=")), |x| x.is_some()) >>
                    end: float_v >>
                    (FloatRangeItem::Bounded { start, include_start, end, include_end })
                ) |
                do_parse!(
                    tag!("<") >>
                    include_end: map!(opt!(tag!("=")), |x| x.is_some()) >>
                    end: float_v >>
                    (FloatRangeItem::To { end, include_end })
                ) |
                do_parse!(
                    tag!(">") >>
                    include_start: map!(opt!(tag!("=")), |x| x.is_some()) >>
                    start: float_v >>
                    (FloatRangeItem::From { start, include_start })
                )
            )
        ),
        Property::FloatRange
    ),
    pair!(sep, tag!(";"))
));

named!(date_range<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("range"), sep, tag!(":"), sep),
    map!(
        separated_nonempty_list_complete!(
            delimited!(sep, tag!(","), sep),
            alt_complete!(
                do_parse!(
                    start: date_v >>
                    tag!("..") >>
                    end: date_v >>
                    (DateRangeItem::Bounded { start, end })
                ) |
                map!(
                    terminated!(date_v, tag!("..")),
                    |start| DateRangeItem::From { start }
                ) |
                map!(
                    preceded!(tag!(".."), date_v),
                    |end| DateRangeItem::To { end }
                )
            )
        ),
        Property::DateRange
    ),
    pair!(sep, tag!(";"))
));

named!(string_range<CompleteByteSlice, Property>, map_opt!(
    uint_range,
    |prop: Property| match prop {
        Property::UintRange(ur) => {
            ur.iter()
              .map(|uri| uri.to_string_range_item())
              .collect::<Option<Vec<_>>>()
              .map(Property::StringRange)
        }
        _ => unreachable!(),
    }
));

named!(binary_range<CompleteByteSlice, Property>, map_opt!(
    uint_range,
    |prop: Property| match prop {
        Property::UintRange(ur) => {
            ur.iter()
              .map(|uri| uri.to_binary_range_item())
              .collect::<Option<Vec<_>>>()
              .map(Property::BinaryRange)
        }
        _ => unreachable!(),
    }
));

named!(size<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("size"), sep, tag!(":"), sep),
    map!(
        separated_nonempty_list_complete!(
            delimited!(sep, tag!(","), sep),
            alt_complete!(
                do_parse!(
                    start: map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ) >>
                    tag!("..") >>
                    end: map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ) >>
                    (UintRangeItem::Bounded { start, end })
                ) |
                map!(
                    terminated!(
                        map_res!(
                            take_while!(nom::is_digit),
                            parse_from_complete_slice
                        ),
                        tag!("..")
                    ),
                    |start| UintRangeItem::From { start }
                ) |
                map!(
                    map_res!(
                        take_while!(nom::is_digit),
                        parse_from_complete_slice
                    ),
                    UintRangeItem::Single
                )
            )
        ),
        Property::Size
    ),
    pair!(sep, tag!(";"))
));

named!(ordered<CompleteByteSlice, Property>, delimited!(
    tuple!(tag!("ordered"), sep, tag!(":"), sep),
    alt_complete!(
        value!(
            Property::Ordered(true),
            alt_complete!(tag!("yes") | tag!("1"))
        ) |
        value!(
            Property::Ordered(false),
            alt_complete!(tag!("no") | tag!("0"))
        )
    ),
    pair!(sep, tag!(";"))
));

// Types impossible to distinguish:
//      Uint vs Int, if the Int happens to be positive
//      String vs Binary, if the Binary happens to be valid Unicode
named!(header_statement<CompleteByteSlice, HeaderStatement>, do_parse!(
    name: name >>
    sep >>
    tag!(":=") >>
    sep >>
    value: alt_complete!(
        // By including the terminator in these parsers, we stop floats from getting interpreted as
        // integers.
        map!(
            terminated!(
                map_res!(take_while!(nom::is_digit), parse_from_complete_slice),
                pair!(sep, tag!(";"))
            ),
            |value| HeaderStatement::Uint { name, value }
        ) |
        map!(
            terminated!(int_v, pair!(sep, tag!(";"))),
            |value| HeaderStatement::Int { name, value }
        ) |
        map!(
            terminated!(float_v, pair!(sep, tag!(";"))),
            |value| HeaderStatement::Float { name, value }
        ) |
        map!(
            terminated!(date_v, pair!(sep, tag!(";"))),
            |value| HeaderStatement::Date { name, value }
        ) |
        map!(
            terminated!(
                map_res!(binary_v, String::from_utf8),
                pair!(sep, tag!(";"))
            ),
            |value| HeaderStatement::String { name, value }
        ) |
        map!(
            terminated!(binary_v, pair!(sep, tag!(";"))),
            |value| HeaderStatement::Binary { name, value }
        ) |
        map!(
            terminated!(::parsers::name, pair!(sep, tag!(";"))),
            |value| HeaderStatement::Named { name, value }
        )
    ) >>
    (value)
));

named!(hblock<CompleteByteSlice, Header>, preceded!(
    tuple!(tag!("declare"), sep, tag!("header"), sep, tag!("{"), sep),
    separated_nonempty_list_complete!(sep, header_statement)
));

fn update_newtype_with_property<'a, 'b>(mut nt: NewType<'a>, p: Property<'b>) -> NewType<'a> {
    nt.update(p);
    nt
}

named!(dtype_param_open<CompleteByteSlice, CompleteByteSlice>, delimited!(sep, tag!("["), sep));
named!(dtype_param_close<CompleteByteSlice, ()>, do_parse!(
    sep >>
    tag!("]") >>
    sep >>
    opt!(tag!(";")) >>
    (())
));

named_args!(int_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(int_range | int_def), sep),
        NewType::Int { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named_args!(uint_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(uint_range | uint_def), sep),
        NewType::Uint { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named_args!(float_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(float_range | float_def), sep),
        NewType::Float { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named_args!(date_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(date_range | date_def), sep),
        NewType::Date { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named_args!(string_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(string_range | string_def), sep),
        NewType::String { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named_args!(binary_properties<'a>(name: &'a str) <CompleteByteSlice<'a>, NewType<'a>>, delimited!(
    dtype_param_open,
    fold_many1!(
        delimited!(sep, alt_complete!(binary_range | binary_def), sep),
        NewType::Binary { name, default: None, range: None },
        update_newtype_with_property
    ),
    dtype_param_close
));

named!(dtype<CompleteByteSlice, NewType>, do_parse!(
    name: name >>
    sep >>
    tag!(":=") >>
    value: switch!(delimited!(sep, type_, sep),

        Type::Int => alt!(
            apply!(int_properties, name) |
            value!(
                NewType::Int { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        Type::Uint => alt!(
            apply!(uint_properties, name) |
            value!(
                NewType::Uint { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        Type::Float => alt_complete!(
            apply!(float_properties, name) |
            value!(
                NewType::Float { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        Type::Date => alt_complete!(
            apply!(date_properties, name) |
            value!(
                NewType::Date { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        Type::String => alt_complete!(
            apply!(string_properties, name) |
            value!(
                NewType::String { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        Type::Binary => alt_complete!(
            apply!(binary_properties, name) |
            value!(
                NewType::Binary { name, default: None, range: None },
                not!(dtype_param_open)
            )
        ) |

        // TODO: Type::Container and Type::Name are unimplemented
        _ => value!(NewType::Int { name, default: None, range: None })
    ) >>
    (value)
));

#[cfg(test)]
mod tests;
