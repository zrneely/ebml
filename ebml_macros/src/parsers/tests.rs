use super::*;

// TODO instead of taking the whole file name, just assume it's got the same name as the function
// being tested and append a number (passed in place of the name)
macro_rules! gen_test {
    ($fn_name:ident, $test_file:expr, $expected:expr) => (
        match ::parsers::$fn_name(include_bytes!(concat!("../../tests/", $test_file))) {
            ::nom::IResult::Done(_, val) => assert_eq!($expected, val),
            ::nom::IResult::Error(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            },
            ::nom::IResult::Incomplete(amount) => {
                println!("Incomplete: {:?}", amount);
                assert!(false);
            },
        }
    );
    ($fn_name:ident, $test_file:expr, $expected:expr, $left:expr) => (
        match ::parsers::$fn_name(include_bytes!(concat!("../../tests/", $test_file))) {
            ::nom::IResult::Done(left, val) => {
                assert_eq!($expected, val);
                assert_eq!($left, left);
            },
            ::nom::IResult::Error(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            },
            ::nom::IResult::Incomplete(amount) => {
                println!("Incomplete: {:?}", amount);
                assert!(false);
            },
        }
    );
    (fail $fn_name:ident, $test_file:expr) => (
        match ::parsers::$fn_name(include_bytes!(concat!("../../tests/", $test_file))) {
            ::nom::IResult::Done(_, result) => {
                println!("Unexpected success: {:?}", result);
                assert!(false);
            }
            ::nom::IResult::Incomplete(_) => assert!(false),
            ::nom::IResult::Error(_) => {},
        }
    );
}

#[test]
fn test_lcomment() {
    gen_test!(lcomment, "lcomment", " comment");
}

#[test]
fn test_bcomment() {
    gen_test!(bcomment, "bcomment", " comment ");
}

#[test]
fn test_comment() {
    gen_test!(comment, "lcomment", " comment");
    gen_test!(comment, "bcomment", " comment ");
}

#[test]
fn test_separator() {
    gen_test!(sep, "separator0", (), b"test\n");
    gen_test!(sep, "separator1", (), b"t\n");
}

#[test]
fn test_name() {
    gen_test!(name, "name0", "SimpleName");
    gen_test!(name, "name1", "_complexName1");
    gen_test!(name, "name2", "___name___", b" foo\n");
    gen_test!(fail name, "name3");
    gen_test!(fail name, "name4");
}

#[test]
fn test_id() {
    gen_test!(id, "id0", Id::new_class_d(0x0A45_DFA3).unwrap());
    gen_test!(id, "id1", Id::new_class_a(0x1).unwrap());
    gen_test!(id, "id2", Id::new_class_a(0x7E).unwrap());
    gen_test!(fail id, "id3");
    gen_test!(id, "id4", Id::new_class_b(0x7F).unwrap());
}

#[test]
fn test_type() {
    gen_test!(type_, "vtype0", Type::Int);
    gen_test!(type_, "vtype1", Type::Uint);
    gen_test!(type_, "vtype2", Type::Float);
    gen_test!(type_, "vtype3", Type::String);
    gen_test!(type_, "vtype4", Type::Date);
    gen_test!(type_, "vtype5", Type::Binary);
    gen_test!(type_, "vtype6", Type::Name("foo_bar123"));
    gen_test!(fail type_, "vtype7");
    gen_test!(type_, "ctype0", Type::Container);
}

#[test]
fn test_parents() {
    gen_test!(parents, "parents0", vec!["name1"]);
    gen_test!(parents, "parents1", vec!["name1", "name2", "name3", "name4"]);
    gen_test!(parents, "parents2", vec!["name1"], b", 2notaname\n");
}

#[test]
fn test_parent() {
    gen_test!(parent, "parent0", vec!["name1"]);
    gen_test!(parent, "parent1", vec!["name1", "name2", "name3", "name4"]);
    // Since the parents list must end with a ";", bad names in the list can't be ignored.
    gen_test!(fail parent, "parent2");
}

