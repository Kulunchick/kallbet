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

#[test]
fn missing_secrets_are_simply_absent() {
    let config = config_from(&[("BOT_TOKEN", "")]);
    assert_eq!(config.bot_token, None);
    assert_eq!(config.pandascore_token, None);
}
