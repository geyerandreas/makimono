use napi::bindgen_prelude::Error;
use napi_derive::napi;

#[napi(object)]
pub struct SectionOptions {
    pub label: String,
    pub header: String,
}

#[napi(object)]
pub struct GenerateContentOptions {
    pub start_header: Option<String>,
    pub label_header_prefix: Option<String>,
    pub labels: Option<Vec<SectionOptions>>,
    pub end_regex: Option<String>,
}

fn build_settings(options: Option<GenerateContentOptions>) -> makimono::Settings {
    let mut settings = makimono::Settings::default();

    if let Some(options) = options {
        if let Some(start_header) = options.start_header {
            settings.start_header = start_header;
        }

        if let Some(label_header_prefix) = options.label_header_prefix {
            settings.label_header_prefix = label_header_prefix;
        }

        if let Some(labels) = options.labels {
            settings.labels = labels
                .into_iter()
                .map(|label| makimono::Section {
                    label: label.label,
                    header: label.header,
                })
                .collect();
        }

        if let Some(end_regex) = options.end_regex {
            settings.end_regex = end_regex;
        }
    }

    settings
}

#[napi]
pub fn generate_content(
    content: String,
    message: String,
    labels: Vec<String>,
    options: Option<GenerateContentOptions>,
) -> napi::Result<String> {
    let settings = build_settings(options);
    makimono::generate_content(&content, &settings, message, &labels)
        .map_err(|err| Error::from_reason(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_content_smoke_test() {
        let result = generate_content(
            "### Latest Changes\n".to_string(),
            "* New & first PR Feature".to_string(),
            vec![],
            None,
        );

        assert_eq!(
            result.unwrap(),
            "### Latest Changes\n\n* New & first PR Feature\n"
        );
    }

    #[test]
    fn test_first_entry_with_new_label() {
        let result = generate_content(
            "### Latest Changes\n".to_string(),
            "* New & first PR Feature".to_string(),
            vec!["feature".to_string()],
            Some(GenerateContentOptions {
                start_header: None,
                label_header_prefix: Some("#### ".to_string()),
                labels: Some(vec![SectionOptions {
                    label: "feature".to_string(),
                    header: "Features".to_string(),
                }]),
                end_regex: None,
            }),
        );

        assert_eq!(
            result.unwrap(),
            "### Latest Changes\n\n#### Features\n\n* New & first PR Feature\n"
        );
    }
}
