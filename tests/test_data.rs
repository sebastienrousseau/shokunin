#[cfg(test)]
mod tests {

    use ssg::models::data::{
        CnameData, FileData, IconData, ManifestData, SiteMapData,
        TxtData,
    };

    #[test]
    fn test_cname_data_default() {
        let cname_data = CnameData::default();
        let expected_cname_data = CnameData {
            cname: String::default(),
        };
        assert_eq!(cname_data, expected_cname_data);
    }

    #[test]
    fn test_file_data_default() {
        let file_data = FileData::default();
        let expected_file_data = FileData {
            cname: String::default(),
            content: String::default(),
            human: String::default(),
            json: String::default(),
            keyword: String::default(),
            name: String::default(),
            rss: String::default(),
            sitemap: String::default(),
            txt: String::default(),
        };
        assert_eq!(file_data, expected_file_data);
    }

    #[test]
    fn test_icon_data_default() {
        let icon_data = IconData::default();
        let expected_icon_data = IconData {
            src: String::default(),
            sizes: String::default(),
            icon_type: None,
            purpose: None,
        };
        assert_eq!(icon_data, expected_icon_data);
    }

    #[test]
    fn test_manifest_options_default() {
        let manifest_options = ManifestData::default();
        let expected_manifest_options = ManifestData {
            background_color: String::default(),
            description: String::default(),
            display: String::default(),
            icons: Vec::new(),
            name: String::default(),
            orientation: String::default(),
            scope: String::default(),
            short_name: String::default(),
            start_url: String::default(),
            theme_color: String::default(),
        };
        assert_eq!(manifest_options, expected_manifest_options);
    }

    #[test]
    fn test_sitemap_data_default() {
        let sitemap_data = SiteMapData::default();
        let expected_sitemap_data = SiteMapData {
            loc: String::default(),
            lastmod: String::default(),
            changefreq: String::default(),
        };
        assert_eq!(sitemap_data, expected_sitemap_data);
    }

    #[test]
    fn test_txt_data_default() {
        let txt_data = TxtData::default();
        let expected_txt_data = TxtData {
            permalink: String::default(),
        };
        assert_eq!(txt_data, expected_txt_data);
    }
}
