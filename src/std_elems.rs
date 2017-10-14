
//! Standard EBML elements common to all documents.

use {AnyContainer, AnyLevel, BinaryValue, Id, UintValue, Restriction, cardinality};
use element::Element;
use std_containers::EbmlHeader;

use typenum;

/// A member of the header; the EBML version the document conforms to.
#[derive(Debug)]
pub enum EbmlVersion {}
impl Element for EbmlVersion {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "EBMLVersion";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(1));

    fn get_id() -> Id {
        Id::new_class_b(0x4286).unwrap()
    }
}

/// A member of the header; the minimum EBML version a parser must be aware of to read the
/// document.
#[derive(Debug)]
pub enum EbmlReadVersion {}
impl Element for EbmlReadVersion {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "EBMLReadVersion";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(1));

    fn get_id() -> Id {
        Id::new_class_b(0x42F7).unwrap()
    }
}

/// A member of the header; an upper bound on the width of ID used in the document.
#[derive(Debug)]
pub enum EbmlMaxIdWidth {}
impl Element for EbmlMaxIdWidth {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "EBMLMaxIDWidth";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(4));

    fn get_id() -> Id {
        Id::new_class_b(0x42F2).unwrap()
    }
}

/// A member of the header; an upper bound on the width of size used in the document.
#[derive(Debug)]
pub enum EbmlMaxSizeWidth {}
impl Element for EbmlMaxSizeWidth {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "EBMLMaxSizeWidth";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(8));

    fn get_id() -> Id {
        Id::new_class_b(0x42F3).unwrap()
    }
}

/// A member of the header; an ASCII string that identifies the type of document.
#[derive(Debug)]
pub enum DocType {}
impl Element for DocType {
    type Value = BinaryValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "DocType";

    fn get_id() -> Id {
        Id::new_class_b(0x4282).unwrap()
    }

    fn get_restrictions() -> Option<Box<Restriction<BinaryValue>>> {
        // TODO 32 .. 126
        unimplemented!()
    }
}

/// A member of the header; the version of the document type to which this document conforms.
#[derive(Debug)]
pub enum DocTypeVersion {}
impl Element for DocTypeVersion {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "DocTypeVersion";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(1));

    fn get_id() -> Id {
        Id::new_class_b(0x4287).unwrap()
    }
}

/// A member of the header; the minimum version of the document type an interpreter has to support
/// to be able to read the document.
#[derive(Debug)]
pub enum DocTypeReadVersion {}
impl Element for DocTypeReadVersion {
    type Value = UintValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "DocTypeReadVersion";
    const DEFAULT_VALUE: Option<Self::Value> = Some(UintValue::Uint1(1));

    fn get_id() -> Id {
        Id::new_class_b(0x4285).unwrap()
    }
}

/// The actual computed CRC-32 checksum over elements in a `Crc32` container.
#[derive(Debug)]
pub enum Crc32Value {}
impl Element for Crc32Value {
    type Value = BinaryValue;
    type Cardinality = cardinality::ZeroOrOne;
    type AllowedParent = EbmlHeader;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "CRC32Value";

    fn get_id() -> Id {
        Id::new_class_b(0x42FE).unwrap()
    }

    //fn validate(value: &Value) -> bool {
    //    if let Some(value) = value.as_binary() {
    //        value.len() == 4
    //    } else {
    //        false
    //    }
    //}
}

/// An element whose data is ignored.
#[derive(Debug)]
pub enum Void {}
impl Element for Void {
    const NAME: &'static str = "Void";
    type Value = BinaryValue;
    type Cardinality = cardinality::ZeroOrMany;
    type AllowedParent = AnyContainer;
    type MinAllowedLevel = typenum::P1;
    type MaxAllowedLevel = typenum::P8192;

    fn get_id() -> Id {
        Id::new_class_a(0xEC).unwrap()
    }
}
