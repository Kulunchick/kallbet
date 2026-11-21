use kallbet::config::Config;

fn config_from(vars: &[(&str, &str)]) -> Config {
    Config::from_lookup(|key| {
        vars.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.to_string())
    })
    .unwrap()
}

#[test]
fn dev_login_is_off_by_default() {
    assert!(!config_from(&[]).dev_login);
}

#[test]
fn dev_login_is_on_only_for_an_explicit_true() {
    assert!(config_from(&[("KALLBET_DEV_LOGIN", "true")]).dev_login);
    assert!(!config_from(&[("KALLBET_DEV_LOGIN", "false")]).dev_login);
    assert!(!config_from(&[("KALLBET_DEV_LOGIN", "")]).dev_login);
}

#[test]
fn an_unrecognised_dev_login_value_is_an_error_not_a_silent_enable() {
    let result = Config::from_lookup(|key| (key == "KALLBET_DEV_LOGIN").then(|| "yes please".into()));
    assert!(result.is_err());
}

#[test]
fn secrets_come_from_the_environment_and_are_not_printed() {
    let config = config_from(&[
        ("BOT_TOKEN", "123:super-secret"),
        ("PANDASCORE_TOKEN", "panda-secret"),
    ]);

    assert_eq!(config.bot_token.as_deref(), Some("123:super-secret"));
    assert_eq!(config.pandascore_token.as_deref(), Some("panda-secret"));
    let printed = format!("{config:?}");
    assert!(!printed.contains("super-secret"), "{printed}");
    assert!(!printed.contains("panda-secret"), "{printed}");
}

/// A11: a remote browser's DevTools URL can carry credentials (user info or
/// a token in the query); only where it points is printed.
#[test]
fn the_chrome_url_is_printed_without_credentials() {
    let config = config_from(&[(
        "KALLBET_CHROME_URL",
        "https://user:hunter2@chrome.example:3000/devtools?token=tok-secret",
    )]);

    let printed = format!("{config:?}");
    assert!(!printed.contains("hunter2"), "{printed}");
    assert!(!printed.contains("user:"), "{printed}");
    assert!(!printed.contains("tok-secret"), "{printed}");
    assert!(printed.contains("chrome.example:3000"), "{printed}");
}

#[test]
fn missing_secrets_are_simply_absent() {
    let config = config_from(&[("BOT_TOKEN", "")]);
    assert_eq!(config.bot_token, None);
    assert_eq!(config.pandascore_token, None);
}

#[test]
fn the_fallback_chain_settings_have_defaults_and_can_be_set() {
    let defaults = config_from(&[]);
    assert_eq!(
        (defaults.breaker_failures, defaults.breaker_open_minutes, defaults.match_window_hours),
        (3, 15, 6)
    );
    let set = config_from(&[
        ("KALLBET_BREAKER_FAILURES", "5"),
        ("KALLBET_BREAKER_OPEN_MINUTES", "30"),
        ("KALLBET_MATCH_WINDOW_HOURS", "2"),
    ]);
    assert_eq!(
        (set.breaker_failures, set.breaker_open_minutes, set.match_window_hours),
        (5, 30, 2)
    );
    let zero = Config::from_lookup(|key| (key == "KALLBET_BREAKER_FAILURES").then(|| "0".into()));
    assert!(zero.is_err());
}
