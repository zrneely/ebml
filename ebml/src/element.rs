
// Elements are serialized as their ID, their size, then the data
// Data can be either more elements (in which case the size is the number of them), or a Value.
// If the number of sub-elements in a container is unknown, then the first element not allowed to
// be a child of the container is the next higher-level element.

//! EBML elements, or value with semantic significance.

use {cardinality, EbmlValue, Id, Restriction};
use container::Container;

use typenum;

/// Implement this trait on an empty enum for each element type in your document.
pub trait Element {
    /// The kind of value for elements of this type.
    type Value: EbmlValue;

    /// The valid parent container of elements of this type. If this type is not restricted by its
    /// parent container, set this to the special type `AnyContainer`.
    type AllowedParent: Container;

    /// The minimum possible nesting level of elements of this type. If this type is not restricted
    /// by its parent container, set this to the special type `AnyLevel`.
    type MinAllowedLevel: typenum::Integer;

    /// The maximum possible nesting level of elements of this type. If this type is not restricted
    /// by its parent container, set this to the special type `AnyLevel`.
    ///
    /// If there should be no maximum, set this to a large value. It is recommended to use
    /// `typenum::P8192`.
    type MaxAllowedLevel: typenum::Integer;

    /// The cardinality of elements of this type.
    ///
    /// The default value is `ZeroOrOne`, indicating that the element may either not be present, or
    /// have exactly one instance.
    // TODO set default of ZeroOrOne once associated type defaults become stable
    type Cardinality: cardinality::Cardinality;

    /// The default value for elements of this type.
    ///
    /// The default value is `None`, indicating no default.
    const DEFAULT_VALUE: Option<Self::Value> = None;

    /// The name of the element type. This is a symbolic identifier for the element, and the set of
    /// element and container names must have a 1-to-1 mapping onto the set of element and
    /// container IDs. The legal characters for a name to start with are uppercase letters,
    /// lowercase letters, and the underscore "\_".  Legal characters for the remaining characters
    /// in a name are uppercase letters, lowercase letters, the underscore "\_", and numbers.
    const NAME: &'static str;

    /// Gets the restrictions on values of elements of this type.
    ///
    /// The default implementation returns `None`.
    fn get_restrictions() -> Option<Box<Restriction<Self::Value>>> {
        None
    }

    /// Gets the ID of the element type. This must be a method instead of a associated constant
    /// since there is no way to create an `Id` at compile time. This is for a few reasons, but
    /// mainly the lack of `const fn` and the fact that `Size` allocates from the heap internally
    /// (which may be changed eventually).
    fn get_id() -> Id;
}

/// An element containing some data.
#[derive(Debug)]
pub struct ElementImpl<E: Element> {
    value: E::Value,
}
impl<E: Element> ElementImpl<E> {
    /// Retrieves the actual value of the Element.
    pub fn to_value(&self) -> <E::Value as EbmlValue>::Repr {
        self.value.to_repr()
    }
}
