use std::collections::BTreeMap;
use std::path::Path;

use crate::engine::{ErrorCode, Result, system::fs};

/// Manages environment variable substitution for Cadman configurations.
///
/// Supports `${VAR}`, `${VAR:default}`, and `${VAR?required message}` syntax.
#[derive(Debug, Clone, Default)]
pub struct EnvManager {
    vars: BTreeMap<String, String>,
}

impl EnvManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable directly.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }

    /// Get a variable value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str)
    }

    /// Load all current process environment variables.
    /// Existing variables are overwritten.
    pub fn load_process_env(&mut self) {
        for (key, value) in std::env::vars() {
            self.vars.insert(key, value);
        }
    }

    /// Load variables from a `.env` file.
    /// Blank lines and lines starting with `#` are ignored.
    /// Single and double quotes are stripped from values.
    pub fn load_env_file(&mut self, path: &Path) -> Result<()> {
        let raw = fs::read_text(path).map_err(|_| {
            ErrorCode::ConfigNotFound
                .error()
                .with_context("path", path.display().to_string())
        })?;

        for (i, line) in raw.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match line.split_once('=') {
                Some((key, value)) => {
                    self.vars
                        .insert(key.trim().to_string(), strip_quotes(value.trim()));
                }
                None => {
                    return Err(ErrorCode::ConfigInvalid
                        .error()
                        .with_context("path", path.display().to_string())
                        .with_context("line", (i + 1).to_string())
                        .with_context("reason", "expected KEY=value format"));
                }
            }
        }
        Ok(())
    }

    /// Parse and apply `KEY=value` override strings.
    pub fn add_overrides<I, S>(&mut self, vars: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for var in vars {
            let s = var.as_ref();
            match s.split_once('=') {
                Some((key, value)) => {
                    self.vars.insert(key.trim().to_string(), value.to_string());
                }
                None => {
                    return Err(ErrorCode::UserInvalidArgument
                        .error()
                        .with_context("override", s)
                        .with_context("reason", "expected KEY=value format"));
                }
            }
        }
        Ok(())
    }

    /// Substitute `${VAR}`, `${VAR:default}`, and `${VAR?message}` in `input`.
    pub fn substitute(&self, input: &str) -> Result<String> {
        let mut result = String::with_capacity(input.len());
        let mut remaining = input;

        while let Some(start) = remaining.find("${") {
            result.push_str(&remaining[..start]);
            remaining = &remaining[start + 2..];

            let end = remaining.find('}').ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("reason", "unclosed ${ in substitution")
                    .with_context("input", input)
            })?;

            let expr = &remaining[..end];
            remaining = &remaining[end + 1..];

            result.push_str(&self.resolve_expr(expr, input)?);
        }

        result.push_str(remaining);
        Ok(result)
    }

    /// Substitute `${...}` in each string in the collection.
    pub fn substitute_in_collection(&self, values: &[String]) -> Result<Vec<String>> {
        values.iter().map(|v| self.substitute(v)).collect()
    }

    /// Returns true if `input` contains at least one `${...}` placeholder.
    pub fn has_unresolved(input: &str) -> bool {
        let mut remaining = input;
        while let Some(start) = remaining.find("${") {
            remaining = &remaining[start + 2..];
            if remaining.contains('}') {
                return true;
            }
        }
        false
    }

    fn resolve_expr(&self, expr: &str, original_input: &str) -> Result<String> {
        // ${VAR?required message}
        if let Some((var, msg)) = expr.split_once('?') {
            let var = var.trim();
            return self.vars.get(var).cloned().ok_or_else(|| {
                let message = if msg.is_empty() {
                    format!("required variable ${var} is not set")
                } else {
                    msg.to_string()
                };
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("variable", var)
                    .with_context("reason", message)
                    .with_context("input", original_input)
            });
        }

        // ${VAR:default}
        if let Some((var, default)) = expr.split_once(':') {
            let var = var.trim();
            return Ok(self.vars.get(var).cloned().unwrap_or_else(|| default.to_string()));
        }

        // ${VAR}
        let var = expr.trim();
        Ok(self.vars.get(var).cloned().unwrap_or_default())
    }
}

fn strip_quotes(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr(vars: &[(&str, &str)]) -> EnvManager {
        let mut m = EnvManager::new();
        for (k, v) in vars {
            m.set(*k, *v);
        }
        m
    }

    #[test]
    fn test_simple_substitution() {
        assert_eq!(mgr(&[("FOO", "bar")]).substitute("${FOO}").unwrap(), "bar");
    }

    #[test]
    fn test_default_value_used_when_unset() {
        assert_eq!(
            EnvManager::new().substitute("${FOO:fallback}").unwrap(),
            "fallback"
        );
    }

    #[test]
    fn test_default_value_not_used_when_set() {
        assert_eq!(
            mgr(&[("FOO", "actual")]).substitute("${FOO:fallback}").unwrap(),
            "actual"
        );
    }

    #[test]
    fn test_required_var_present() {
        assert_eq!(
            mgr(&[("FOO", "val")]).substitute("${FOO?must be set}").unwrap(),
            "val"
        );
    }

    #[test]
    fn test_required_var_missing() {
        assert!(EnvManager::new().substitute("${FOO?must be set}").is_err());
    }

    #[test]
    fn test_missing_required_no_message() {
        assert!(EnvManager::new().substitute("${FOO?}").is_err());
    }

    #[test]
    fn test_multiple_variables() {
        let m = mgr(&[("HOST", "localhost"), ("PORT", "8080")]);
        assert_eq!(
            m.substitute("http://${HOST}:${PORT}").unwrap(),
            "http://localhost:8080"
        );
    }

    #[test]
    fn test_env_file_loading() {
        use std::fs;
        let path = std::env::temp_dir()
            .join(format!("cadman_env_test_{}.env", std::process::id()));
        fs::write(&path, "KEY1=value1\n# comment\n\nKEY2=\"quoted\"\n").unwrap();

        let mut m = EnvManager::new();
        m.load_env_file(&path).unwrap();
        assert_eq!(m.get("KEY1"), Some("value1"));
        assert_eq!(m.get("KEY2"), Some("quoted"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_quoted_env_values() {
        use std::fs;
        let path = std::env::temp_dir()
            .join(format!("cadman_env_quotes_{}.env", std::process::id()));
        fs::write(&path, "A='single'\nB=\"double\"\n").unwrap();

        let mut m = EnvManager::new();
        m.load_env_file(&path).unwrap();
        assert_eq!(m.get("A"), Some("single"));
        assert_eq!(m.get("B"), Some("double"));

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_override_precedence() {
        let mut m = mgr(&[("FOO", "original")]);
        m.add_overrides(["FOO=override"]).unwrap();
        assert_eq!(m.get("FOO"), Some("override"));
    }

    #[test]
    fn test_invalid_override_format() {
        assert!(EnvManager::new().add_overrides(["NOTVALID"]).is_err());
    }

    #[test]
    fn test_has_unresolved() {
        assert!(EnvManager::has_unresolved("hello ${WORLD}"));
        assert!(!EnvManager::has_unresolved("hello world"));
        assert!(!EnvManager::has_unresolved("hello $WORLD"));
    }

    #[test]
    fn test_substitute_in_collection() {
        let m = mgr(&[("X", "1"), ("Y", "2")]);
        let result = m
            .substitute_in_collection(&["${X}".to_string(), "${Y}".to_string()])
            .unwrap();
        assert_eq!(result, vec!["1".to_string(), "2".to_string()]);
    }
}
