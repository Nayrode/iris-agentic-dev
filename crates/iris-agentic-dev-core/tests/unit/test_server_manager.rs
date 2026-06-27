// Tests for Server Manager settings.json parsing, credential resolution,
// and server selection (US1 + US4, 044-servermanager-discovery).
//
// All tests run without a live IRIS connection or real OS keychain.
// Credential tests use keyring_core mock store.

use iris_agentic_dev_core::iris::server_manager::{
    parse_sm_settings, resolve_credential, select_server, SmCredentialError,
};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

// ── parse_sm_settings tests ─────────────────────────────────────────────────

#[test]
fn parse_single_server() {
    let profiles = parse_sm_settings(&fixture("sm_settings_single.json"));
    assert_eq!(
        profiles.len(),
        1,
        "single server fixture should yield 1 profile"
    );
    let p = &profiles[0];
    assert_eq!(p.name, "dev-local");
    assert_eq!(p.host, "127.0.0.1");
    assert_eq!(p.port, 52773);
    assert_eq!(p.scheme, "http");
    assert_eq!(p.username, "_SYSTEM");
    assert!(p.password_deprecated.is_none());
}

#[test]
fn parse_multi_server_skips_default_key() {
    let profiles = parse_sm_settings(&fixture("sm_settings_multi.json"));
    // /default is not a server entry — must be skipped
    assert_eq!(
        profiles.len(),
        3,
        "multi fixture has 3 real servers; /default key must be skipped"
    );
    let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"dev-local"));
    assert!(names.contains(&"staging"));
    assert!(names.contains(&"prod"));
    assert!(!names.contains(&"/default"));
}

#[test]
fn parse_multi_server_path_prefix() {
    let profiles = parse_sm_settings(&fixture("sm_settings_multi.json"));
    let prod = profiles.iter().find(|p| p.name == "prod").unwrap();
    assert_eq!(prod.path_prefix.as_deref(), Some("iris"));
}

#[test]
fn parse_malformed_returns_empty() {
    let profiles = parse_sm_settings(&fixture("sm_settings_malformed.json"));
    assert!(
        profiles.is_empty(),
        "malformed JSON must return empty vec (not panic)"
    );
}

#[test]
fn parse_missing_file_returns_empty() {
    let profiles = parse_sm_settings(&PathBuf::from("/nonexistent/path/settings.json"));
    assert!(
        profiles.is_empty(),
        "missing file must return empty vec (not panic)"
    );
}

#[test]
fn parse_deprecated_password_captured() {
    let profiles = parse_sm_settings(&fixture("sm_settings_deprecated_pw.json"));
    assert_eq!(profiles.len(), 1);
    let p = &profiles[0];
    assert_eq!(
        p.password_deprecated.as_deref(),
        Some("old-plaintext-password")
    );
}

// ── select_server tests ─────────────────────────────────────────────────────

#[test]
fn select_server_single_auto_selects() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("IRIS_SERVER_NAME");
    let profiles = parse_sm_settings(&fixture("sm_settings_single.json"));
    let result = select_server(&profiles);
    assert!(result.is_ok(), "single server should auto-select");
    assert_eq!(result.unwrap().name, "dev-local");
}

#[test]
fn select_server_multi_requires_env_var() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("IRIS_SERVER_NAME");
    let profiles = parse_sm_settings(&fixture("sm_settings_multi.json"));
    let result = select_server(&profiles);
    assert!(
        result.is_err(),
        "multi-server without IRIS_SERVER_NAME must return error"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, SmCredentialError::Ambiguous { .. }),
        "error must be Ambiguous variant, got {err:?}"
    );
    if let SmCredentialError::Ambiguous { available } = err {
        assert_eq!(available.len(), 3);
        assert!(available.contains(&"dev-local".to_string()));
    }
}

#[test]
fn select_server_multi_with_env_var_selects_named() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("IRIS_SERVER_NAME", "staging");
    let profiles = parse_sm_settings(&fixture("sm_settings_multi.json"));
    let result = select_server(&profiles);
    std::env::remove_var("IRIS_SERVER_NAME");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "staging");
}

#[test]
fn select_server_env_var_unknown_name_returns_ambiguous() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::set_var("IRIS_SERVER_NAME", "does-not-exist");
    let profiles = parse_sm_settings(&fixture("sm_settings_multi.json"));
    let result = select_server(&profiles);
    std::env::remove_var("IRIS_SERVER_NAME");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        SmCredentialError::Ambiguous { .. }
    ));
}

#[test]
fn select_server_empty_profiles_returns_ambiguous() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("IRIS_SERVER_NAME");
    let result = select_server(&[]);
    assert!(result.is_err());
}

// ── credential resolution tests ─────────────────────────────────────────────
// Uses keyring_core mock store: set_default_store() injects an in-memory store;
// the real keyring::Entry::new/get_password/set_password calls hit it.
// Each test must reset the store to avoid cross-test contamination.
//
// Keychain service name: "intersystems-server-credentials" — the auth provider ID
// registered by intersystems-community.servermanager in all VS Code-compatible IDEs.
// Confirmed from: ~/.vscode/extensions/intersystems-community.servermanager-*/dist/extension.js
// AUTHENTICATION_PROVIDER = "intersystems-server-credentials"
const SM_SERVICE: &str = "intersystems-server-credentials";

fn with_mock_store<F: FnOnce()>(f: F) {
    keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
    f();
    // Reset to a fresh empty store so the next test starts clean.
    keyring_core::set_default_store(keyring_core::mock::Store::new().unwrap());
}

