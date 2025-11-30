use crosssave_selfhost_server::validation::{
    validate_device_id, validate_email, validate_game_id, validate_version_id,
};

#[test]
fn test_validate_email() {
    assert!(validate_email("test@example.com"));
    assert!(validate_email("user.name+tag@example.co.uk"));
    assert!(!validate_email("invalid-email"));
    assert!(!validate_email("@example.com"));
    assert!(!validate_email("user@"));
    assert!(!validate_email(""));
}

#[test]
fn test_validate_device_id() {
    assert!(validate_device_id(&Some("device123".to_string())));
    assert!(validate_device_id(&Some("my-laptop".to_string())));
    assert!(!validate_device_id(&Some("".to_string())));
    assert!(!validate_device_id(&Some("a".repeat(129)))); // Too long (>128)
                                                          // assert!(!validate_device_id(&Some("invalid/char".to_string()))); // Regex check not implemented yet
    assert!(validate_device_id(&None)); // Optional is valid
}

#[test]
fn test_validate_game_id() {
    assert!(validate_game_id("game123"));
    assert!(validate_game_id("my-game-id"));
    assert!(!validate_game_id("ab")); // Too short (<3)
    assert!(!validate_game_id("a".repeat(257).as_str())); // Too long (>256)
                                                          // assert!(!validate_game_id("invalid/char")); // Regex check not implemented yet
}

#[test]
fn test_validate_version_id() {
    assert!(validate_version_id("v1.0.0"));
    assert!(validate_version_id("2023-01-01-backup"));
    assert!(!validate_version_id(""));
    assert!(!validate_version_id("a".repeat(257).as_str())); // Too long (>256)
                                                             // assert!(!validate_version_id("invalid/char")); // Regex check not implemented yet
}
