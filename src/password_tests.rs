//! Unit tests for password hashing and verification using the password-auth library

use crate::test_utils::*;
use password_auth::{generate_hash, verify_password};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hash_success() {
        let password = TEST_PASSWORD;
        let hash = generate_hash(password);

        // Hash should not be empty
        assert!(!hash.is_empty());

        // Hash should be different from original password
        assert_ne!(hash, password);

        // Hash should be longer than password (includes salt)
        assert!(hash.len() > password.len());
    }

    #[test]
    fn test_generate_hash_different_hashes() {
        let password = TEST_PASSWORD;
        let hash1 = generate_hash(password);
        let hash2 = generate_hash(password);

        // Same password should generate different hashes (due to salt)
        assert_ne!(hash1, hash2);

        // Both hashes should be valid
        assert!(verify_password(password, &hash1).is_ok());
        assert!(verify_password(password, &hash2).is_ok());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = TEST_PASSWORD;
        let hash = generate_hash(password);

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = TEST_PASSWORD;
        let wrong_password = "wrongpassword123";
        let hash = generate_hash(password);

        let result = verify_password(wrong_password, &hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_empty_password() {
        let password = "";
        let hash = generate_hash(password);

        // Empty password should still work
        let result = verify_password(password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_password_empty_hash() {
        let password = TEST_PASSWORD;
        let empty_hash = "";

        let result = verify_password(password, empty_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_password_malformed_hash() {
        let password = TEST_PASSWORD;
        let malformed_hash = "not.a.valid.hash";

        let result = verify_password(password, malformed_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_with_special_characters() {
        let password = "P@ssw0rd!@#$%^&*()";
        let hash = generate_hash(password);

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_with_unicode() {
        let password = "пароль123";
        let hash = generate_hash(password);

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_very_long() {
        let password = "a".repeat(1000);
        let hash = generate_hash(&password);

        let result = verify_password(&password, &hash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_consistency() {
        let password = TEST_PASSWORD;
        let hash = generate_hash(password);

        // Verify the same hash multiple times
        for _ in 0..10 {
            let result = verify_password(password, &hash);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = generate_hash(password1);
        let hash2 = generate_hash(password2);

        // Different passwords should generate different hashes
        assert_ne!(hash1, hash2);

        // Each password should only verify with its own hash
        assert!(verify_password(password1, &hash1).is_ok());
        assert!(verify_password(password2, &hash2).is_ok());
        assert!(verify_password(password1, &hash2).is_err());
        assert!(verify_password(password2, &hash1).is_err());
    }
}
