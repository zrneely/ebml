
use std::io::{Read, Write};

use error::{EbmlError, EbmlResult};
use peek::PeekableReader;
use size::Size;

/// An EBML ID. These are nearly identical to Sizes, except there are additional reserved values
/// and different maximum widths.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Id {
    data: Size,
}
impl Id {
    /// Attempts to read an `Id` from a data source.
    pub(crate) fn load<R: Read>(source: &mut PeekableReader<R>) -> EbmlResult<Self> {
        let size = Size::load(source)?;
        if size.get_width() <= 4 {
            Ok(Id { data: size })
        } else {
            Err(EbmlError::IdOutOfRange)
        }
    }

    /// Attempts to write an `Id` to a data sink.
    pub(crate) fn write<W: Write>(_target: &mut W) -> EbmlResult<()> {
        unimplemented!("writing not yet supported")
    }

    /// Constructs an EBML ID from its encoded representation.
    pub fn from_encoded(data: u32) -> Option<Self> {
        if data        >= 0x0000_0080 && data <= 0x0000_00FE {
            Self::new_class_a((data & 0x7F) as u8)
        } else if data >= 0x0000_4000 && data <= 0x0000_7FFF {
            Self::new_class_b((data & 0x3FFF) as u16)
        } else if data >= 0x0020_0000 && data <= 0x003F_FFFF {
            Self::new_class_c(data & 0x1F_FFFF)
        } else if data >= 0x1000_0000 && data <= 0x1FFF_FFFF {
            Self::new_class_d(data & 0x0FFF_FFFF)
        } else {
            None
        }
    }

    /// Constructs an EBML Class A ID (width 1) from its literal value, returning `None` if the
    /// value is not in range for the ID. The range of valid values is 0x01 to 0x7E inclusive, so
    /// there are 126 possible Class A IDs.
    ///
    /// This does _not_ take the 'encoded' form of the ID.
    // TODO make this and similar const
    pub fn new_class_a(data: u8) -> Option<Self> {
        if data == 0u8 || data >= 0x7Fu8 {
            None
        } else {
            Some(Id { data: data.into() })
        }
    }

    /// Constructs an EBML Class B ID (width 2) from its literal value, returning `None` if the
    /// value is not in range for the ID. The range of valid values is 0x7F to 0x3FFE inclusive, so
    /// there are 16256 Class B IDs.
    ///
    /// This does _not_ take the 'encoded' form of the ID.
    pub fn new_class_b(data: u16) -> Option<Self> {
        if data < 0x7Fu16 || data >= 0x3FFFu16 {
            None
        } else {
            Some(Id { data: data.into() })
        }
    }

    /// Constructs an EBML Class C ID (width 3) from its literal value, returning `None` if the
    /// value is not in range for the ID. The range of valid values is 0x3FFF to 0x1F_FFFE
    /// inclusive, so there are 2080768 Class C IDs.
    ///
    /// This does _not_ take the 'encoded' form of the ID.
    pub fn new_class_c(data: u32) -> Option<Self> {
        if data < 0x3FFF || data >= 0x1F_FFFF {
            None
        } else {
            Some(Id { data: data.into() })
        }
    }

    /// Constructs an EBML Class D ID (width 4) from its literal value, returning `None` if the
    /// value is not in range for the ID. The range of valid values is 0x001F_FFFF to 0x0FFF_FFFE
    /// inclusive, so there are 266338304 Class D IDs.
    ///
    /// This does _not_ take the 'encoded' form of the ID.
    pub fn new_class_d(data: u32) -> Option<Self> {
        if data < 0x1F_FFFF || data >= 0x0FFF_FFFF {
            None
        } else {
            Some(Id { data: data.into() })
        }
    }

    /// Gets the width of the ID. A width of 1 means the ID is Class A, width of 2 means Class B,
    /// etc.
    // TODO make this const
    pub fn get_width(&self) -> usize {
        self.data.get_width()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_a() {
        assert!(Id::new_class_a(0x00).is_none());
        assert_eq!(1, Id::new_class_a(0x01).unwrap().get_width());
        assert_eq!(1, Id::new_class_a(0x15).unwrap().get_width());
        assert_eq!(1, Id::new_class_a(0x7E).unwrap().get_width());
        assert!(Id::new_class_a(0x7F).is_none());
        assert!(Id::new_class_a(0xFF).is_none());
    }

    #[test]
    fn class_b() {
        assert!(Id::new_class_b(0x00).is_none());
        assert!(Id::new_class_b(0x7E).is_none());
        assert_eq!(2, Id::new_class_b(0x7F).unwrap().get_width());
        assert_eq!(2, Id::new_class_b(0x05A4).unwrap().get_width());
        assert_eq!(2, Id::new_class_b(0x3FFE).unwrap().get_width());
        assert!(Id::new_class_b(0x3FFF).is_none());
        assert!(Id::new_class_b(0xFFFF).is_none());
    }

    #[test]
    fn class_c() {
        assert!(Id::new_class_c(0x00).is_none());
        assert!(Id::new_class_c(0x3FFE).is_none());
        assert_eq!(3, Id::new_class_c(0x3FFF).unwrap().get_width());
        assert_eq!(3, Id::new_class_c(0x001D_B5C3).unwrap().get_width());
        assert_eq!(3, Id::new_class_c(0x001F_FFFE).unwrap().get_width());
        assert!(Id::new_class_c(0x001F_FFFF).is_none());
        assert!(Id::new_class_c(0xFFFF_FFFF).is_none());
    }

    #[test]
    fn class_d() {
        assert!(Id::new_class_d(0x00).is_none());
        assert!(Id::new_class_d(0x001F_FFFE).is_none());
        assert_eq!(4, Id::new_class_d(0x001F_FFFF).unwrap().get_width());
        assert_eq!(4, Id::new_class_d(0x0C0F_FEE0).unwrap().get_width());
        assert_eq!(4, Id::new_class_d(0x0FFF_FFFE).unwrap().get_width());
        assert!(Id::new_class_d(0x0FFF_FFFF).is_none());
        assert!(Id::new_class_d(0xFFFF_FFFF).is_none());
    }
}
