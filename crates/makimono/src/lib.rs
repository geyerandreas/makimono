mod config;

use std::collections::HashMap;

use regex::Regex;

/// Parsed content for a single label section in the unreleased changelog area.
#[derive(Debug, Clone)]
pub struct SectionContent {
    /// Label key associated with this section (for example `feature`).
    pub label: String,
    /// Human-readable section header rendered in markdown.
    pub header: String,
    /// Raw markdown items contained in this section.
    pub content: String,
    /// Byte offset in the source release block, used to keep original order.
    pub index: usize,
}

/// Inserts a new changelog `message` into the unreleased section.
///
/// The function looks for `settings.start_header`, parses known label sections,
/// prepends the new message to the first matching label in `labels`, and rebuilds
/// the unreleased block while preserving the rest of the file.
///
/// Returns an error when required regexes cannot be compiled or when the start
/// header cannot be found in `content`.
pub fn generate_content(
    content: &str,
    settings: &config::NewSettings,
    message: String,
    labels: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    // Find header
    let header_regex = Regex::new(&format!("(?m){}", regex::escape(&settings.start_header)))?;
    let header_match = header_regex.find(content).ok_or(format!(
        "Content doesn't contain the header: {}",
        settings.start_header
    ))?;

    let pre_header_content = content[..header_match.end()].trim();

    // Find next release boundary - search in original (untrimmed) content
    let end_regex = Regex::new(&settings.end_regex)?;
    let next_release_match = end_regex.find(&content[header_match.end()..]);

    let release_end = if let Some(m) = next_release_match {
        header_match.end() + m.start()
    } else {
        // There is not next release yet, so we will add to the end of the file
        content.len()
    };

    let release_content = content[header_match.end()..release_end].trim();
    let post_release_content = content[release_end..].trim();

    let mut sections: Vec<SectionContent> = vec![];
    let label_prefix_escaped = regex::escape(&settings.label_header_prefix);

    // Build section content for each label
    for label_config in &settings.labels {
        let section_regex = Regex::new(&format!(
            r"(?m)^{}{}",
            label_prefix_escaped,
            regex::escape(&label_config.header)
        ))?;

        // Find label header inside release_content - e.g search for "#### Features"
        if let Some(label_match) = section_regex.find(release_content) {
            let next_label_regex = Regex::new(&format!(r"(?m)^{}", label_prefix_escaped))?;
            let next_label_match = next_label_regex.find(&release_content[label_match.end()..]);

            let label_section_end = if let Some(m) = next_label_match {
                label_match.end() + m.start()
            } else {
                release_content.len()
            };

            let label_content = release_content[label_match.end()..label_section_end]
                .trim()
                .to_string();

            sections.push(SectionContent {
                label: label_config.label.clone(),
                header: label_config.header.clone(),
                content: label_content,
                index: label_match.start(),
            });
        }
    }

    sections.sort_by_key(|s| s.index);

    let section_keys: HashMap<String, SectionContent> = sections
        .iter()
        .map(|s| (s.label.clone(), s.clone()))
        .collect();

    let mut sectionless_content = String::new();

    if sections.is_empty() {
        sectionless_content = release_content.to_string();
    } else if sections[0].index > 0 {
        sectionless_content = release_content[0..sections[0].index].trim().to_string();
    }

    let mut new_sections: Vec<SectionContent> = vec![];
    let mut found = false;

    // Build new section content for each label, adding the message to the ones that match the PR labels
    for label_config in &settings.labels {
        let mut section = if let Some(existing_section) = section_keys.get(&label_config.label) {
            existing_section.clone()
        } else {
            SectionContent {
                label: label_config.label.clone(),
                header: label_config.header.clone(),
                content: String::new(),
                index: usize::MAX,
            }
        };

        if labels.contains(&label_config.label) && !found {
            found = true;
            let new_content = if section.content.is_empty() {
                message.clone()
            } else {
                format!("{}\n{}", message, section.content)
            };
            section.content = new_content
        }

        new_sections.push(section);
    }

    if !found {
        if sectionless_content.is_empty() {
            sectionless_content = message;
        } else {
            sectionless_content = format!("{}\n{}", message, sectionless_content);
        }
    }

    let mut new_release_content = sectionless_content.clone();
    if !sectionless_content.is_empty() {
        new_release_content = sectionless_content.clone();
    }

    let use_sections: Vec<String> = new_sections
        .iter()
        .filter(|s| !s.content.is_empty())
        .map(|s| {
            format!(
                "{}{}\n\n{}",
                settings.label_header_prefix, s.header, s.content
            )
        })
        .collect();

    let updated_content = use_sections.join("\n\n");

    if !new_release_content.is_empty() && !updated_content.is_empty() {
        new_release_content = format!("{}\n\n{}", new_release_content, updated_content);
    } else if updated_content.is_empty() {
        // Keep new_release_content as-is (only sectionless content)
    } else {
        new_release_content = updated_content;
    }

    let result = if post_release_content.is_empty() {
        format!("{}\n\n{}\n", pre_header_content, new_release_content)
            .trim()
            .to_string()
            + "\n"
    } else {
        format!(
            "{}\n\n{}\n\n{}\n",
            pre_header_content, new_release_content, post_release_content
        )
        .trim()
        .to_string()
            + "\n"
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{config::NewSettings, generate_content};

    #[test]
    fn test_before_release() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            ### 0.1.0

            * Old entry
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New PR Feature by geyerandreas"),
            &[],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                * New PR Feature by geyerandreas

                ### 0.1.0

                * Old entry
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_first_entry() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {"### Latest Changes\n"};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New & first PR Feature"),
            &[],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {"### Latest Changes\n\n* New & first PR Feature\n"}
        );

        Ok(())
    }

    #[test]
    fn test_second_entry() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {"### Latest Changes\n\n* Some previous change\n"};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New exciting Feature"),
            &[],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {"### Latest Changes\n\n* New exciting Feature\n* Some previous change\n"}
        );

        Ok(())
    }

    #[test]
    fn test_without_a_matching_label() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * Existing feature.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(content, &settings, String::from("* New PR Feature"), &[])?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                * New PR Feature

                #### Features

                * Existing feature.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_with_matching_label() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * Existing feature.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New PR Feature."),
            &[String::from("feature")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                #### Features

                * New PR Feature.
                * Existing feature.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_multiple_sections_with_matching_label() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * Existing feature.

            #### Refactors

            * Existing refactor.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New feature."),
            &[String::from("feature")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                #### Features

                * New feature.
                * Existing feature.

                #### Refactors

                * Existing refactor.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_sectionless_content_with_matching_label() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            * Some sectionless entry.

            #### Features

            * Existing feature.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New feature."),
            &[String::from("feature")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                * Some sectionless entry.

                #### Features

                * New feature.
                * Existing feature.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_multiple_matching_labels() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * Existing feature.

            #### Refactors

            * Existing refactor.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New item."),
            &[String::from("feature"), String::from("refactor")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                #### Features

                * New item.
                * Existing feature.

                #### Refactors

                * Existing refactor.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_content_preservation_across_releases() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * New in dev.

            ### 0.2.0

            * Old release content.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* Another new feature."),
            &[String::from("feature")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                #### Features

                * Another new feature.
                * New in dev.

                ### 0.2.0

                * Old release content.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_first_sectionless_entry_after_release() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            ### 0.2.0

            * Old release content.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New special feature."),
            &[],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                * New special feature.

                ### 0.2.0

                * Old release content.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_empty_section_header_with_matching_label() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            #### Refactors

            * Existing refactor.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(
            content,
            &settings,
            String::from("* New feature."),
            &[String::from("feature")],
        )?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                #### Features

                * New feature.

                #### Refactors

                * Existing refactor.
            "#}
        );

        Ok(())
    }

    #[test]
    fn test_no_matching_labels_with_multiple_sections() -> Result<(), Box<dyn std::error::Error>> {
        let content = indoc::indoc! {r#"
            ### Latest Changes

            #### Features

            * Existing feature.

            #### Refactors

            * Existing refactor.
        "#};

        let settings = NewSettings::default();

        let result = generate_content(content, &settings, String::from("* Untagged item."), &[])?;

        assert_eq!(
            result,
            indoc::indoc! {r#"
                ### Latest Changes

                * Untagged item.

                #### Features

                * Existing feature.

                #### Refactors

                * Existing refactor.
            "#}
        );

        Ok(())
    }
}