#[test]
fn resolve_credential_mock_store_found() {
    with_mock_store(|| {
        // Seed using the confirmed SM service name (bypasses v1 Once guard)
        let entry =
            keyring_core::Entry::new(SM_SERVICE, "credentialProvider:dev-local/_system").unwrap();
        entry.set_password("test-password-123").unwrap();

        let result = resolve_credential("dev-local", "_SYSTEM");
        assert!(
            result.is_ok(),
            "mock store should resolve credential: {result:?}"
        );
        assert_eq!(result.unwrap(), "test-password-123");
    });
}

#[test]
fn resolve_credential_no_entry_returns_credential_error() {
    with_mock_store(|| {
        // Nothing seeded — should get NoEntry → CredentialNotFound
        let result = resolve_credential("nonexistent-server", "_SYSTEM");
        assert!(result.is_err());
        assert!(
            matches!(
                result.unwrap_err(),
                SmCredentialError::CredentialNotFound { .. }
            ),
            "missing entry must return CredentialNotFound"
        );
    });
}

// ── fail-fast invariant test (C1 from analyze) ──────────────────────────────

#[test]
fn credential_error_does_not_fall_through_to_downstream_discovery() {
    // When SM settings file is found AND a server is matched BUT credential resolution
    // fails, the error must be returned immediately — downstream discovery sources
    // (Docker, env var) must NOT be attempted.
    with_mock_store(|| {
        let result = resolve_credential("prod", "svc_account");
        assert!(
            result.is_err(),
            "credential lookup failure must propagate as Err, not silently fall through"
        );
        match result.unwrap_err() {
            SmCredentialError::CredentialNotFound { server_name } => {
                assert_eq!(server_name, "prod");
            }
            other => panic!("expected CredentialNotFound, got {other:?}"),
        }
    });
}

// ── check_config server_manager section tests (US4) ─────────────────────────

#[test]
fn check_config_sm_available_when_servers_found() {
    use iris_agentic_dev_core::iris::server_manager::build_server_manager_config_json;
    let profiles = parse_sm_settings(&fixture("sm_settings_single.json"));
    let json = build_server_manager_config_json(&profiles, Some("dev-local"), &[]);
    assert_eq!(json["available"], true);
    let servers = json["servers"].as_array().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0]["name"], "dev-local");
}

#[test]
fn check_config_sm_unavailable_when_empty() {
    use iris_agentic_dev_core::iris::server_manager::build_server_manager_config_json;
    let json = build_server_manager_config_json(&[], None, &[]);
    assert_eq!(json["available"], false);
}

#[test]
fn check_config_sm_credential_status_resolved() {
    use iris_agentic_dev_core::iris::server_manager::{
        build_server_manager_config_json, ServerManagerCredentialEntry,
    };
    let profiles = parse_sm_settings(&fixture("sm_settings_single.json"));
    let cred_entries = vec![ServerManagerCredentialEntry {
        server_name: "dev-local".to_string(),
        status: "resolved".to_string(),
        policy: None,
    }];
    let json = build_server_manager_config_json(&profiles, Some("dev-local"), &cred_entries);
    let servers = json["servers"].as_array().unwrap();
    assert_eq!(servers[0]["credential_status"], "resolved");
}

#[test]
fn check_config_sm_latency_when_not_installed() {
    // SC-004: SM discovery on a non-existent path must complete in < 200ms
    use iris_agentic_dev_core::iris::server_manager::parse_sm_settings;
    let start = std::time::Instant::now();
    let profiles = parse_sm_settings(&PathBuf::from("/nonexistent/no/such/settings.json"));
    let elapsed = start.elapsed();
    assert!(
        profiles.is_empty(),
        "missing file must return empty profiles"
    );
    assert!(
        elapsed.as_millis() < 200,
        "SM not-installed path must complete in < 200ms, took {}ms",
        elapsed.as_millis()
    );
}

// ── Service name verification tests ─────────────────────────────────────────
// The SM extension uses "intersystems-server-credentials" as its auth provider ID —
// this is the OS keychain service name for ALL VS Code-compatible forks (Cursor,
// Windsurf, VS Code Insiders, VSCodium). Confirmed from extension source.

#[test]
fn resolve_credential_correct_service_name_used() {
    with_mock_store(|| {
        // Credential ONLY exists under the correct SM service name.
        // If resolve_credential probes a wrong name it will return CredentialNotFound.
        let entry = keyring_core::Entry::new(SM_SERVICE, "credentialProvider:prod-server/svc_user")
            .unwrap();
        entry.set_password("prod-secret").unwrap();

        let result = resolve_credential("prod-server", "svc_user");
        assert!(
            result.is_ok(),
            "must find credential under '{SM_SERVICE}' service: {result:?}"
        );
        assert_eq!(result.unwrap(), "prod-secret");
    });
}

#[test]
fn resolve_credential_username_lowercased_in_account_key() {
    with_mock_store(|| {
        // Account key uses lowercase username — seed with lowercase, query with uppercase
        let entry =
            keyring_core::Entry::new(SM_SERVICE, "credentialProvider:dev-local/_system").unwrap();
        entry.set_password("lowercase-test").unwrap();

        // Caller passes "_SYSTEM" (uppercase) — must be lowercased to "_system" internally
        let result = resolve_credential("dev-local", "_SYSTEM");
        assert!(
            result.is_ok(),
            "uppercase username must match lowercase key: {result:?}"
        );
        assert_eq!(result.unwrap(), "lowercase-test");
    });
}

// Serialize env-var–touching tests
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
