
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
        let value = size.get_value().ok_or(EbmlError::IdOutOfRange)?;

        Ok(match size.get_width() {
            1 if value >  0x00 && value < 0x7F => Id { data: size },
            2 if value >= 0x7F && value < 0x3FFF => Id { data: size },
            3 if value >= 0x3FFF && value < 0x1F_FFFF => Id { data: size },
            4 if value >= 0x1F_FFFF && value < 0x0FFF_FFFF => Id { data: size },

            _ => return Err(EbmlError::IdOutOfRange),
        })
    }

    /// Attempts to write an `Id` to a data source.
    pub(crate) fn write<W: Write>(_target: &mut W) -> EbmlResult<()> {
        unimplemented!("writing not yet supported")
    }

    /// Constructs an EBML Class A ID (width 1) from its literal value, returning `None` if the
    /// value is not in range for the ID. The range of valid values is 0x01 to 0x7E inclusive, so
    /// there are 126 possible Class A IDs.
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
    pub fn new_class_d(data: u32) -> Option<Self> {
        if data < 0x1F_FFFF || data >= 0x0FFF_FFFF {
            None
        } else {
            Some(Id { data: data.into() })
        }
    }

    /// Gets the width of the ID. A width of 1 means the ID is Class A, width of 2 means Class B,
    /// etc.
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
