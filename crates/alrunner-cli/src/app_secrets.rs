//! `--app-secrets FILE`: reads the App Key Vault secrets of the apps from a JSON file
//! (`{"<app id or name>": {"<secret name>": "<value>"}}`), or from `$ALRUNNER_APP_SECRETS`. An error names the app and the secret, never a value.

use al_host::AppSecrets;
use std::path::Path;

/// The secrets in the file at `path`.
pub fn read(path: &Path) -> Result<AppSecrets, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("--app-secrets {}: cannot read the file: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("--app-secrets {}: {e}", path.display()))
}

/// The secrets `$ALRUNNER_APP_SECRETS` gives, and where they came from: the JSON itself when the value
/// starts with `{` (so a pipeline secret never touches the disk), else the file it names.
pub fn from_variable(value: &str) -> Result<(AppSecrets, String), String> {
    let from = "$ALRUNNER_APP_SECRETS".to_string();
    if value.trim_start().starts_with('{') {
        return parse(value)
            .map(|s| (s, from.clone()))
            .map_err(|e| format!("{from}: {e}"));
    }
    let path = Path::new(value);
    read(path).map(|s| (s, path.display().to_string()))
}

/// The secrets in `text`.
pub fn parse(text: &str) -> Result<AppSecrets, String> {
    let json: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not valid JSON: {e}"))?;
    let serde_json::Value::Object(apps) = json else {
        return Err("expected a JSON object of apps".into());
    };
    let mut secrets = AppSecrets::default();
    for (app, names) in apps {
        let serde_json::Value::Object(names) = names else {
            return Err(format!("app '{app}': expected an object of secret names"));
        };
        for (name, value) in names {
            let serde_json::Value::String(value) = value else {
                return Err(format!("app '{app}', secret '{name}': the value is not a text"));
            };
            secrets.add(&app, &name, &value);
        }
    }
    Ok(secrets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_secrets_of_each_app() {
        let s = parse(r#"{"{1B4D7E2C-0000-4000-8000-000000000001}": {"api-key": "v1"}, "Core": {"a": "x", "b": "y"}}"#)
            .unwrap();
        assert_eq!(s.app_count(), 2);
        assert_eq!(s.secret_count(), 3);
        let app = s.of_app("1b4d7e2c-0000-4000-8000-000000000001", "").unwrap();
        assert_eq!(app.get("api-key").map(String::as_str), Some("v1"));
    }

    #[test]
    fn an_error_never_shows_a_value() {
        let e = parse(r#"{"Core": {"pin": 123456789}}"#).unwrap_err();
        assert!(e.contains("'pin'") && !e.contains("123456789"), "{e}");
        let e = parse(r#"{"Core": "top-secret"}"#).unwrap_err();
        assert!(!e.contains("top-secret"), "{e}");
        assert!(parse("[]").is_err());
        assert!(parse("{").unwrap_err().starts_with("not valid JSON"));
    }

    #[test]
    fn the_variable_holds_a_file_or_the_secrets_themselves() {
        let (s, from) = from_variable(r#"  {"Core": {"a": "x"}}"#).unwrap();
        assert_eq!((s.secret_count(), from.as_str()), (1, "$ALRUNNER_APP_SECRETS"));
        let e = from_variable(r#"{"Core": {"pin": 7}}"#).unwrap_err();
        assert!(e.starts_with("$ALRUNNER_APP_SECRETS: ") && !e.contains('7'), "{e}");
        let e = from_variable("no-such-file.json").unwrap_err();
        assert!(e.contains("no-such-file.json") && e.contains("cannot read"), "{e}");
    }
}
