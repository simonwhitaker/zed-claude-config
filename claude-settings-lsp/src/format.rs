use serde_json::Value;

/// Format a Claude settings JSON file by sorting and deduplicating
/// the `permissions.allow`, `permissions.ask`, and `permissions.deny` arrays.
pub fn format_claude_settings(input: &str) -> Result<String, String> {
    let mut value: Value =
        serde_json::from_str(input).map_err(|e| format!("invalid JSON: {e}"))?;

    if let Some(permissions) = value.get_mut("permissions").and_then(|v| v.as_object_mut()) {
        for key in &["allow", "ask", "deny"] {
            if let Some(arr) = permissions.get_mut(*key).and_then(|v| v.as_array_mut()) {
                sort_and_dedup_string_array(arr);
            }
        }
    }

    let mut output =
        serde_json::to_string_pretty(&value).map_err(|e| format!("serialization error: {e}"))?;
    output.push('\n');
    Ok(output)
}

fn sort_and_dedup_string_array(arr: &mut Vec<Value>) {
    arr.sort_by(|a, b| {
        let a_str = a.as_str().unwrap_or("");
        let b_str = b.as_str().unwrap_or("");
        a_str.cmp(b_str)
    });
    arr.dedup_by(|a, b| a.as_str() == b.as_str());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorts_and_deduplicates() {
        let input = r#"{
  "permissions": {
    "allow": [
      "Bash(git status)",
      "Bash(git diff)",
      "Bash(git status)",
      "Read(*)",
      "Bash(cargo build)"
    ],
    "deny": [
      "Bash(rm -rf /)",
      "Bash(rm -rf /)"
    ]
  }
}"#;
        let result = format_claude_settings(input).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        let allow = parsed["permissions"]["allow"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            allow,
            vec![
                "Bash(cargo build)",
                "Bash(git diff)",
                "Bash(git status)",
                "Read(*)"
            ]
        );

        let deny = parsed["permissions"]["deny"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(deny, vec!["Bash(rm -rf /)"]);
    }

    #[test]
    fn test_preserves_other_keys() {
        let input = r#"{
  "model": "opus",
  "permissions": {
    "allow": ["Write(*)", "Read(*)"]
  },
  "customKey": 42
}"#;
        let result = format_claude_settings(input).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(parsed["model"], "opus");
        assert_eq!(parsed["customKey"], 42);
    }

    #[test]
    fn test_no_permissions_key() {
        let input = r#"{"foo": "bar"}"#;
        let result = format_claude_settings(input).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["foo"], "bar");
    }

    #[test]
    fn test_trailing_newline() {
        let input = r#"{}"#;
        let result = format_claude_settings(input).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_invalid_json() {
        let result = format_claude_settings("not json");
        assert!(result.is_err());
    }
}
