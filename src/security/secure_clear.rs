use zeroize::Zeroize;

/// A wrapper around String that zeroizes its contents on drop.
/// Use this for any data that might transiently hold plaintext,
/// decrypted content, or other sensitive material.
#[derive(Clone)]
pub struct SecureString {
    inner: String,
}

impl SecureString {
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn into_inner(mut self) -> String {
        let out = std::mem::take(&mut self.inner);
        out
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

impl std::fmt::Display for SecureString {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never allow accidental printing of secure content
        write!(_f, "[REDACTED]")
    }
}

/// Zeroize a Vec<u8> buffer in-place
#[allow(dead_code)]
pub fn clear_bytes(buf: &mut Vec<u8>) {
    buf.zeroize();
}

/// Zeroize a String in-place
pub fn clear_string(s: &mut String) {
    s.zeroize();
}
