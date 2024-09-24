// Copyright © 2024 Shokunin Static Site Generator. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents the different versions of RSS.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
pub enum RssVersion {
    /// RSS version 0.90
    RSS0_90,
    /// RSS version 0.91
    RSS0_91,
    /// RSS version 0.92
    RSS0_92,
    /// RSS version 1.0
    RSS1_0,
    /// RSS version 2.0
    RSS2_0,
}

impl RssVersion {
    /// Returns the string representation of the RSS version.
    pub fn as_str(&self) -> &'static str {
        match self {
            RssVersion::RSS0_90 => "0.90",
            RssVersion::RSS0_91 => "0.91",
            RssVersion::RSS0_92 => "0.92",
            RssVersion::RSS1_0 => "1.0",
            RssVersion::RSS2_0 => "2.0",
        }
    }
}

impl Default for RssVersion {
    fn default() -> Self {
        RssVersion::RSS2_0
    }
}

impl fmt::Display for RssVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RssVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.90" => Ok(RssVersion::RSS0_90),
            "0.91" => Ok(RssVersion::RSS0_91),
            "0.92" => Ok(RssVersion::RSS0_92),
            "1.0" => Ok(RssVersion::RSS1_0),
            "2.0" => Ok(RssVersion::RSS2_0),
            _ => Err(format!("Invalid RSS version: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_version_as_str() {
        assert_eq!(RssVersion::RSS0_90.as_str(), "0.90");
        assert_eq!(RssVersion::RSS0_91.as_str(), "0.91");
        assert_eq!(RssVersion::RSS0_92.as_str(), "0.92");
        assert_eq!(RssVersion::RSS1_0.as_str(), "1.0");
        assert_eq!(RssVersion::RSS2_0.as_str(), "2.0");
    }

    #[test]
    fn test_rss_version_default() {
        assert_eq!(RssVersion::default(), RssVersion::RSS2_0);
    }

    #[test]
    fn test_rss_version_display() {
        assert_eq!(format!("{}", RssVersion::RSS0_90), "0.90");
        assert_eq!(format!("{}", RssVersion::RSS0_91), "0.91");
        assert_eq!(format!("{}", RssVersion::RSS0_92), "0.92");
        assert_eq!(format!("{}", RssVersion::RSS1_0), "1.0");
        assert_eq!(format!("{}", RssVersion::RSS2_0), "2.0");
    }

    #[test]
    fn test_rss_version_from_str() {
        assert_eq!(
            "0.90".parse::<RssVersion>(),
            Ok(RssVersion::RSS0_90)
        );
        assert_eq!(
            "0.91".parse::<RssVersion>(),
            Ok(RssVersion::RSS0_91)
        );
        assert_eq!(
            "0.92".parse::<RssVersion>(),
            Ok(RssVersion::RSS0_92)
        );
        assert_eq!("1.0".parse::<RssVersion>(), Ok(RssVersion::RSS1_0));
        assert_eq!("2.0".parse::<RssVersion>(), Ok(RssVersion::RSS2_0));
        assert!("3.0".parse::<RssVersion>().is_err());
    }

    #[test]
    fn test_rss_version_serialization() {
        let version = RssVersion::RSS2_0;
        let serialized = serde_json::to_string(&version).unwrap();
        assert_eq!(serialized, "\"RSS2_0\"");
    }

    #[test]
    fn test_rss_version_deserialization() {
        let deserialized: RssVersion =
            serde_json::from_str("\"RSS1_0\"").unwrap();
        assert_eq!(deserialized, RssVersion::RSS1_0);
    }
}
