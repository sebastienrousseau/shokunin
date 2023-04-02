use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use std::{borrow::Cow, io::Cursor};

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// RSS options struct to hold the options for the RSS feed.
pub struct RssOptions {
    /// The title of the RSS feed.
    pub title: String,
    /// The link to the RSS feed.
    pub link: String,
    /// The description of the RSS feed.
    pub description: String,
    /// The link to the RSS feed.
    pub atom_link: String,
    /// The last build date of the RSS feed.
    pub last_build_date: String,
    /// The publication date of the RSS feed.
    pub pub_date: String,
    /// The generator of the RSS feed.
    pub generator: String,
    /// The title of the RSS feed item.
    pub item_title: String,
    /// The link to the RSS feed item.
    pub item_link: String,
    /// The GUID of the RSS feed item.
    pub item_guid: String,
    /// The description of the RSS feed item.
    pub item_description: String,
    /// The publication date of the RSS feed item.
    pub item_pub_date: String,
}

impl RssOptions {
    /// Create a new `RssOptions` struct with default values.
    pub fn new() -> RssOptions {
        RssOptions {
            title: "".to_string(),
            link: "".to_string(),
            description: "".to_string(),
            atom_link: "".to_string(),
            last_build_date: "".to_string(),
            pub_date: "".to_string(),
            generator: "".to_string(),
            item_title: "".to_string(),
            item_link: "".to_string(),
            item_guid: "".to_string(),
            item_description: "".to_string(),
            item_pub_date: "".to_string(),
        }
    }
}
/// ## Function: generate_rss - returns a Result containing a String
///
/// Generates an RSS feed from the given `RssOptions` struct.
///
/// If an error occurs while generating the RSS feed, such as the
/// `RssOptions` struct not containing the required fields, an error is
/// printed to the console and the feed is not generated. If the RSS
/// feed is generated successfully, the function returns a `String`
/// containing the RSS feed.
///
/// # Arguments
///
/// - `options`: A `RssOptions` struct containing the options for the
/// RSS feed.
///
/// # Returns
///
/// A `Result<String, Box<dyn std::error::Error>>` containing the RSS
/// feed, or an error if the RSS feed cannot be generated.
///
pub fn generate_rss(
    options: &RssOptions,
) -> Result<String, Box<dyn std::error::Error>> {
    // Write the XML declaration
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("utf-8"),
        None,
    )))?;

    // Write the <rss> opening tag with version attribute
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start
        .push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    writer.write_event(Event::Start(rss_start))?;

    // Write the <channel> opening tag
    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    // Write the required channel elements
    let channel_elements = [
        ("title", &options.title),
        ("link", &options.link),
        ("description", &options.description),
        ("atom:link", &options.atom_link),
        ("lastBuildDate", &options.last_build_date),
        ("pubDate", &options.pub_date),
        ("generator", &options.generator),
    ];

    // Write each channel element that has a non-empty value
    for &(element, value) in channel_elements.iter() {
        // Write the element only if the value is not empty
        if !value.is_empty() {
            let element_start = BytesStart::new(element);
            writer.write_event(Event::Start(element_start.clone()))?;
            writer.write_event(Event::Text(
                BytesText::from_escaped(value),
            ))?;

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
    }

    // Write the <item> opening tag
    writer.write_event(Event::Start(BytesStart::new("item")))?;

    // Write the required item elements
    let item_elements = [
        ("item_title", &options.item_title),
        ("item_link", &options.item_link),
        ("item_guid", &options.item_guid),
        ("item_description", &options.item_description),
        ("item_pub_date", &options.item_pub_date),
    ];

    // Write each item element that has a non-empty value
    for &(element, value) in item_elements.iter() {
        // Write the element only if the value is not empty
        if !value.is_empty() {
            let element_start = BytesStart::new(element);
            writer.write_event(Event::Start(element_start.clone()))?;
            writer.write_event(Event::Text(
                BytesText::from_escaped(value),
            ))?;
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
    }

    // Write the <item> closing tag
    writer.write_event(Event::End(BytesEnd::new("item")))?;

    // Write the <channel> closing tag
    writer.write_event(Event::End(BytesEnd::new("channel")))?;

    // Write the <rss> closing tag
    writer.write_event(Event::End(BytesEnd::new("rss")))?;

    // Convert the XML into a UTF-8 string
    let xml = writer.into_inner().into_inner();
    let rss_str = String::from_utf8(xml)?;

    Ok(rss_str)
}
