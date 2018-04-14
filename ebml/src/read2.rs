
//! This module provides the necessary functionality to parse and safely interpret EBML documents.
//! In general, you provide a callback for each type of value you plan on reading, then call the
//! `parse_document` method, which calls the callbacks as necessary.

fn example() {
    let mut document: Container = Parser::<CustomType>::load("test.ebml");

    for event in doc.events() {
        match event {
            Events::Child_EBML(header) => {

            },
            Events::Value(value) => {

            },
            Events::End => {

            },
        }
    }
}


