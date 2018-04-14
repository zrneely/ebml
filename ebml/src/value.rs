
//! Values which can be stored in an EBML document.

#[cfg(feature = "chrono")]
use chrono::{Utc, DateTime, TimeZone, Duration};

const UNIX_TO_MILLENNIUM_NANOS: i64 = 978_307_200_000_000_000;
const UNIX_TO_MILLENNIUM_SECONDS: i64 = 978_307_200;

use size::Size;

/// All EBML leaf values implement this trait.
pub trait EbmlValue: ::std::fmt::Debug {
    /// The Rust representation of the value.
    // TODO once associated type constructors land, let this be generic over a lifetime so that we
    // don't have to clone the data to return it. For example, StringValue could have Repr<'a> =
    // Cow<'a, str>.
    type Repr;

    /// Gets the size of the value in bytes, or in number of elements for a container.
    ///
    /// ## Panics
    ///
    /// May panic if the size is too large (bigger than 2^56 - 2), the maximum storable size.
    fn get_size(&self) -> Size;

    /// Copies this value to its Rust representation.
    fn to_repr(&self) -> Self::Repr;
}

/// A signed integer.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum IntValue {
    /// A 0-byte signed integer whose only possible value is 0.
    Int0,
    /// A 1-byte signed integer.
    Int1(i8),
    /// A 2-byte signed integer.
    Int2(i16),
    /// A 3-byte signed integer.
    Int3(i32),
    /// A 4-byte signed integer.
    Int4(i32),
    /// A 5-byte signed integer.
    Int5(i64),
    /// A 6-byte signed integer.
    Int6(i64),
    /// A 7-byte signed integer.
    Int7(i64),
    /// A 8-byte signed integer.
    Int8(i64),
}
impl From<i8> for IntValue {
    fn from(data: i8) -> Self {
        if data == 0x00 {
            IntValue::Int0
        } else {
            IntValue::Int1(data)
        }
    }
}
impl From<i16> for IntValue {
    fn from(data: i16) -> Self {
        if data > 0x7F || data < -0x80 {
            IntValue::Int2(data)
        } else {
            (data as i8).into()
        }
    }
}
impl From<i32> for IntValue {
    fn from(data: i32) -> Self {
        if data > 0x7F_FFFF || data < -0x80_0000 {
            IntValue::Int4(data)
        } else if data > 0x7FFF || data < -0x8000 {
            IntValue::Int3(data)
        } else {
            (data as i16).into()
        }
    }
}
impl From<i64> for IntValue {
    fn from(data: i64) -> Self {
        if data > 0x7F_FFFF_FFFF_FFFF || data < -0x80_0000_0000_0000 {
            IntValue::Int8(data)
        } else if data > 0x7FFF_FFFF_FFFF || data < -0x8000_0000_0000 {
            IntValue::Int7(data)
        } else if data > 0x7F_FFFF_FFFF || data < -0x80_0000_0000 {
            IntValue::Int6(data)
        } else if data > 0x7FFF_FFFF || data < -0x8000_0000 {
            IntValue::Int5(data)
        } else {
            (data as i32).into()
        }
    }
}
impl EbmlValue for IntValue {
    type Repr = i64;

    fn get_size(&self) -> Size {
        use self::IntValue::*;

        match *self {
            Int0 => 0u8.into(),
            Int1(_) => 1u8.into(),
            Int2(_) => 2u8.into(),
            Int3(_) => 3u8.into(),
            Int4(_) => 4u8.into(),
            Int5(_) => 5u8.into(),
            Int6(_) => 6u8.into(),
            Int7(_) => 7u8.into(),
            Int8(_) => 8u8.into(),
        }
    }

    fn to_repr(&self) -> Self::Repr {
        use self::IntValue::*;

        match *self {
            Int0 => 0i64,
            Int1(x) => x as i64,
            Int2(x) => x as i64,
            Int3(x) | Int4(x) => x as i64,
            Int5(x) | Int6(x) | Int7(x) | Int8(x) => x,
        }
    }
}