#[test]
fn test_level() {
    gen_test!(level, "level0", Level::Open { start: 1 });
    gen_test!(level, "level1", Level::Bounded { start: 1, end: 3 });
    gen_test!(level, "level2", Level::Bounded { start: 4, end: 5 });
    gen_test!(level, "level3", Level::Open { start: 2341 });
    gen_test!(fail level, "level4");
}

#[test]
fn test_cardinality() {
    gen_test!(cardinality, "cardinality0", Cardinality::ZeroOrMany);
    gen_test!(cardinality, "cardinality1", Cardinality::ZeroOrOne);
    gen_test!(cardinality, "cardinality2", Cardinality::ExactlyOne);
    gen_test!(cardinality, "cardinality3", Cardinality::OneOrMany);
    gen_test!(fail cardinality, "cardinality4");
}

#[test]
fn test_int_v() {
    gen_test!(int_v, "int0", 1234);
    gen_test!(int_v, "int1", -1234);
    gen_test!(int_v, "int2", 0x7FFF_FFFF_FFFF_FFFF);
    gen_test!(int_v, "int3", -9223372036854775808);
    gen_test!(fail int_v, "int4");
    gen_test!(fail int_v, "int5");
}

#[test]
fn test_float_v() {
    gen_test!(float_v, "float0", 1f64);
    gen_test!(float_v, "float1", -1f64);
    gen_test!(float_v, "float2", 1.25132f64);
    gen_test!(float_v, "float3", -1.25132f64);
    gen_test!(float_v, "float4", 1.32e7f64);
    gen_test!(float_v, "float5", -1.31e7f64);
    gen_test!(float_v, "float6", 1e+3f64);
    gen_test!(float_v, "float7", 1e-3f64);
    gen_test!(float_v, "float8", -1e-3f64);

    // Make sure it doesn't accept random garbage
    gen_test!(fail float_v, "level1");
}

#[test]
fn test_int_def() {
    gen_test!(int_def, "int_def0", Property::IntDefault(1234));
}

#[test]
fn test_uint_def() {
    gen_test!(uint_def, "uint_def0", Property::UintDefault(1234));
}

#[test]
fn test_float_def() {
    gen_test!(float_def, "float_def0", Property::FloatDefault(1f64));
}

#[test]
fn test_date_def() {
    gen_test!(date_def, "date0", Property::DateDefault(NaiveDateTime::new(
        NaiveDate::from_ymd(2017, 1, 1),
        NaiveTime::from_hms(0, 0, 0)
    )));
    gen_test!(date_def, "date1", Property::DateDefault(NaiveDateTime::new(
        NaiveDate::from_ymd(1234, 12, 25),
        NaiveTime::from_hms_milli(14, 15, 32, 420)
    )));
    gen_test!(fail date_def, "date2");
    gen_test!(fail date_def, "date3");
    gen_test!(date_def, "date4", Property::DateDefault(NaiveDateTime::new(
        NaiveDate::from_ymd(2001, 1, 1),
        NaiveTime::from_hms_nano(0, 0, 0, 1234)
    )));
}

#[test]
fn test_string_def() {
    gen_test!(string_def, "string0", Property::StringDefault("hello".to_string()));
    gen_test!(string_def, "string1", Property::StringDefault("Test".to_string()));
    gen_test!(string_def, "string2", Property::StringDefault("Test\x04".to_string()));
    // invalid unicode
    gen_test!(fail string_def, "string3");
    // unclosed quote
    gen_test!(fail string_def, "string4");
}

#[test]
fn test_binary_def() {
    gen_test!(binary_def, "string0", Property::BinaryDefault(vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]));
    gen_test!(binary_def, "string1", Property::BinaryDefault(vec![0x54, 0x65, 0x73, 0x74]));
    gen_test!(binary_def, "string2", Property::BinaryDefault(vec![0x54, 0x65, 0x73, 0x74, 0x04]));

    // invalid unicode/ascii is fine for a binary default
    gen_test!(binary_def, "string3", Property::BinaryDefault(
        vec![0x54, 0x65, 0x73, 0x74, 0x80, 0x81, 0x82])
    );

    // unclosed quote
    gen_test!(fail binary_def, "string4");
}

