// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use std::{borrow::Cow, io::Cursor};

/// Helper function to write XML element
///
/// This function takes a reference to a `Writer` object, a string containing
/// the name of the element, and a string containing the value of the element,
///
/// # Arguments
///
/// * `writer` - A reference to a `Writer` object.
/// * `name` - A string containing the name of the element.
/// * `value` - A string containing the value of the element.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - A result indicating success or
///    failure.
///    - `Ok(())` if the element was written successfully.
///    - `Err(Box<dyn std::error::Error>)` if an error occurred during the
///       writing process.
///
pub fn write_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    name: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !value.is_empty() {
        let element_start = BytesStart::new(name);
        writer.write_event(Event::Start(element_start.clone()))?;
        writer
            .write_event(Event::Text(BytesText::from_escaped(value)))?;

        let element_end = BytesEnd::new::<Cow<'static, str>>(
            std::str::from_utf8(
                element_start.name().local_name().as_ref(),
            )
            .unwrap()
            .to_string()
            .into(),
        );

        writer.write_event(Event::End(element_end))?;
    }
    Ok(())
}