/// An unsigned integer.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum UintValue {
    /// A 0-byte unsigned integer whose only possible value is 0.
    Uint0,
    /// A 1-byte unsigned integer.
    Uint1(u8),
    /// A 2-byte unsigned integer.
    Uint2(u16),
    /// A 3-byte unsigned integer.
    Uint3(u32),
    /// A 4-byte unsigned integer.
    Uint4(u32),
    /// A 5-byte unsigned integer.
    Uint5(u64),
    /// A 6-byte unsigned integer.
    Uint6(u64),
    /// A 7-byte unsigned integer.
    Uint7(u64),
    /// A 8-byte unsigned integer.
    Uint8(u64),
}
impl From<u8> for UintValue {
    fn from(data: u8) -> Self {
        if data == 0x00 {
            UintValue::Uint0
        } else {
            UintValue::Uint1(data)
        }
    }
}
impl From<u16> for UintValue {
    fn from(data: u16) -> Self {
        if data > 0xFF {
            UintValue::Uint2(data)
        } else {
            (data as u8).into()
        }
    }
}
impl From<u32> for UintValue {
    fn from(data: u32) -> Self {
        if data > 0xFF_FFFF {
            UintValue::Uint4(data)
        } else if data > 0xFFFF {
            UintValue::Uint3(data)
        } else {
            (data as u16).into()
        }
    }
}
impl From<u64> for UintValue {
    fn from(data: u64) -> Self {
        if data > 0xFF_FFFF_FFFF_FFFF {
            UintValue::Uint8(data)
        } else if data > 0xFFFF_FFFF_FFFF {
            UintValue::Uint7(data)
        } else if data > 0xFF_FFFF_FFFF {
            UintValue::Uint6(data)
        } else if data > 0xFFFF_FFFF {
            UintValue::Uint5(data)
        } else {
            (data as u32).into()
        }
    }
}
impl EbmlValue for UintValue {
    type Repr = u64;

    fn get_size(&self) -> Size {
        use self::UintValue::*;

        (match *self {
            Uint0 => 0u8,
            Uint1(_) => 1u8,
            Uint2(_) => 2u8,
            Uint3(_) => 3u8,
            Uint4(_) => 4u8,
            Uint5(_) => 5u8,
            Uint6(_) => 6u8,
            Uint7(_) => 7u8,
            Uint8(_) => 8u8,
        }).into()
    }

    fn to_repr(&self) -> Self::Repr {
        use self::UintValue::*;

        match *self {
            Uint0 => 0u64,
            Uint1(x) => x as u64,
            Uint2(x) => x as u64,
            Uint3(x) | Uint4(x) => x as u64,
            Uint5(x) | Uint6(x) | Uint7(x) | Uint8(x) => x,
        }
    }
}

/// A floating-point number.
#[derive(Debug, Clone)]
pub enum FloatValue {
    /// A 0-byte IEEE float whose only possible value is 0.0.
    Float0,
    /// A 4-byte IEEE float.
    Float4(f32),
    /// A 8-byte IEEE float.
    Float8(f64),
    /// A 10-byte IEEE float. Rust lacks support for x86 extended precision floats, so the actual
    /// value is stored as binary data.
    Float10([u8; 10]),
}
impl From<f32> for FloatValue {
    fn from(data: f32) -> Self {
        if data == 0.0 {
            FloatValue::Float0
        } else {
            FloatValue::Float4(data)
        }
    }
}
impl From<f64> for FloatValue {
    fn from(data: f64) -> Self {
        if data == 0.0 {
            FloatValue::Float0
        } else {
            FloatValue::Float8(data)
        }
    }
}
/// Since there is no `f80` type, this allows `FloatValue::get_repr` to return such values as
/// binary data.
#[derive(Debug, PartialEq, Clone)]
pub enum FloatValueRepr {
    /// The ordinary representation, an `f64`.
    F64(f64),
    /// The representation of a 10-byte floating point number.
    F80([u8; 10]),
}
impl EbmlValue for FloatValue {
    type Repr = FloatValueRepr;

    fn get_size(&self) -> Size {
        use self::FloatValue::*;

        (match *self {
            Float0 => 0u8,
            Float4(_) => 4u8,
            Float8(_) => 8u8,
            Float10(_) => 10u8,
        }).into()
    }