#[test]
fn test_int_range() {
    gen_test!(int_range, "int_range0", Property::IntRange(vec![
        IntRangeItem::Bounded { start: -2, end: 5 },
    ]));
    gen_test!(int_range, "int_range1", Property::IntRange(vec![IntRangeItem::From { start: 4 }]));
    gen_test!(int_range, "int_range2", Property::IntRange(vec![IntRangeItem::To { end: 102 }]));
    gen_test!(int_range, "int_range3", Property::IntRange(vec![IntRangeItem::Single(45)]));
    gen_test!(int_range, "int_range4", Property::IntRange(vec![
        IntRangeItem::Bounded { start: -1, end: 4 },
        IntRangeItem::Single(5),
        IntRangeItem::From { start: 66 },
    ]));
    gen_test!(int_range, "int_range5", Property::IntRange(vec![
        IntRangeItem::Bounded { start: -100, end: -99 },
        IntRangeItem::Single(44),
        IntRangeItem::Single(55),
        IntRangeItem::Bounded { start: 66, end: 70 },
    ]));
    gen_test!(fail int_range, "int_range6");
}

#[test]
fn test_uint_range() {
    gen_test!(uint_range, "uint_range0", Property::UintRange(vec![
         UintRangeItem::Bounded { start: 2, end: 5 },
    ]));
    gen_test!(uint_range, "uint_range1", Property::UintRange(vec![
        UintRangeItem::From { start: 4 },
    ]));
    gen_test!(uint_range, "uint_range2", Property::UintRange(vec![
        UintRangeItem::Single(45),
    ]));
    gen_test!(uint_range, "uint_range3", Property::UintRange(vec![
        UintRangeItem::Bounded { start: 1, end: 4 },
        UintRangeItem::Single(5),
        UintRangeItem::From { start: 66 },
    ]));
    gen_test!(uint_range, "uint_range4", Property::UintRange(vec![
        UintRangeItem::Bounded { start: 100, end: 200 },
        UintRangeItem::Single(44),
        UintRangeItem::Single(55),
        UintRangeItem::Bounded { start: 66, end: 70 },
    ]));
    gen_test!(fail uint_range, "uint_range5");
}

#[test]
fn test_float_range() {
    gen_test!(float_range, "float_range0", Property::FloatRange(vec![
        FloatRangeItem::From { start: 0f64, include_start: false },
    ]));
    gen_test!(float_range, "float_range1", Property::FloatRange(vec![
        FloatRangeItem::From { start: 0f64, include_start: true },
    ]));
    gen_test!(float_range, "float_range2", Property::FloatRange(vec![
        FloatRangeItem::To { end: 0f64, include_end: false },
    ]));
    gen_test!(float_range, "float_range3", Property::FloatRange(vec![
        FloatRangeItem::To { end: 1.2f64, include_end: true },
    ]));
    gen_test!(float_range, "float_range4", Property::FloatRange(vec![
        FloatRangeItem::Bounded {
            start: -1.34e4,
            include_start: false,
            end: 4.0f64,
            include_end: true,
        }
    ]));
    gen_test!(float_range, "float_range5", Property::FloatRange(vec![
        FloatRangeItem::Bounded {
            start: -4.4f64,
            include_start: true,
            end: -4.2f64,
            include_end: false,
        },
        FloatRangeItem::Bounded {
            start: 1.2e6f64,
            include_start: false,
            end: 1.3e7f64,
            include_end: true,
        },
        FloatRangeItem::From {
            start: 2.4e8,
            include_start: true,
        },
    ]));
}

