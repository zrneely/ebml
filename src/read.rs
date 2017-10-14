
//! Reading EBML documents

use std::borrow::BorrowMut;
use std::io::Read;
use std::marker::PhantomData;
use std::ops::Add;

use typenum;

use {cardinality, child_order, AnyLevel, Container, EbmlResult, Id, Size};
use element::Element;
use error::EbmlError;
use peek::PeekableReader;
use std_containers::EbmlHeader;

/// A source for elements in a container.
#[derive(Debug)]
pub struct ContainerReader<C: Container, L, R: Read, B: BorrowMut<PeekableReader<R>>> {
    _c: PhantomData<C>,
    _l: PhantomData<L>,
    _r: PhantomData<R>,

    source: B,
    length: Size,
}
impl<C, L, R, B> ContainerReader<C, L, R, B>
where
    L: Add<typenum::P1>,
    C: Container<ChildOrder = child_order::Significant>,
    R: Read,
    B: BorrowMut<PeekableReader<R>>,
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
        &mut self
    ) -> EbmlResult<Option<ContainerReader<
            NC,
            typenum::Sum<L, typenum::P1>,
            R,
            &mut PeekableReader<R>
         >>>
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
        Ok(if NC::get_id() == Id::load(self.source.borrow_mut())? {
            Some(ContainerReader {
                _c: PhantomData, _l: PhantomData, _r: PhantomData,

                length: Size::load(self.source.borrow_mut())?,
                source: self.source.borrow_mut(),
            })
        } else {
            None
        })
    }

    pub fn read_zero_or_many_children_by_container<NC: Container>(
        &mut self
    ) -> EbmlResult<impl Iterator<Item = ContainerReader<
            NC,
            typenum::Sum<L, typenum::P1>,
            R,
            &mut PeekableReader<R>,
        >>>
    where
        NC: Container<
            Cardinality = cardinality::ZeroOrMany,
            MinAllowedLevel = AnyLevel,
            MaxAllowedLevel = AnyLevel,
            AllowedParent = C,
        >,
    {
        let mut result = Vec::new();
        while NC::get_id() == Id::load(self.source.borrow_mut())? {
            result.push(ContainerReader {
                _c: PhantomData, _l: PhantomData, _r: PhantomData,

                length: Size::load(self.source.borrow_mut())?,
                source: self.source.borrow_mut(),
            });
        }
        Ok(result)
    }

    // TODO other child-reading methods
}

/// Reads an EBML document, producing the root container.
pub fn read_document<R: Read>(
    source: R
) -> EbmlResult<ContainerReader<EbmlHeader, typenum::Z0, R, PeekableReader<R>>> {
    let mut source = PeekableReader::new(source)?;
    if EbmlHeader::get_id() == Id::load(&mut source)? {
        Ok(ContainerReader {
            _c: PhantomData, _l: PhantomData, _r: PhantomData,

            length: Size::load(&mut source)?,
            source: source,
        })
    } else {
        Err(EbmlError::WrongId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn load_vaild_document() {
        // This is the header of a valid document
        let data = include_bytes!("../tests/min_valid_header");
        let cursor = Cursor::new(data);

        let _doc = read_document(cursor).unwrap();
    }
}
