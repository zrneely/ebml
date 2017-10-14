
//! Reading EBML documents

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use std::ops::Add;

use typenum;

use {cardinality, child_order, AnyContainer, AnyLevel, EbmlResult, Id, Size};
use element::Element;
use std_containers::EbmlHeader;

/// A source for elements in a container.
#[derive(Debug)]
pub struct ContainerReader<C: Container, L, R: Read + Seek> {
    _c: PhantomData<C>,
    _l: PhantomData<L>,

    source: R,
}
impl<C, L> ContainerReader<C, L>
where
    L: Add<typenum::P1>,
    C: Container<ChildOrder = child_order::Significant>,
    R: Read,
{
    /// Reads all values in this container of the given type. Use this method when:
    ///
    /// * The element may occur zero or many times in the container.
    /// * The element is restricted by allowed parent, and not by allowed level.
    pub fn read_zero_or_many_values_by_container<T>(&self) -> EbmlResult<Vec<T::Value>>
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

    // TODO other value-reading methods


    /// Reads the child container of this container of the given type. Use this method when:
    ///
    /// * The child may occur zero or one times in the container.
    /// * The child is restricted by allowed parent, and not by allowed level.
    pub fn read_zero_or_one_children_by_container<NC: Container>(
        &self
    ) -> EbmlResult<Option<ContainerReader<NC, typenum::Sum<L, typenum::P1>, R>>>
    where
        NC: Container<
            Cardinality = cardinality::ZeroOrOne,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        // Ensure that the ID in the source matches the expected ID. If not, then the element does
        // not occur and we can return Ok(None). Alternatively, it could be an invalid ID, in which
        // case the document as a whole is invalid.
        let expected_id = NC::get_id();
        let actual_id = Id::load(&self.source);
    }

    // TODO other child-reading methods
}

pub fn read_document<P: AsRef<Path>>(
    path: P
) -> EbmlResult<ContainerReader<EbmlHeader, typenum::Z0>> {
    ContainerReader {
        _c: PhantomData,
        _l: PhantomData,

        source: File::open(path)?,
    }
}