    fn to_repr(&self) -> Self::Repr {
        use self::FloatValue::*;

        match *self {
            Float0 => FloatValueRepr::F64(0.0f64),
            Float4(x) => FloatValueRepr::F64(x as f64),
            Float8(x) => FloatValueRepr::F64(x),
            Float10(ref x) => FloatValueRepr::F80(x.clone()),
        }
    }
}

/// A UTF-8 encoded Unicode string.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StringValue {
    data: String,
    padding_len: usize,
}
impl From<String> for StringValue {
    fn from(data: String) -> Self {
        StringValue {
            data,
            padding_len: 0,
        }
    }
}
impl<T: AsRef<str>> From<T> for StringValue {
    default fn from(data: T) -> Self {
        StringValue {
            data: data.as_ref().to_string(),
            padding_len: 0,
        }
    }
}
impl StringValue {
    /// Creates a string value with some amount of 0-padding appended to it. The padding is
    /// reflected in the size of the value but not the representation.
    pub fn with_padding(data: String, padding_len: usize) -> Self {
        StringValue { data, padding_len }
    }
}
impl EbmlValue for StringValue {
    type Repr = String;

    fn get_size(&self) -> Size {
        Size::from_u64((self.data.len() + self.padding_len) as u64)
            .expect("string + padding too long")
    }

    fn to_repr(&self) -> String {
        self.data.clone()
    }
}

/// A timestamp with nanosecond precision.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DateValue {
    nanos_since_millennium: i64,
}
#[cfg(feature = "chrono")]
impl<Tz: TimeZone> From<DateTime<Tz>> for DateValue {
    fn from(data: DateTime<Tz>) -> Self {
        let epoch = Utc.ymd(2001, 1, 1).and_hms(0, 0, 0);
        let duration = data.signed_duration_since(epoch);
        DateValue {
            nanos_since_millennium: duration.num_nanoseconds().expect(
                "Date/Time value out of range",
            ),
        }
    }
}
impl DateValue {
    /// Creates a `DateValue` given the number of milliseconds since the Unix epoch, returning
    /// `None` if the value would over/underflow.
    pub fn from_unix_millis(millis: i64) -> Option<Self> {
        millis
            .checked_sub(UNIX_TO_MILLENNIUM_NANOS)
            .and_then(|x| x.checked_mul(1_000_000i64))
            .map(|nanos_since_millennium| {
                DateValue { nanos_since_millennium }
            })
    }

    /// Creates a `DateValue` given the number of milliseconds since the Unix epoch, returning
    /// `None` if the value would over/underflow.
    pub fn from_unix_seconds(seconds: i64) -> Option<Self> {
        seconds
            .checked_sub(UNIX_TO_MILLENNIUM_SECONDS)
            .and_then(|x| x.checked_mul(1_000_000_000i64))
            .map(|nanos_since_millennium| {
                DateValue { nanos_since_millennium }
            })
    }

    #[cfg(feature = "chrono")]
    /// Creates a `DateValue` given a `DateTime`, returning `None` if the value would
    /// over/underflow.
    pub fn from_datetime<Tz: TimeZone>(datetime: DateTime<Tz>) -> Option<Self> {
        let epoch = Utc.ymd(2001, 1, 1).and_hms(0, 0, 0);
        let duration = datetime.signed_duration_since(epoch);
        duration.num_nanoseconds().map(|nanos_since_millennium| {
            DateValue { nanos_since_millennium }
        })
    }
}
impl EbmlValue for DateValue {
    #[cfg(feature = "chrono")]
    // TODO once Associated Type Constructors land, make this type generic over time zones.
    type Repr = DateTime<Utc>;
    #[cfg(not(feature = "chrono"))]
    type Repr = i64;

    fn get_size(&self) -> Size {
        8u8.into()
    }

    #[cfg(feature = "chrono")]
    fn to_repr(&self) -> Self::Repr {
        let millennium = Utc.ymd(2001, 1, 1).and_hms(0, 0, 0);
        millennium + Duration::nanoseconds(self.nanos_since_millennium)
    }

