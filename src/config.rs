use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG_FILE_NAME: &str = ".verilog-format.yaml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSettings {
    pub settings: FormatSettings,
    pub source: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IndentType {
    #[default]
    Space,
    Tab,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FormatSettings {
    #[serde(default = "default_indent_width", alias = "IndentWidth")]
    pub indent_width: usize,

    #[serde(default, alias = "IndentType")]
    pub indent_type: IndentType,

    #[serde(
        default = "default_spaces_before_trailing_comments",
        alias = "SpacesBeforeTrailingComments"
    )]
    pub spaces_before_trailing_comments: usize,

    #[serde(
        default = "default_spaces_after_trailing_comments",
        alias = "SpacesAfterTrailingComments"
    )]
    pub spaces_after_trailing_comments: usize,

    #[serde(
        default = "default_spaces_before_if_statement",
        alias = "SpacesBeforeIfStatement"
    )]
    pub spaces_before_if_statement: usize,

    #[serde(
        default = "default_spaces_blocking_assignment",
        alias = "SpacesBlockingAssignment"
    )]
    pub spaces_blocking_assignment: usize,

    #[serde(
        default = "default_spaces_no_blocking_assignment",
        alias = "SpacesNoBlockingAssignment"
    )]
    pub spaces_no_blocking_assignment: usize,

    #[serde(default = "default_false", alias = "SpacesInParentheses")]
    pub spaces_in_parentheses: bool,

    #[serde(default = "default_false", alias = "SpacesInSquareBrackets")]
    pub spaces_in_square_brackets: bool,

    #[serde(default = "default_true", alias = "AlignBlockingAssignments")]
    pub align_blocking_assignments: bool,

    #[serde(default = "default_true", alias = "AlignNoBlockingAssignments")]
    pub align_no_blocking_assignments: bool,

    #[serde(default = "default_false", alias = "AlignLineComments")]
    pub align_line_comments: bool,
}

impl Default for FormatSettings {
    fn default() -> Self {
        Self {
            indent_width: default_indent_width(),
            indent_type: IndentType::default(),
            spaces_before_trailing_comments: default_spaces_before_trailing_comments(),
            spaces_after_trailing_comments: default_spaces_after_trailing_comments(),
            spaces_before_if_statement: default_spaces_before_if_statement(),
            spaces_blocking_assignment: default_spaces_blocking_assignment(),
            spaces_no_blocking_assignment: default_spaces_no_blocking_assignment(),
            spaces_in_parentheses: default_false(),
            spaces_in_square_brackets: default_false(),
            align_blocking_assignments: default_true(),
            align_no_blocking_assignments: default_true(),
            align_line_comments: default_false(),
        }
    }
}

impl FormatSettings {
    pub fn indent_unit(&self) -> &str {
        match self.indent_type {
            IndentType::Space => " ",
            IndentType::Tab => "\t",
        }
    }

    pub fn indent(&self, level: usize) -> String {
        self.indent_unit()
            .repeat(self.indent_width.saturating_mul(level))
    }
}

pub fn resolve_settings(explicit_path: Option<&Path>, cwd: &Path) -> Result<LoadedSettings> {
    if let Some(path) = explicit_path {
        let settings = read_settings(path)?;
        return Ok(LoadedSettings {
            settings,
            source: Some(path.to_path_buf()),
        });
    }

    let local_path = cwd.join(DEFAULT_CONFIG_FILE_NAME);
    if local_path.exists() {
        let settings = read_settings(&local_path)?;
        return Ok(LoadedSettings {
            settings,
            source: Some(local_path),
        });
    }

    Ok(LoadedSettings {
        settings: FormatSettings::default(),
        source: None,
    })
}

fn read_settings(path: &Path) -> Result<FormatSettings> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read settings file {}", path.display()))?;
    serde_yaml::from_str::<FormatSettings>(&content)
        .with_context(|| format!("failed to parse YAML settings from {}", path.display()))
}

const fn default_indent_width() -> usize {
    4
}

const fn default_spaces_before_trailing_comments() -> usize {
    1
}

const fn default_spaces_after_trailing_comments() -> usize {
    0
}

const fn default_spaces_before_if_statement() -> usize {
    1
}

const fn default_spaces_blocking_assignment() -> usize {
    1
}

const fn default_spaces_no_blocking_assignment() -> usize {
    1
}

const fn default_true() -> bool {
    true
}

const fn default_false() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{resolve_settings, FormatSettings, IndentType};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn yaml_supports_snake_case_keys() {
        let settings: FormatSettings = serde_yaml::from_str(
            r#"
indent_width: 2
indent_type: tab
spaces_in_parentheses: true
align_line_comments: true
"#,
        )
        .unwrap();

        assert_eq!(settings.indent_width, 2);
        assert_eq!(settings.indent_type, IndentType::Tab);
        assert!(settings.spaces_in_parentheses);
        assert!(settings.align_line_comments);
    }

    #[test]
    fn yaml_supports_legacy_property_names() {
        let settings: FormatSettings = serde_yaml::from_str(
            r#"
IndentWidth: 8
IndentType: space
AlignBlockingAssignments: false
"#,
        )
        .unwrap();

        assert_eq!(settings.indent_width, 8);
        assert_eq!(settings.indent_type, IndentType::Space);
        assert!(!settings.align_blocking_assignments);
    }

    #[test]
    fn resolve_settings_uses_current_directory_yaml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(".verilog-format.yaml");
        fs::write(&config_path, "indent_width: 2\nalign_line_comments: true\n").unwrap();

        let loaded = resolve_settings(None, dir.path()).unwrap();
        assert_eq!(loaded.source.as_deref(), Some(config_path.as_path()));
        assert_eq!(loaded.settings.indent_width, 2);
        assert!(loaded.settings.align_line_comments);
    }
}
