//! The App Key Vault secrets of a run (`--app-secrets FILE`): per app, the secret names and values that System
//! Application's "App Key Vault Secret Provider" answers for that app. Without them no app has a key vault, as in a BC
//! test session with none configured. No real key vault is ever reached: the caller reads the values from a file.
//!
//! The values are secrets. Nothing here prints them: `Debug` shows only how many there are.

use std::collections::BTreeMap;

/// Secret values per app. An app is found by its id (case and braces do not matter) or by its name.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct AppSecrets {
    apps: BTreeMap<String, BTreeMap<String, String>>,
}

impl AppSecrets {
    /// Maps `name` to `value` for the app `app` (an app id or an app name). A second value for a name replaces the first.
    pub fn add(&mut self, app: &str, name: &str, value: &str) {
        self.apps
            .entry(app_key(app))
            .or_default()
            .insert(name.to_string(), value.to_string());
    }

    /// The secrets of the app with id `id` or name `name`, if the run has any for it.
    pub fn of_app(&self, id: &str, name: &str) -> Option<&BTreeMap<String, String>> {
        self.apps.get(&app_key(id)).or_else(|| self.apps.get(&app_key(name)))
    }

    /// How many apps have secrets.
    pub fn app_count(&self) -> usize {
        self.apps.len()
    }

    /// How many secrets there are over all apps.
    pub fn secret_count(&self) -> usize {
        self.apps.values().map(BTreeMap::len).sum()
    }
}

impl std::fmt::Debug for AppSecrets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppSecrets({} secrets for {} apps)",
            self.secret_count(),
            self.app_count()
        )
    }
}

fn app_key(app: &str) -> String {
    app.trim().trim_start_matches('{').trim_end_matches('}').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_app_is_found_by_id_in_any_case_and_with_braces_or_by_name() {
        let mut s = AppSecrets::default();
        s.add("{6F0B1A6E-0000-4000-8000-000000000001}", "k", "v");
        s.add("Continia Core", "c", "w");
        let by_id = s.of_app("6f0b1a6e-0000-4000-8000-000000000001", "Other").unwrap();
        assert_eq!(by_id.get("k").map(String::as_str), Some("v"));
        let by_name = s
            .of_app("00000000-0000-0000-0000-000000000000", "continia core")
            .unwrap();
        assert_eq!(by_name.get("c").map(String::as_str), Some("w"));
        assert!(s.of_app("x", "y").is_none());
    }

    #[test]
    fn debug_never_shows_a_value() {
        let mut s = AppSecrets::default();
        s.add("a", "name", "very-secret-value");
        let shown = format!("{s:?}");
        assert_eq!(shown, "AppSecrets(1 secrets for 1 apps)");
    }
}
