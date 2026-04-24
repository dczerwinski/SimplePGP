/// Validates that a string does not contain shell metacharacters
/// that could be used for injection attacks.
#[allow(dead_code)]
pub fn sanitize_input(input: &str) -> Result<(), InputValidationError> {
    let forbidden = ['|', ';', '&', '$', '`', '\\', '\n', '\r', '\0'];
    for ch in forbidden {
        if input.contains(ch) {
            return Err(InputValidationError::ForbiddenCharacter(ch));
        }
    }
    Ok(())
}

/// Validates a key ID / fingerprint contains only hex characters.
pub fn validate_key_id(key_id: &str) -> Result<(), InputValidationError> {
    if key_id.is_empty() {
        return Err(InputValidationError::EmptyKeyId);
    }
    if !key_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(InputValidationError::InvalidKeyIdCharacters);
    }
    Ok(())
}

/// Validates a free-form text field (name, comment, email) used when
/// generating keys. Rejects control characters that could break the
/// unattended GPG batch script (CR/LF/NUL).
pub fn validate_keygen_field(field: &str) -> Result<(), InputValidationError> {
    for ch in field.chars() {
        if ch == '\n' || ch == '\r' || ch == '\0' {
            return Err(InputValidationError::ForbiddenCharacter(ch));
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum InputValidationError {
    #[error("Input contains forbidden character: '{0:?}'")]
    ForbiddenCharacter(char),
    #[error("Key ID must not be empty")]
    EmptyKeyId,
    #[error("Key ID contains non-hex characters")]
    InvalidKeyIdCharacters,
    #[error("Required field is empty")]
    EmptyField,
}
