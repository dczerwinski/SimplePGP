use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgpKey {
    pub key_id: String,
    pub fingerprint: String,
    pub uid: String,
    pub has_secret: bool,
    pub algorithm: String,
    pub key_length: u32,
    pub creation_date: String,
    pub expiration_date: Option<String>,
    pub trust: TrustLevel,
}

impl PgpKey {
    pub fn display_name(&self) -> String {
        if self.uid.is_empty() {
            format!("[{}]", self.short_id())
        } else {
            self.uid.clone()
        }
    }

    pub fn short_id(&self) -> &str {
        if self.key_id.len() >= 8 {
            &self.key_id[self.key_id.len() - 8..]
        } else {
            &self.key_id
        }
    }

    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        let secret_marker = if self.has_secret { " [secret]" } else { "" };
        format!(
            "{} ({}){} — {}",
            self.display_name(),
            self.short_id(),
            secret_marker,
            self.trust
        )
    }
}

impl fmt::Display for PgpKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.display_name(), self.short_id())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Unknown,
    Expired,
    Undefined,
    Never,
    Marginal,
    Full,
    Ultimate,
}

impl TrustLevel {
    pub fn from_colon_field(field: &str) -> Self {
        match field {
            "e" => TrustLevel::Expired,
            "n" => TrustLevel::Never,
            "m" => TrustLevel::Marginal,
            "f" => TrustLevel::Full,
            "u" => TrustLevel::Ultimate,
            "q" | "-" => TrustLevel::Undefined,
            _ => TrustLevel::Unknown,
        }
    }
}

impl fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TrustLevel::Unknown => "unknown",
            TrustLevel::Expired => "expired",
            TrustLevel::Undefined => "undefined",
            TrustLevel::Never => "never trust",
            TrustLevel::Marginal => "marginal",
            TrustLevel::Full => "full",
            TrustLevel::Ultimate => "ultimate",
        };
        write!(f, "{}", label)
    }
}