    #[cfg(not(feature = "chrono"))]
    /// Converts this to the number of nanoseconds since the Unix epoch.
    fn to_repr(&self) -> Self::Repr {
        self.nanos_since_millennium
            .checked_add(UNIX_TO_MILLENNIUM_NANOS)
            .expect("time out of range")
    }
}

/// Arbitrary binary data.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct BinaryValue {
    data: Vec<u8>,
}
impl<T: AsRef<[u8]>> From<T> for BinaryValue {
    default fn from(data: T) -> Self {
        BinaryValue { data: data.as_ref().to_vec() }
    }
}
impl From<Vec<u8>> for BinaryValue {
    fn from(data: Vec<u8>) -> Self {
        BinaryValue { data }
    }
}
impl EbmlValue for BinaryValue {
    type Repr = Vec<u8>;

    fn get_size(&self) -> Size {
        Size::from_u64(self.data.len() as u64).expect("binary data too large")
    }

    fn to_repr(&self) -> Self::Repr {
        self.data.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_8_bit_unsigned_vals() {
        let x: UintValue = 0u8.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: UintValue = 1u8.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());
    }

    #[test]
    fn from_8_bit_signed_vals() {
        let x: IntValue = 0i8.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: IntValue = 1i8.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: IntValue = (-1i8).into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(-1, x.to_repr());
    }

    #[test]
    fn from_16_bit_unsigned_vals() {
        let x: UintValue = 0u16.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: UintValue = 1u16.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: UintValue = 256u16.into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(256, x.to_repr());
    }

    #[test]
    fn from_16_bit_signed_vals() {
        let x: IntValue = 0i16.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: IntValue = 1i16.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: IntValue = (-1i16).into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(-1, x.to_repr());

        let x: IntValue = (-129i16).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(-129, x.to_repr());

        let x: IntValue = (128i16).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(128, x.to_repr());
    }

    #[test]
    fn from_32_bit_unsigned_vals() {
        let x: UintValue = 0u32.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: UintValue = 1u32.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: UintValue = 256u32.into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(256, x.to_repr());

        let x: UintValue = 65_536u32.into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(65_536, x.to_repr());

        let x: UintValue = 16_777_216u32.into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(16_777_216, x.to_repr());
    }

    #[test]
    fn from_32_bit_signed_vals() {
        let x: IntValue = 0i32.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: IntValue = 1i32.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: IntValue = (-1i32).into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(-1, x.to_repr());

        let x: IntValue = (-129i32).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(-129, x.to_repr());

        let x: IntValue = (128i32).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(128, x.to_repr());

        let x: IntValue = (-32_769i32).into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(-32_769, x.to_repr());

        let x: IntValue = (32_768i32).into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(32_768, x.to_repr());

        let x: IntValue = (-8_388_609i32).into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(-8_388_609, x.to_repr());

        let x: IntValue = (8_388_608i32).into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(8_388_608, x.to_repr());
    }

    #[test]
    fn from_64_bit_unsigned_vals() {
        let x: UintValue = 0u64.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: UintValue = 1u64.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: UintValue = 256u64.into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(256, x.to_repr());

        let x: UintValue = 65_536u64.into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(65_536, x.to_repr());

        let x: UintValue = 16_777_216u64.into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(16_777_216, x.to_repr());

        let x: UintValue = 4_294_967_296u64.into();
        assert_eq!(5, x.get_size().get_value().unwrap());
        assert_eq!(4_294_967_296, x.to_repr());

        let x: UintValue = 1_099_511_627_776u64.into();
        assert_eq!(6, x.get_size().get_value().unwrap());
        assert_eq!(1_099_511_627_776, x.to_repr());

        let x: UintValue = 281_474_976_710_656u64.into();
        assert_eq!(7, x.get_size().get_value().unwrap());
        assert_eq!(281_474_976_710_656, x.to_repr());

        let x: UintValue = 72_057_594_037_927_936u64.into();
        assert_eq!(8, x.get_size().get_value().unwrap());
        assert_eq!(72_057_594_037_927_936, x.to_repr());
    }