#[test]
fn test_date_range() {
    gen_test!(date_range, "date_range0", Property::DateRange(vec![
        DateRangeItem::From {
            start: NaiveDateTime::new(
                NaiveDate::from_ymd(1902, 01, 02),
                NaiveTime::from_hms(0, 0, 24)
            ),
        },
    ]));
    gen_test!(date_range, "date_range1", Property::DateRange(vec![
        DateRangeItem::To {
            end: NaiveDateTime::new(
                NaiveDate::from_ymd(1995, 04, 18),
                NaiveTime::from_hms_milli(4, 20, 0, 420)
            ),
        },
    ]));
    gen_test!(date_range, "date_range2", Property::DateRange(vec![
        DateRangeItem::Bounded {
            start: NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 1, 1),
                NaiveTime::from_hms_nano(0, 0, 0, 1234)
            ),
            end: NaiveDateTime::new(
                NaiveDate::from_ymd(2017, 1, 1),
                NaiveTime::from_hms_milli(19, 20, 45, 245)
            ),
        },
        DateRangeItem::From {
            start: NaiveDateTime::new(
                NaiveDate::from_ymd(2020, 01, 01),
                NaiveTime::from_hms(0, 0, 0)
            ),
        },
    ]));
    gen_test!(fail date_range, "date_range3");
}

#[test]
fn test_string_range() {
    gen_test!(string_range, "string_range0", Property::StringRange(vec![
        StringRangeItem::From { start: 32 },
    ]));
    gen_test!(string_range, "string_range1", Property::StringRange(vec![
        StringRangeItem::Bounded { start: 0x3040, end: 0x309F },
    ]));
    gen_test!(string_range, "string_range2", Property::StringRange(vec![
        StringRangeItem::Single(42),
    ]));
    gen_test!(fail string_range, "string_range3");
}

#[test]
fn test_binary_range() {
    gen_test!(binary_range, "binary_range0", Property::BinaryRange(vec![
        BinaryRangeItem::From { start: 32 },
    ]));
    gen_test!(binary_range, "binary_range1", Property::BinaryRange(vec![
        BinaryRangeItem::Bounded {
            start: 0x01,
            end:   0xFF,
        },
    ]));
    gen_test!(binary_range, "binary_range2", Property::BinaryRange(vec![
        BinaryRangeItem::Single(42),
    ]));
    gen_test!(fail binary_range, "binary_range3");
}

#[test]
fn test_size() {
    gen_test!(size, "size_range0", Property::Size(
        vec![UintRangeItem::Bounded { start: 2, end: 5 }])
    );
    gen_test!(size, "size_range1", Property::Size(vec![UintRangeItem::From { start: 4 }]));
    gen_test!(size, "size_range2", Property::Size(vec![UintRangeItem::Single(45)]));
    gen_test!(size, "size_range3", Property::Size(vec![
        UintRangeItem::Bounded { start: 1, end: 4 },
        UintRangeItem::Single(5),
        UintRangeItem::From { start: 66 },
    ]));
    gen_test!(size, "size_range4", Property::Size(vec![
        UintRangeItem::Bounded { start: 100, end: 200 },
        UintRangeItem::Single(44),
        UintRangeItem::Single(55),
        UintRangeItem::Bounded { start: 66, end: 70 },
    ]));
    gen_test!(fail size, "size_range5");
}

#[test]
fn test_ordered() {
    gen_test!(ordered, "ordered0", Property::Ordered(true));
    gen_test!(ordered, "ordered1", Property::Ordered(true));
    gen_test!(ordered, "ordered2", Property::Ordered(false));
    gen_test!(ordered, "ordered3", Property::Ordered(false));
}

