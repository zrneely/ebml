
/// Use this macro to generate code to parse and write a document type.
/// Example usage:
///
/// ```rust
/// use ebml::std_elems::*;
///
/// ebml_document! {
///     name: Matroska,
///     HeaderContainer {
///         EbmlVersion,
///         EbmlReadVersion,
///         EbmlMaxIdWidth,
///         EbmlMaxSizeWidth,
///         DocType,
///         DocTypeVersion,
///     }
/// }
/// ```
///
/// This will generate a struct named `Matroska`.
macro_rules! ebml_document {
    (name: $doc_name:ident, $(
        $container:ty {
            $(
                $subdata:ty
            )*
        }
    )*) => {

    }
}
