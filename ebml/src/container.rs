
// Elements are serialized as their ID, their size, then the data
// Data can be either more elements (in which case the size is the number of them), or a Value.
// If the number of sub-elements in a container is unknown, then the first element not allowed to
// be a child of the container is the next higher-level element.

//! EBML containers, which are values containing a list of more elements.

use std::marker::PhantomData;
use std::ops::Add;

use typenum;

use {cardinality, Id, EbmlResult, child_order, Size, AnyLevel, AnyContainer};
use element::Element;
use std_containers::EbmlHeader;

/// Implement this trait on an empty enum for each container type in your document.
pub trait Container {
    /// The cardinality of a container (how many times it is required/allowed to appear in each
    /// valid scope).
    // TODO once associated type defaults are stable, set it to ZeroOrOne
    type Cardinality: cardinality::Cardinality;

    /// The importance of the order of elements in a container.
    // TODO once associated type defaults are stable, set it to child_order::Significant
    type ChildOrder: child_order::ChildOrder;

    /// The valid parent container of containers of this type. If this type is not restricted by
    /// its parent container, set this to the special type `AnyContainer`.
    type AllowedParent: Container;

    /// The minimum possible nesting level of elements of this type. If this type is not restricted
    /// by its parent container, set this to the special type `AnyLevel`.
    type MinAllowedLevel: typenum::Integer;

    /// The maximum possible nesting level of containers of this type. If this type is not
    /// restricted by its parent container, set this to the special type `AnyLevel`.
    ///
    /// If there should be no maximum, set this to a large value. It is recommended to use
    /// `typenum::P8192`.
    type MaxAllowedLevel: typenum::Integer;

    /// The name of the container type. This is a symbolic identifier for the container, and the
    /// set of element and container names must have a 1-to-1 mapping onto the set of element and
    /// container IDs. The legal characters for a name to start with are uppercase letters,
    /// lowercase letters, and the underscore "\_".  Legal characters for the remaining characters
    /// in a name are uppercase letters, lowercase letters, the underscore "\_", and numbers.
    const NAME: &'static str;

    /// Gets the ID of the container type. This must be a method instead of a associated constant
    /// since there is no way to create an `Id` at compile time. This is for a few reasons, but
    /// mainly the lack of `const fn` and the fact that `Size` allocates from the heap internally
    /// (which may be changed eventually).
    fn get_id() -> Id;
}

/// A container containing one or more elements or containers. The second type parameter is the
/// nesting level of the container.
#[derive(Debug)]
pub struct ContainerImpl<C: Container, L> {
    _c: PhantomData<C>,
    _l: PhantomData<L>,
}

impl<C, L> ContainerImpl<C, L>
where
    L: Add<typenum::P1>,
    C: Container<ChildOrder = child_order::Insignificant>,
{
    // What I wouldn't give for the usability improvements of overloading...

    /// Finds all values in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or many times in the container.
    /// * The element is restricted by allowed parent, and not by allowed level.
    pub fn get_zero_or_many_values_by_container<T>(&self) -> EbmlResult<Vec<T::Value>>
    where
        T: Element<
            Cardinality = cardinality::ZeroOrMany,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        unimplemented!()
    }

    /// Finds all values in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or more times in the container.
    /// * The element is restricted by allowed level, and not by allowed parent.
    pub fn get_zero_or_many_values_by_level<T>(&self) -> EbmlResult<Vec<T::Value>>
    where
        T: Element<Cardinality = cardinality::ZeroOrMany, AllowedParent = AnyContainer>,
        T::MaxAllowedLevel: typenum::IsGreater<L, Output = typenum::True>,
        T::MinAllowedLevel: typenum::IsLess<L, Output = typenum::True>,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or one times in the container.
    /// * The element is restricted by allowed parent, and not by allowed level.
    pub fn get_zero_or_one_value_by_container<T>(&self) -> EbmlResult<Option<T::Value>>
    where
        T: Element<
            Cardinality = cardinality::ZeroOrOne,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or one times in the container.
    /// * The element is restricted by allowed level, and not by allowed parent.
    pub fn get_zero_or_one_value_by_level<T>(&self) -> EbmlResult<Option<T::Value>>
    where
        T: Element<Cardinality = cardinality::ZeroOrOne, AllowedParent = AnyContainer>,
        T::MaxAllowedLevel: typenum::IsGreater<L, Output = typenum::True>,
        T::MinAllowedLevel: typenum::IsLess<L, Output = typenum::True>,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element must occur exactly once in the container.
    /// * The element is restricted by allowed parent, and not by allowed level.
    pub fn get_exactly_one_value_by_container<T>(&self) -> EbmlResult<T::Value>
    where
        T: Element<
            Cardinality = cardinality::ExactlyOne,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or one times in the container.
    /// * The element is restricted by allowed level, and not by allowed parent.
    pub fn get_exactly_one_value_by_level<T>(&self) -> EbmlResult<T::Value>
    where
        T: Element<Cardinality = cardinality::ExactlyOne, AllowedParent = AnyContainer>,
        T::MaxAllowedLevel: typenum::IsGreater<L, Output = typenum::True>,
        T::MinAllowedLevel: typenum::IsLess<L, Output = typenum::True>,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element must occur once but may occur multiple times in the container.
    /// * The element is restricted by allowed parent, and not by allowed level.
    pub fn get_one_or_many_values_by_container<T>(&self) -> EbmlResult<(T::Value, Vec<T::Value>)>
    where
        T: Element<
            Cardinality = cardinality::OneOrMany,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        unimplemented!()
    }

    /// Finds the value in this container of the given type. Use this method when:
    ///
    /// * The element must occur once but may occur multiple times in the container.
    /// * The element is restricted by allowed level, and not by allowed parent.
    pub fn get_one_or_many_values_by_level<T>(&self) -> EbmlResult<(T::Value, Vec<T::Value>)>
    where
        T: Element<Cardinality = cardinality::OneOrMany, AllowedParent = AnyContainer>,
        T::MaxAllowedLevel: typenum::IsGreater<L, Output = typenum::True>,
        T::MinAllowedLevel: typenum::IsLess<L, Output = typenum::True>,
    {
        unimplemented!()
    }

    /// Finds the child container of this container of the given type. Use this method when:
    ///
    /// * The child may occur zero or one times in the container.
    /// * The child is restricted by allowed level, and not by allowed parent.
    pub fn get_zero_or_one_child_by_level<NC: Container>(
        &self,
    ) -> ContainerImpl<NC, typenum::Sum<L, typenum::P1>>
    where
        NC: Container<Cardinality = cardinality::ZeroOrOne, AllowedParent = AnyContainer>,
        NC::MaxAllowedLevel: typenum::IsGreater<L, Output = typenum::True>,
        NC::MinAllowedLevel: typenum::IsLess<L, Output = typenum::True>,
    {
        unimplemented!()
    }

    /// Returns the number of children in this container.
    pub fn len(&self) -> Size {
        unimplemented!()
    }

    /// Returns true if the container has no children.
    pub fn is_empty(&self) -> bool {
        self.len().get_value().map(|x| x > 0).unwrap_or(false)
    }
}

/// Retrieves an EBML root container.
pub fn root_container() -> ContainerImpl<EbmlHeader, typenum::Z0> {
    ContainerImpl {
        _c: PhantomData,
        _l: PhantomData,
    }
}
