
use std::cmp::Ordering;
use std::io::{Read, Write};

use error::EbmlResult;
use peek::PeekableReader;

// The reserved "unknown" values have these heads and tails of 0xFF.
const UNKNOWN_HEAD_VALUES: [u8; 8] = [0xFF, 0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01];
// The bitmask applied to the head to decode it as part of the real value.
const HEAD_MASK_VALUES: [u8; 8] = [0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x03, 0x01, 0x00];

/// An integer with a special value, representing an unknown size.
pub const UNKNOWN_SIZE: Size = Size {
    head: 0xFF,
    tail: [0; 7],
};

/// An unsigned variable-width integer, used by EBML to represent a size. It can also represent an
/// unknown size. The range of this integer is 0 to 2^56 - 2.
///
/// The unknown size is always equal to the unknown size, and each other value is equal to itself.
/// However, the unknown size can not be ordered with respect to known sizes.
#[derive(Debug, Clone)]
pub struct Size {
    head: u8,
    tail: [u8; 7], // the "length" of the array is head.leading_zeros(). MSB always at index 0.
}
impl Size {
    /// Attempts to read a `Size` from a data source.
    pub(crate) fn load<R: Read>(source: &mut PeekableReader<R>) -> EbmlResult<Self> {
        let (head, tail) = {
            // read the next 8 bytes, which is the maximum length of a Size
            let buf = source.peek8();
            let tail_len = buf[0].leading_zeros() as usize;
            let mut tail = [0u8; 7];
            for i in 0..tail_len {
                tail[i] = buf[1 + i];
            }
            (buf[0], tail)
        };

        source.advance(1 + head.leading_zeros() as usize)?;
        Ok(Size { head, tail })
    }

    /// Attempts to write a `Size` to a data sink.
    pub(crate) fn write<W: Write>(_target: &mut W) -> EbmlResult<()> {
        unimplemented!("writing not yet supported")
    }

    /// Retrieves the width of this integer (the number of bytes the representation requires).
    pub fn get_width(&self) -> usize {
        self.head.leading_zeros() as usize + 1
    }

    /// Retrieves the value as a `u64`, returning `None` if this represents an unknown size.
    pub fn get_value(&self) -> Option<u64> {
        let tail_len = self.head.leading_zeros() as usize;

        if self.tail[..tail_len].iter().all(|x| *x == 0xFFu8) &&
                self.head == UNKNOWN_HEAD_VALUES[tail_len] {
            return None;
        }

        let mut mantissa = 1u64;
        let mut value = 0u64;

        for i in 0..tail_len {
            // The tail always stores the MSB at position 0.
            value += mantissa * (self.tail[tail_len - 1 - i] as u64);
            mantissa <<= 8;
        }
        Some(
            value + (mantissa * (self.head & HEAD_MASK_VALUES[tail_len]) as u64),
        )
    }

