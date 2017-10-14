
//! Standard EBML containers common to all documents.

use {AnyContainer, Id, cardinality, child_order};
use container::Container;

use typenum;

/// The EBML header which all documents must begin with.
#[derive(Debug)]
pub enum EbmlHeader {}
impl Container for EbmlHeader {
    type Cardinality = cardinality::ZeroOrMany;
    type ChildOrder = child_order::Significant;
    type AllowedParent = AnyContainer;
    type MinAllowedLevel = typenum::Z0;
    type MaxAllowedLevel = typenum::P8192;

    const NAME: &'static str = "EBML";

    fn get_id() -> Id {
        Id::new_class_d(0x0A45DFA3).unwrap()
    }
}

/// The CRC-32 container can be placed around any element or elements; the value in the
/// `CRC32Value` element is the CRC-32 checksum over the other elements.
#[derive(Debug)]
pub enum Crc32Container {}
impl Container for Crc32Container {
    type Cardinality = cardinality::ZeroOrMany;
    type ChildOrder = child_order::Significant;
    type AllowedParent = AnyContainer;
    type MinAllowedLevel = typenum::Z0;
    type MaxAllowedLevel = typenum::P8192;

    const NAME: &'static str = "CRC32";

    fn get_id() -> Id {
        Id::new_class_a(0xC3).unwrap()
    }
}