    #[test]
    fn from_64_bit_signed_vals() {
        let x: IntValue = 0i64.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(0, x.to_repr());

        let x: IntValue = 1i64.into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(1, x.to_repr());

        let x: IntValue = (-1i64).into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!(-1, x.to_repr());

        let x: IntValue = (-129i64).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(-129, x.to_repr());

        let x: IntValue = (128i64).into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(128, x.to_repr());

        let x: IntValue = (-32_769i64).into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(-32_769, x.to_repr());

        let x: IntValue = (32_768i64).into();
        assert_eq!(3, x.get_size().get_value().unwrap());
        assert_eq!(32_768, x.to_repr());

        let x: IntValue = (-8_388_609i64).into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(-8_388_609, x.to_repr());

        let x: IntValue = (8_388_608i32).into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(8_388_608, x.to_repr());

        let x: IntValue = (-2_147_483_649i64).into();
        assert_eq!(5, x.get_size().get_value().unwrap());
        assert_eq!(-2_147_483_649, x.to_repr());

        let x: IntValue = (2_147_483_648i64).into();
        assert_eq!(5, x.get_size().get_value().unwrap());
        assert_eq!(2_147_483_648, x.to_repr());

        let x: IntValue = (-549_755_813_889i64).into();
        assert_eq!(6, x.get_size().get_value().unwrap());
        assert_eq!(-549_755_813_889, x.to_repr());

        let x: IntValue = (549_755_813_888i64).into();
        assert_eq!(6, x.get_size().get_value().unwrap());
        assert_eq!(549_755_813_888, x.to_repr());

        let x: IntValue = (-140_737_488_355_329i64).into();
        assert_eq!(7, x.get_size().get_value().unwrap());
        assert_eq!(-140_737_488_355_329, x.to_repr());

        let x: IntValue = (140_737_488_355_328i64).into();
        assert_eq!(7, x.get_size().get_value().unwrap());
        assert_eq!(140_737_488_355_328, x.to_repr());

        let x: IntValue = (-36_028_797_018_963_969i64).into();
        assert_eq!(8, x.get_size().get_value().unwrap());
        assert_eq!(-36_028_797_018_963_969, x.to_repr());

        let x: IntValue = (36_028_797_018_963_968i64).into();
        assert_eq!(8, x.get_size().get_value().unwrap());
        assert_eq!(36_028_797_018_963_968, x.to_repr());
    }

    #[test]
    fn from_float_vals() {
        let x: FloatValue = 0.0f32.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(FloatValueRepr::F64(0.0), x.to_repr());

        let x: FloatValue = 1.0f32.into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!(FloatValueRepr::F64(1.0), x.to_repr());

        let x: FloatValue = 0.0f64.into();
        assert_eq!(0, x.get_size().get_value().unwrap());
        assert_eq!(FloatValueRepr::F64(0.0), x.to_repr());

        let x: FloatValue = 1.0f64.into();
        assert_eq!(8, x.get_size().get_value().unwrap());
        assert_eq!(FloatValueRepr::F64(1.0), x.to_repr());
    }

    #[test]
    fn from_string_vals() {
        let x: StringValue = "a".to_string().into();
        assert_eq!(1, x.get_size().get_value().unwrap());
        assert_eq!("a".to_string(), x.to_repr());

        let x: StringValue = "abcd".to_string().into();
        assert_eq!(4, x.get_size().get_value().unwrap());
        assert_eq!("abcd".to_string(), x.to_repr());

        let x: StringValue = StringValue::with_padding("asdfg".into(), 100);
        assert_eq!(105, x.get_size().get_value().unwrap());
        assert_eq!("asdfg".to_string(), x.to_repr());
    }

    #[test]
    fn from_binary_vals() {
        let x: BinaryValue = vec![0x01, 0x02][..].into();
        assert_eq!(2, x.get_size().get_value().unwrap());
        assert_eq!(vec![0x01, 0x02], x.to_repr());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn from_datetime() {
        let sample = Utc.ymd(2017, 4, 20).and_hms(4, 20, 0);
        let x: DateValue = sample.clone().into();
        assert_eq!(8, x.get_size().get_value().unwrap());
        assert_eq!(sample, x.to_repr());
    }
}
