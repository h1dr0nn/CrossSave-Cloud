/// Validate email format
pub fn validate_email(email: &str) -> bool {
    if email.is_empty() || email.len() > 255 {
        return false;
    }

    // Basic email validation
    email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
}

/// Validate device ID (UUID format)
pub fn validate_device_id(device_id: &Option<String>) -> bool {
    match device_id {
        Some(id) => !id.trim().is_empty() && id.len() <= 128,
        None => true, // Optional field
    }
}

/// Validate game ID
pub fn validate_game_id(game_id: &str) -> bool {
    !game_id.is_empty() && game_id.len() <= 256
}

/// Validate version ID
pub fn validate_version_id(version_id: &str) -> bool {
    !version_id.is_empty() && version_id.len() <= 256
}

/// Validate SHA256 hash
pub fn validate_sha256(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate file size
pub fn validate_size_bytes(size: u64) -> bool {
    size > 0 && size <= 1024 * 1024 * 1024 // Max 1GB
}

/// Validate file list
pub fn validate_file_list(files: &[String]) -> bool {
    if files.is_empty() || files.len() > 10000 {
        return false;
    }

    files.iter().all(|f| !f.is_empty() && f.len() <= 512)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.user@domain.co.uk"));
        assert!(!validate_email(""));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("user@"));
        assert!(!validate_email("not-an-email"));
    }

    #[test]
    fn test_validate_sha256() {
        assert!(validate_sha256(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        ));
        assert!(!validate_sha256("invalid"));
        assert!(!validate_sha256(""));
    }

    #[test]
    fn test_validate_file_list() {
        assert!(validate_file_list(&vec![
            "file1.txt".to_string(),
            "file2.dat".to_string()
        ]));
        assert!(!validate_file_list(&vec![]));
    }
}