    /// Converts the given value to an `Size`, failing if the value is out of range (that is,
    /// greater than 2^56 - 2).
    pub fn from_u64(data: u64) -> Option<Self> {
        let log = 64 - data.leading_zeros() as u64;
        // 0xFF_FFFF_FFFF_FFFF would normally be stored in width 8, but data of all 1's is reserved
        Some(if log > 56 || data == 0xFF_FFFF_FFFF_FFFFu64 {
            // width = 9, which we can't represent
            return None;
        // 0x01_FFFF_FFFF_FFFF would normally be stored in width 7, but data of all 1's is reserved
        } else if log > 49 || data == 0x01_FFFF_FFFF_FFFFu64 {
            // width = 8 (the tail gets the 56 LSB, the head gets the next 0 LSB)
            Size {
                head: 0x01,
                tail: [
                    (data >> 48) as u8,
                    (data >> 40) as u8,
                    (data >> 32) as u8,
                    (data >> 24) as u8,
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                ],
            }
        // 0x3F_FFFF_FFFF would normally be stored in width 6, but data of all 1's is reserved
        } else if log > 42 || data == 0x03FF_FFFF_FFFFu64 {
            // width = 7 (the tail gets the 48 LSB, the head gets the next 1 LSB)
            Size {
                head: 0x02 | (data >> 48) as u8,
                tail: [
                    (data >> 40) as u8,
                    (data >> 32) as u8,
                    (data >> 24) as u8,
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                ],
            }
        // 0x07_FFFF_FFFF would normally be stored in width 5, but data of all 1's is reserved
        } else if log > 35 || data == 0x07_FFFF_FFFFu64 {
            // width = 6 (the tail gets the 40 LSB, the head gets the next 2 LSB)
            Size {
                head: 0x04 | (data >> 40) as u8,
                tail: [
                    (data >> 32) as u8,
                    (data >> 24) as u8,
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                ],
            }
        // 0x0FFF_FFFF would normally be stored in width 4, but data of all 1's is reserved
        } else if log > 28 || data == 0x0FFF_FFFF {
            // width = 5 (the tail gets the 32 LSB, the head gets the next 3 LSB)
            Size {
                head: 0x08 | (data >> 32) as u8,
                tail: [
                    (data >> 24) as u8,
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        } else {
            (data as u32).into()
        })
    }
}
impl From<u8> for Size {
    fn from(data: u8) -> Self {
        let log = 8 - data.leading_zeros() as u64;
        // 0x7F would normally be stored in width 1, but data of all 1's is reserved
        if log > 7 || data == 0x7Fu8 {
            // width = 2, but we only need the last byte
            Size {
                head: 0x40,
                tail: [
                    data,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        } else {
            // width = 1 (we fit in 7 bits)
            Size {
                head: 0x80 | data,
                tail: [0; 7],
            }
        }
    }
}
impl From<u16> for Size {
    fn from(data: u16) -> Self {
        let log = 16 - data.leading_zeros() as u64;
        // 0x3FFF would normally be stored in width 2, but data of all 1's is reserved
        if log > 14 || data == 0x3FFFu16 {
            // width = 3, but we only need the last 2 bytes
            Size {
                head: 0x20,
                tail: [
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        // 0x7F would normally be stored in width 1, but data of all 1's is reserved
        } else if log > 7 || data == 0x7F {
            // width = 2 (the tail gets the 8 LSB, the head the next 6 LSB)
            Size {
                head: 0x40 | (data >> 8) as u8,
                tail: [
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        } else {
            (data as u8).into()
        }
    }
}
impl From<u32> for Size {
    fn from(data: u32) -> Self {
        let log = 32 - data.leading_zeros() as u64;
        // 0x0FFF_FFFF would normally be stored in width 4, but data of all 1's is reserved
        if log > 28 || data == 0x0FFF_FFFFu32 {
            // width = 5, but we only need the last 4 bytes
            Size {
                head: 0x08,
                tail: [
                    (data >> 24) as u8,
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        // 0x1F_FFFF would normally be stored in width 3, but data of all 1's is reserved
        } else if log > 21 || data == 0x1F_FFFFu32 {
            // width = 4 (the tail gets the 24 LSB, the head gets the next 4 LSB)
            Size {
                head: 0x10 | (data >> 24) as u8,
                tail: [
                    (data >> 16) as u8,
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        // 0x3FFF would normally be stored in width 2, but data of all 1's is reserved
        } else if log > 14 || data == 0x3FFFu32 {
            // width = 3 (the tail gets the 16 LSB, the head gets the next 5 LSB)
            Size {
                head: 0x20 | (data >> 16) as u8,
                tail: [
                    (data >> 8) as u8,
                    data as u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                    0u8,
                ],
            }
        } else {
            (data as u16).into()
        }
    }
}
impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if let Some(self_val) = self.get_value() {
            if let Some(other_val) = other.get_value() {
                self_val.partial_cmp(&other_val)
            } else {
                None
            }
        } else {
            None
        }
    }
}
impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        let other = other.get_value();
        if let Some(self_val) = self.get_value() {
            if let Some(other_val) = other {
                self_val == other_val
            } else {
                false
            }
        } else {
            other.is_none()
        }
    }
}
impl Eq for Size {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ord_eq() {
        let x: Size = 4u8.into();
        let y: Size = 5u32.into();
        let z = UNKNOWN_SIZE;

        assert_eq!(x, x);
        assert_ne!(x, y);
        assert_ne!(x, z);
        assert_ne!(y, z);
        assert_eq!(z, z);

        assert!(x < y);
        assert!(x <= y);
        assert!(y > x);
        assert!(y >= x);
        assert!(x >= x);
        assert!(x <= x);
        assert!(y <= y);
        assert!(y >= y);

        assert!(!(z > x));
        assert!(!(z >= x));
        assert!(!(x > z));
        assert!(!(x >= z));
        assert!(!(z > y));
        assert!(!(z >= y));
        assert!(!(y > z));
        assert!(!(y >= z));
    }

    #[test]
    fn unknown() {
        let x = UNKNOWN_SIZE;
        assert_eq!(0b1111_1111, x.head);
        assert_eq!(1, x.get_width());
        assert!(x.get_value().is_none());
    }

    #[test]
    fn from_u8() {
        let x: Size = 0u8.into();
        assert_eq!(0b1000_0000, x.head);
        assert_eq!(1, x.get_width());
        assert_eq!(0, x.get_value().unwrap());

        let x: Size = 1u8.into();
        assert_eq!(0b1000_0001, x.head);
        assert_eq!(1, x.get_width());
        assert_eq!(1, x.get_value().unwrap());

        let x: Size = 127u8.into();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b0111_1111, x.tail[0]);
        assert_eq!(127, x.get_value().unwrap());

        let x: Size = 128u8.into();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b1000_0000, x.tail[0]);
        assert_eq!(128, x.get_value().unwrap());

        let x: Size = 233u8.into();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b1110_1001, x.tail[0]);
        assert_eq!(233, x.get_value().unwrap());
    }

    #[test]
    fn from_u16() {
        let x: Size = 4u16.into();
        assert_eq!(0b1000_0100, x.head);
        assert_eq!(1, x.get_width());
        assert_eq!(4, x.get_value().unwrap());

        let x: Size = 127u16.into();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b0111_1111, x.tail[0]);
        assert_eq!(127, x.get_value().unwrap());

        let x: Size = 4000u16.into();
        assert_eq!(0b0100_1111, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b1010_0000, x.tail[0]);
        assert_eq!(4000, x.get_value().unwrap());

        let x: Size = 16383u16.into();
        assert_eq!(0b0010_0000, x.head);
        assert_eq!(3, x.get_width());
        assert_eq!(0b0011_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(16383, x.get_value().unwrap());

        let x: Size = 65534u16.into();
        assert_eq!(0b0010_0000, x.head);
        assert_eq!(3, x.get_width());
        assert_eq!(0b1111_1111, x.tail[0]);
        assert_eq!(0b1111_1110, x.tail[1]);
        assert_eq!(65534, x.get_value().unwrap());
    }

    #[test]
    fn from_u32() {
        let x: Size = 4u32.into();
        assert_eq!(0b1000_0100, x.head);
        assert_eq!(1, x.get_width());
        assert_eq!(4, x.get_value().unwrap());

        let x: Size = 8_323_591u32.into();
        assert_eq!(0b0001_0000, x.head);
        assert_eq!(4, x.get_width());
        assert_eq!(0b0111_1111, x.tail[0]);
        assert_eq!(0b0000_0010, x.tail[1]);
        assert_eq!(0b0000_0111, x.tail[2]);
        assert_eq!(8_323_591, x.get_value().unwrap());

        let x: Size = 127u32.into();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b0111_1111, x.tail[0]);
        assert_eq!(127, x.get_value().unwrap());

        let x: Size = 16_383u32.into();
        assert_eq!(0b0010_0000, x.head);
        assert_eq!(3, x.get_width());
        assert_eq!(0b0011_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(16_383, x.get_value().unwrap());

        let x: Size = 2_097_151u32.into();
        assert_eq!(0b0001_0000, x.head);
        assert_eq!(4, x.get_width());
        assert_eq!(0b0001_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(2_097_151, x.get_value().unwrap());

        let x: Size = 268_435_455u32.into();
        assert_eq!(0b0000_1000, x.head);
        assert_eq!(5, x.get_width());
        assert_eq!(0b0000_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(268_435_455, x.get_value().unwrap());
    }

    #[test]
    fn from_u64() {
        let x = Size::from_u64(4).unwrap();
        assert_eq!(0b1000_0100, x.head);
        assert_eq!(1, x.get_width());
        assert_eq!(4, x.get_value().unwrap());

        let x = Size::from_u64(3_423_912_007_635).unwrap();
        assert_eq!(0b0000_0111, x.head);
        assert_eq!(6, x.get_width());
        assert_eq!(0b0001_1101, x.tail[0]);
        assert_eq!(0b0011_0001, x.tail[1]);
        assert_eq!(0b0000_1111, x.tail[2]);
        assert_eq!(0b0001_0111, x.tail[3]);
        assert_eq!(0b1101_0011, x.tail[4]);
        assert_eq!(3_423_912_007_635, x.get_value().unwrap());

        let x = Size::from_u64(127).unwrap();
        assert_eq!(0b0100_0000, x.head);
        assert_eq!(2, x.get_width());
        assert_eq!(0b0111_1111, x.tail[0]);
        assert_eq!(127, x.get_value().unwrap());

        let x = Size::from_u64(16_383).unwrap();
        assert_eq!(0b0010_0000, x.head);
        assert_eq!(3, x.get_width());
        assert_eq!(0b0011_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(16_383, x.get_value().unwrap());

        let x = Size::from_u64(2_097_151).unwrap();
        assert_eq!(0b0001_0000, x.head);
        assert_eq!(4, x.get_width());
        assert_eq!(0b0001_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(2_097_151, x.get_value().unwrap());

        let x = Size::from_u64(268_435_455).unwrap();
        assert_eq!(0b0000_1000, x.head);
        assert_eq!(5, x.get_width());
        assert_eq!(0b0000_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(268_435_455, x.get_value().unwrap());

        let x = Size::from_u64(34_359_738_367).unwrap();
        assert_eq!(0b0000_0100, x.head);
        assert_eq!(6, x.get_width());
        assert_eq!(0b0000_0111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(0b1111_1111, x.tail[4]);
        assert_eq!(34_359_738_367, x.get_value().unwrap());

        let x = Size::from_u64(4_398_046_511_103).unwrap();
        assert_eq!(0b0000_0010, x.head);
        assert_eq!(7, x.get_width());
        assert_eq!(0b0000_0011, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(0b1111_1111, x.tail[4]);
        assert_eq!(0b1111_1111, x.tail[5]);
        assert_eq!(4_398_046_511_103, x.get_value().unwrap());

        let x = Size::from_u64(562_949_953_421_311).unwrap();
        assert_eq!(0b0000_0001, x.head);
        assert_eq!(8, x.get_width());
        assert_eq!(0b0000_0001, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(0b1111_1111, x.tail[4]);
        assert_eq!(0b1111_1111, x.tail[5]);
        assert_eq!(0b1111_1111, x.tail[6]);
        assert_eq!(562_949_953_421_311, x.get_value().unwrap());

        // the maximum value storable
        let x = Size::from_u64(72_057_594_037_927_934).unwrap();
        assert_eq!(0b0000_0001, x.head);
        assert_eq!(8, x.get_width());
        assert_eq!(0b1111_1111, x.tail[0]);
        assert_eq!(0b1111_1111, x.tail[1]);
        assert_eq!(0b1111_1111, x.tail[2]);
        assert_eq!(0b1111_1111, x.tail[3]);
        assert_eq!(0b1111_1111, x.tail[4]);
        assert_eq!(0b1111_1111, x.tail[5]);
        assert_eq!(0b1111_1110, x.tail[6]);
        assert_eq!(72_057_594_037_927_934, x.get_value().unwrap());

        let x = Size::from_u64(72_057_594_037_927_935);
        assert!(x.is_none());

        let x = Size::from_u64(72_057_594_037_927_936);
        assert!(x.is_none());
    }
}
