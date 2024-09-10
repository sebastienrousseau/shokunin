// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

// Import the CnameData model from the `crate::models::data` module.
use crate::models::data::CnameData;

// Import the HashMap collection from the `std::collections` module.
use std::collections::HashMap;

/// Function to create a CnameData object from a HashMap of metadata.
///
/// The `metadata` HashMap must contain a key named `cname` with the value
/// of the CNAME record.
///
/// Returns a CnameData object with the CNAME record value.
///
/// The `cname` value is the canonical name of the domain or subdomain. It is
/// the "true" name of the domain or subdomain, and it is what other DNS
/// records point to. For example, the `cname` value for the `www.example.com`
/// subdomain is typically `example.com`.
///
/// The `cname` value is used to create aliases for domains and subdomains.
/// This can be useful for a variety of reasons, such as:
///
/// * To make it easier for users to remember the domain name or subdomain.
/// * To distribute traffic across multiple servers.
/// * To use a domain name or subdomain with a service provider that does not
///   allow you to create your own DNS records.
pub fn create_cname_data(
    metadata: &HashMap<String, String>,
) -> CnameData {
    // Get the value of the `cname` key from the `metadata` HashMap.
    let cname = metadata.get("cname").cloned().unwrap_or_default();

    // Create a new CnameData object with the CNAME record value.
    CnameData { cname }
}
