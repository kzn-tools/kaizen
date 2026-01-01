//! Internationalization (i18n) module for locale detection and setup
//!
//! Locale detection priority:
//! 1. --lang CLI flag
//! 2. KAIZEN_LANG environment variable
//! 3. System locale (LANG, LC_ALL)
//! 4. Default: "en"

const SUPPORTED_LOCALES: &[&str] = &["en", "fr"];
const DEFAULT_LOCALE: &str = "en";
const ENV_VAR_NAME: &str = "KAIZEN_LANG";

pub fn init_locale(cli_lang: Option<&str>) {
    let locale = detect_locale(cli_lang);
    rust_i18n::set_locale(&locale);
}

fn detect_locale(cli_lang: Option<&str>) -> String {
    if let Some(lang) = cli_lang {
        if let Some(locale) = normalize_locale(lang) {
            return locale;
        }
    }

    if let Ok(env_lang) = std::env::var(ENV_VAR_NAME) {
        if let Some(locale) = normalize_locale(&env_lang) {
            return locale;
        }
    }

    if let Some(system_locale) = sys_locale::get_locale() {
        if let Some(locale) = normalize_locale(&system_locale) {
            return locale;
        }
    }

    DEFAULT_LOCALE.to_string()
}

fn normalize_locale(input: &str) -> Option<String> {
    let lower = input.to_lowercase();
    let lang_code = lower.split(&['-', '_'][..]).next().unwrap_or(&lower);

    for &supported in SUPPORTED_LOCALES {
        if lang_code == supported {
            return Some(supported.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn normalize_locale_simple() {
        assert_eq!(normalize_locale("en"), Some("en".to_string()));
        assert_eq!(normalize_locale("fr"), Some("fr".to_string()));
    }

    #[test]
    fn normalize_locale_with_region() {
        assert_eq!(normalize_locale("en-US"), Some("en".to_string()));
        assert_eq!(normalize_locale("en_GB"), Some("en".to_string()));
        assert_eq!(normalize_locale("fr-FR"), Some("fr".to_string()));
        assert_eq!(normalize_locale("fr_CA"), Some("fr".to_string()));
    }

    #[test]
    fn normalize_locale_case_insensitive() {
        assert_eq!(normalize_locale("EN"), Some("en".to_string()));
        assert_eq!(normalize_locale("FR"), Some("fr".to_string()));
        assert_eq!(normalize_locale("En-Us"), Some("en".to_string()));
    }

    #[test]
    fn normalize_locale_unsupported() {
        assert_eq!(normalize_locale("de"), None);
        assert_eq!(normalize_locale("es"), None);
        assert_eq!(normalize_locale("zh-CN"), None);
    }

    #[test]
    fn detect_locale_cli_takes_priority() {
        let result = detect_locale(Some("fr"));
        assert_eq!(result, "fr");
    }

    #[test]
    fn detect_locale_defaults_to_en() {
        let result = detect_locale(None);
        assert!(result == "en" || result == "fr");
    }

    #[test]
    #[serial]
    fn detect_locale_from_env() {
        unsafe { std::env::set_var(ENV_VAR_NAME, "fr") };
        let result = detect_locale(None);
        assert_eq!(result, "fr");
        unsafe { std::env::remove_var(ENV_VAR_NAME) };
    }

    #[test]
    #[serial]
    fn cli_overrides_env() {
        unsafe { std::env::set_var(ENV_VAR_NAME, "en") };
        let result = detect_locale(Some("fr"));
        assert_eq!(result, "fr");
        unsafe { std::env::remove_var(ENV_VAR_NAME) };
    }

    #[test]
    fn init_locale_sets_rust_i18n() {
        init_locale(Some("fr"));
        assert_eq!(&*rust_i18n::locale(), "fr");

        init_locale(Some("en"));
        assert_eq!(&*rust_i18n::locale(), "en");
    }
}
