
#![deny(missing_docs, missing_debug_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code, unused)]
#![warn(dead_code)]

#![feature(specialization)]
#![feature(conservative_impl_trait)]

//! This library provides tools for reading and writing documents in the Extensible Binary Markup
//! Language format. Like XML, EBML is an extensible format with many possible elements, and has a
//! concept of a "EBML Document Type Definition", or EDTD, a specification of the various element
//! types and structures for each use case of the format. The EDTD format is theoretically
//! computer-readable; using this library requires encoding the elements defined in an EDTD into
//! (empty) Rust types. Eventually, this library may support doing that automatically using
//! procedural macros.
//!
//! This library provides a traits for elements and containers to implement, several default
//! elements common to every EBML document, and serialization/deserialization of EBML documents.
//!
//! EBML is primarily used as the base document format for the Matroska media container format.
//!
//! ## Features
//!
//! Enable the "chrono" cargo feature in order to support conversion between `chrono` dates and
//! EBML dates.
//!
//! ## Errata
//!
//! This library attempts to be a complete implementation of the EBML specification. There are a
//! few places it lacks:
//!
//! * The EBML specification allows 10-byte (80 bit) floating point numbers to be stored and read.
//!   Rust lacks support for a hypothetical `f80` type, so when encountering a value of this type,
//!   this crate treats it as binary data (a `[u8; 10]`).
//! * The specification allows restricting the nesting level of an element to an interval which is
//!   unbounded on the right (without a maximum). In order to represent nesting levels at the type
//!   level, this is not supported. Instead, you should set the maximum allowed level to a large
//!   number for such elements. It is recommended to use `typenum::P8192` for this purpose.
//! * The specification forbids elements whose container restrictions are by nesting level from
//!   having unknown sizes. This crate does not enforce that rule.
//! * This crate does not enforce the restriction that elements which must occur one or more times
//!   in a container actually do so when building a container; it is enforced when parsing, as long
//!   as you actually request it.

#[cfg(feature = "chrono")]
extern crate chrono;
extern crate typenum;

pub mod read;
pub mod restrictions;
pub mod std_elems;
pub mod std_containers;
pub mod value;

mod container;
mod element;
mod error;
mod id;
mod peek;
mod size;

pub use container::{Container, ContainerImpl, root_container};
pub use error::{EbmlError, EbmlResult};
pub use element::{Element, ElementImpl};
pub use id::Id;
pub use restrictions::*;
pub use size::Size;
pub use value::*;

/// Set an `Element`'s `MinAllowedLevel` and `MaxAllowedLevel` to this type to show that the element
/// is not restricted by nesting level.
pub type AnyLevel = typenum::N1;

/// Set an `Element`'s `AllowedParent` to this type to show that the element is not restricted by
/// parent container.
#[derive(Debug)]
pub enum AnyContainer {}
impl container::Container for AnyContainer {
    type Cardinality = cardinality::ZeroOrMany;
    type ChildOrder = child_order::Insignificant;
    type AllowedParent = AnyContainer;
    type MinAllowedLevel = AnyLevel;
    type MaxAllowedLevel = AnyLevel;
    const NAME: &'static str = "do not use";

    fn get_id() -> Id {
        unreachable!("get_id called on dummy container")
    }
}

/// Cardinalities describe the number of containers or elements which can sit in a container.
pub mod cardinality {
    /// A marker trait for types that define a Cardinality. This should only be implemented for
    /// types defined in this crate.
    pub trait Cardinality {}

    /// A cardinality indicating that the element may occur any number of times, including zero.
    #[derive(Debug)]
    pub enum ZeroOrMany {}
    impl Cardinality for ZeroOrMany {}

    /// A cardinality indicating that the element may occur either once or not at all.
    #[derive(Debug)]
    pub enum ZeroOrOne {}
    impl Cardinality for ZeroOrOne {}

    /// A cardinality indicating that the element must occur exactly once in each scope it is legal
    /// for it to do so.
    #[derive(Debug)]
    pub enum ExactlyOne {}
    impl Cardinality for ExactlyOne {}

    /// A cardinality indicating that the element must occur at least once in each scope it is
    /// legal for it to do so.
    #[derive(Debug)]
    pub enum OneOrMany {}
    impl Cardinality for OneOrMany {}
}

/// The child order of a container signifies if the order of its elements is significant.
pub mod child_order {
    /// A `Container` may either be ordered or unordered.
    pub trait ChildOrder {}

    /// Indicates that the order of children in a `Container` is significant.
    #[derive(Debug)]
    pub enum Significant {}
    impl ChildOrder for Significant {}

    /// Indicates that the order of children in a `Container` is insignificant.
    #[derive(Debug)]
    pub enum Insignificant {}
    impl ChildOrder for Insignificant {}
}