#[test]
fn test_header_statement() {
    gen_test!(header_statement, "header_statement0", HeaderStatement::Uint {
        name: "FooBar",
        value: 1
    });
    gen_test!(header_statement, "header_statement1", HeaderStatement::Int {
        name: "FooBar",
        value: -1,
    });
    gen_test!(header_statement, "header_statement2", HeaderStatement::Float {
        name: "FooBarBaz",
        value: 1.25e-2f64,
    });
    gen_test!(header_statement, "header_statement3", HeaderStatement::Date {
        name: "FooBar",
        value: NaiveDateTime::new(
            NaiveDate::from_ymd(2014, 2, 3),
            NaiveTime::from_hms_milli(0, 12, 14, 500)
        ),
    });
    gen_test!(header_statement, "header_statement4", HeaderStatement::String {
        name: "FooBar",
        value: "any unicode string 隣町".to_string(),
    });
    gen_test!(header_statement, "header_statement5", HeaderStatement::Binary {
        name: "FooBar",
        value: vec![0xFA, 0xDE, 0xF0, 0x0D],
    });
}

#[test]
fn test_hblock() {
    gen_test!(hblock, "hblock0", vec![
        HeaderStatement::Uint {
            name: "FooBar",
            value: 1,
        },
        HeaderStatement::String {
            name: "Foo1",
            value: "test".to_string(),
        },
        HeaderStatement::Binary {
            name: "FooBaz",
            value: vec![0xFA, 0xDE, 0xF0, 0x0D],
        },
        HeaderStatement::Date {
            name: "FooQux",
            value: NaiveDateTime::new(
                NaiveDate::from_ymd(2000, 1, 1),
                NaiveTime::from_hms(0, 0, 0)
            ),
        },
        HeaderStatement::String {
            name: "Foo",
            value: "隣町".to_string(),
        },
    ]);
}

#[test]
fn test_dtype() {
    gen_test!(dtype, "dtype0", NewType::Int {
        name: "Foo",
        default: None,
        range: None,
    });
    gen_test!(dtype, "dtype1", NewType::Int {
        name: "bar123",
        default: Some(25),
        range: Some(vec![IntRangeItem::Bounded {
            start: -25,
            end: 100,
        }]),
    });
    gen_test!(dtype, "dtype2", NewType::Uint {
        name: "Foo",
        default: None,
        range: None,
    });
    gen_test!(dtype, "dtype3", NewType::Uint {
        name: "Foo",
        default: Some(666),
        range: None,
    });
    gen_test!(dtype, "dtype4", NewType::Float {
        name: "Foo",
        default: None,
        range: None,
    });
    gen_test!(dtype, "dtype5", NewType::Float {
        name: "Foo",
        default: None,
        range: Some(vec![
            FloatRangeItem::To {
                end: -1.0e8,
                include_end: true,
            },
            FloatRangeItem::From {
                start: 6.4,
                include_start: false,
            },
            FloatRangeItem::Bounded {
                start: 4.0,
                include_start: true,
                end: 6.3,
                include_end: false,
            },
        ]),
    });
    gen_test!(dtype, "dtype6", NewType::Date {
        name: "abcdefghijklmnopqrstuvwxyz1234567890",
        default: None,
        range: None,
    });
    gen_test!(dtype, "dtype7", NewType::Date {
        name: "Foo",
        default: None,
        range: Some(vec![
            DateRangeItem::From {
                start: NaiveDateTime::new(
                    NaiveDate::from_ymd(1776, 6, 4),
                    NaiveTime::from_hms_milli(9, 21, 55, 356)
                ),
            },
        ]),
    });
    gen_test!(dtype, "dtype8", NewType::String {
        name: "foo",
        default: None,
        range: None,
    });
    gen_test!(dtype, "dtype9", NewType::String {
        name: "FooA",
        default: Some("elephant".into()),
        range: Some(vec![
            StringRangeItem::Bounded {
                start: 12352,
                end: 12447,
            },
            StringRangeItem::Bounded {
                start: 32,
                end: 127,
            },
        ]),
    });
    gen_test!(dtype, "dtype10", NewType::Binary {
        name: "foo",
        default: None,
        range: None,
    });

    // TODO fail test for every type with empty params list (ie [])
    // TODO fail test for every type with param list that doesn't parse
}
