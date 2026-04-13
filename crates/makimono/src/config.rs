/// A configurable changelog section mapped from a label to a human-readable header.
#[derive(Debug, Clone)]
pub struct Section {
    /// The machine-friendly label used for matching (for example `feature`).
    pub label: String,
    /// The changelog heading shown in markdown (for example `Features`).
    pub header: String,
}

/// Creates a [`Section`] using string literals or any string-like expressions.
macro_rules! section {
    ($label:expr, $header:expr) => {
        Section {
            label: $label.to_string(),
            header: $header.to_string(),
        }
    };
}

/// Settings that control how new changelog entries are inserted into a markdown file.
pub struct Settings {
    /// The header that marks the start of the editable unreleased section.
    pub start_header: String,
    /// Prefix used for subsection headings inside the unreleased section.
    pub label_header_prefix: String,
    /// Ordered list of known labels and their rendered subsection headers.
    pub labels: Vec<Section>,
    /// Regex used to detect where the current unreleased section ends.
    pub end_regex: String,
}

impl Default for Settings {
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
