use crate::get_config;
use icu_collator::{Collator, CollatorBorrowed};
use icu_locale::Locale;
use std::cmp::Ordering;
use std::sync::OnceLock;

static COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();

/// Parse a POSIX locale string (e.g., "en_US.UTF-8") to ICU Locale format
fn parse_posix_locale(locale_str: &str) -> Option<Locale> {
    // Strip encoding suffix (e.g., ".UTF-8")
    let locale_str = locale_str.split('.').next()?;

    if locale_str.is_empty() {
        return None;
    }

    // "C" and "POSIX" are special locales that use ASCII byte-order comparison.
    // Fall back to ICU's default (DUCET) for sensible Unicode-aware sorting.
    if locale_str == "C" || locale_str == "POSIX" {
        return None;
    }

    // Convert POSIX format (en_US) to BCP-47/ICU format (en-US)
    let locale_str = locale_str.replace('_', "-");

    locale_str.parse().ok()
}

/// Get system locale from environment variables (LC_COLLATE, LC_ALL, or LANG)
fn get_system_locale() -> Option<Locale> {
    let locale_str = std::env::var("LC_COLLATE")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LANG"))
        .ok()?;

    parse_posix_locale(&locale_str)
}

fn get_collator() -> &'static CollatorBorrowed<'static> {
    COLLATOR.get_or_init(|| {
        let locale = get_config().locale.clone().or_else(get_system_locale);
        let prefs = locale.map(|l| l.into()).unwrap_or_default();
        Collator::try_new(prefs, Default::default()).expect("Failed to create collator")
    })
}

pub fn string_collate(a: &str, b: &str) -> Ordering {
    get_collator().compare(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_posix_locale() {
        // Standard POSIX format with encoding
        assert_eq!(
            parse_posix_locale("en_US.UTF-8"),
            Some("en-US".parse().unwrap())
        );
        assert_eq!(
            parse_posix_locale("de_DE.UTF-8"),
            Some("de-DE".parse().unwrap())
        );
        assert_eq!(
            parse_posix_locale("sv_SE.UTF-8"),
            Some("sv-SE".parse().unwrap())
        );
        assert_eq!(
            parse_posix_locale("nb_NO.UTF-8"),
            Some("nb-NO".parse().unwrap())
        );
    }

    #[test]
    fn test_parse_posix_locale_without_encoding() {
        assert_eq!(parse_posix_locale("en_US"), Some("en-US".parse().unwrap()));
        assert_eq!(parse_posix_locale("de"), Some("de".parse().unwrap()));
    }

    #[test]
    fn test_parse_posix_locale_special() {
        // C and POSIX should return None
        assert_eq!(parse_posix_locale("C"), None);
        assert_eq!(parse_posix_locale("POSIX"), None);
        assert_eq!(parse_posix_locale("C.UTF-8"), None);
    }

    #[test]
    fn test_parse_posix_locale_empty() {
        assert_eq!(parse_posix_locale(""), None);
    }
}
