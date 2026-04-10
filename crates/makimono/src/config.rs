#[derive(Debug, Clone)]
pub struct Section {
    pub label: String,
    pub header: String,
}

macro_rules! section {
    ($label:expr, $header:expr) => {
        Section {
            label: $label.to_string(),
            header: $header.to_string(),
        }
    };
}

pub struct NewSettings {
    pub start_header: String,
    pub label_header_prefix: String,
    pub labels: Vec<Section>,
    pub end_regex: String,
}

impl Default for NewSettings {
    fn default() -> Self {
        Self {
            start_header: String::from("### Latest Changes"),
            label_header_prefix: String::from("#### "),
            end_regex: String::from("(?m)(^### .*)|(^## .*)"),
            labels: vec![
                section!("breaking", "Breaking Changes"),
                section!("bug", "Fixes"),
                section!("docs", "Docs"),
                section!("feature", "Features"),
                section!("infra", "Infrastructure"),
                section!("internal", "Internal"),
                section!("lang", "Translations"),
                section!("refactor", "Refactors"),
                section!("security", "Security Fixes"),
                section!("upgrade", "Upgrades"),
            ],
        }
    }
}
