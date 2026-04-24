use std::collections::HashMap;
use std::process::Command;

use crate::models::{PgpKey, TrustLevel};
use crate::security::{validate_key_id, validate_keygen_field, SecureString};

/// Parameters for unattended key generation.
#[derive(Debug, Clone)]
pub struct KeyGenParams {
    pub name: String,
    pub email: String,
    pub comment: String,
    /// Key algorithm, e.g. "RSA" or "EDDSA".
    pub algorithm: KeyAlgorithm,
    /// Key length in bits (ignored for EdDSA-based profiles).
    pub key_length: u32,
    /// Expiration in GPG format ("0" = never, "1y", "2y", "6m", ...).
    pub expire: String,
    /// Optional passphrase. Empty means `%no-protection`.
    pub passphrase: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAlgorithm {
    Rsa,
    Ed25519,
}

#[derive(Debug, thiserror::Error)]
pub enum GpgError {
    #[error("Failed to execute gpg: {0}")]
    Execution(#[from] std::io::Error),
    #[error("GPG returned error (exit {code}): {stderr}")]
    CommandFailed { code: i32, stderr: String },
    #[allow(dead_code)]
    #[error("Failed to parse GPG output: {0}")]
    ParseError(String),
    #[error("Input validation failed: {0}")]
    Validation(#[from] crate::security::InputValidationError),
    #[error("No recipient key selected")]
    NoRecipient,
}

pub struct GpgService;

impl GpgService {
    pub fn new() -> Self {
        Self
    }

    /// Lists all public keys from the system keyring.
    pub fn list_public_keys(&self) -> Result<Vec<PgpKey>, GpgError> {
        let output = Command::new("gpg")
            .args(["--list-keys", "--with-colons", "--batch", "--no-tty"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            // gpg returns exit 2 when keyring is empty — treat as empty list
            if output.status.code() == Some(2) && stderr.contains("No public key") {
                return Ok(Vec::new());
            }
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_key_listing(&stdout, false)
    }

    /// Lists all secret keys from the system keyring, returning their key IDs.
    pub fn list_secret_key_ids(&self) -> Result<Vec<String>, GpgError> {
        let output = Command::new("gpg")
            .args([
                "--list-secret-keys",
                "--with-colons",
                "--batch",
                "--no-tty",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if output.status.code() == Some(2) {
                return Ok(Vec::new());
            }
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ids = Vec::new();
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() >= 5 && (fields[0] == "sec" || fields[0] == "ssb") {
                ids.push(fields[4].to_string());
            }
        }
        Ok(ids)
    }

    /// Combined call: loads public keys and marks which ones have secrets.
    pub fn list_all_keys(&self) -> Result<Vec<PgpKey>, GpgError> {
        let mut keys = self.list_public_keys()?;
        let secret_ids = self.list_secret_key_ids()?;

        for key in &mut keys {
            if secret_ids.iter().any(|sid| key.key_id.ends_with(sid) || sid.ends_with(&key.key_id))
            {
                key.has_secret = true;
            }
        }

        Ok(keys)
    }

    /// Generates a new PGP key pair using an unattended batch script.
    /// Returns the GPG stderr output (usually contains the new key's
    /// fingerprint / identifier info).
    pub fn generate_key(&self, params: &KeyGenParams) -> Result<String, GpgError> {
        validate_keygen_field(&params.name)?;
        validate_keygen_field(&params.email)?;
        validate_keygen_field(&params.comment)?;
        validate_keygen_field(&params.expire)?;
        validate_keygen_field(&params.passphrase)?;

        if params.name.trim().is_empty() {
            return Err(GpgError::Validation(
                crate::security::InputValidationError::EmptyField,
            ));
        }

        let script = Self::build_keygen_script(params);

        let mut cmd = Command::new("gpg");
        cmd.args([
            "--batch",
            "--no-tty",
            "--pinentry-mode",
            "loopback",
            "--gen-key",
        ]);
        if !params.passphrase.is_empty() {
            cmd.args(["--passphrase", &params.passphrase]);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(script.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Ok(stderr)
    }

    /// Deletes a key from the keyring by fingerprint.
    /// If the key has a secret component, the secret is deleted first.
    pub fn delete_key(&self, fingerprint: &str, has_secret: bool) -> Result<(), GpgError> {
        validate_key_id(fingerprint)?;

        if has_secret {
            let output = Command::new("gpg")
                .args([
                    "--batch",
                    "--yes",
                    "--no-tty",
                    "--pinentry-mode",
                    "loopback",
                    "--delete-secret-keys",
                    fingerprint,
                ])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                return Err(GpgError::CommandFailed {
                    code: output.status.code().unwrap_or(-1),
                    stderr,
                });
            }
        }

        let output = Command::new("gpg")
            .args([
                "--batch",
                "--yes",
                "--no-tty",
                "--delete-keys",
                fingerprint,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Ok(())
    }

    fn build_keygen_script(params: &KeyGenParams) -> String {
        let mut script = String::new();
        script.push_str("%echo Generating a new PGP key\n");

        match params.algorithm {
            KeyAlgorithm::Rsa => {
                script.push_str("Key-Type: RSA\n");
                script.push_str(&format!("Key-Length: {}\n", params.key_length));
                script.push_str("Key-Usage: sign,cert\n");
                script.push_str("Subkey-Type: RSA\n");
                script.push_str(&format!("Subkey-Length: {}\n", params.key_length));
                script.push_str("Subkey-Usage: encrypt\n");
            }
            KeyAlgorithm::Ed25519 => {
                script.push_str("Key-Type: EDDSA\n");
                script.push_str("Key-Curve: ed25519\n");
                script.push_str("Key-Usage: sign,cert\n");
                script.push_str("Subkey-Type: ECDH\n");
                script.push_str("Subkey-Curve: cv25519\n");
                script.push_str("Subkey-Usage: encrypt\n");
            }
        }

        script.push_str(&format!("Name-Real: {}\n", params.name.trim()));
        if !params.comment.trim().is_empty() {
            script.push_str(&format!("Name-Comment: {}\n", params.comment.trim()));
        }
        if !params.email.trim().is_empty() {
            script.push_str(&format!("Name-Email: {}\n", params.email.trim()));
        }
        script.push_str(&format!("Expire-Date: {}\n", params.expire));

        if params.passphrase.is_empty() {
            script.push_str("%no-protection\n");
        } else {
            script.push_str(&format!("Passphrase: {}\n", params.passphrase));
        }

        script.push_str("%commit\n");
        script.push_str("%echo done\n");
        script
    }

    /// Imports an ASCII-armored key block.
    pub fn import_key(&self, armored_key: &str) -> Result<String, GpgError> {
        let mut child = Command::new("gpg")
            .args(["--import", "--batch", "--no-tty"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(armored_key.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        Ok(stderr)
    }

    /// Encrypts plaintext for the given recipient key ID. Returns ASCII-armored ciphertext.
    pub fn encrypt(&self, plaintext: &str, recipient_key_id: &str) -> Result<String, GpgError> {
        if recipient_key_id.is_empty() {
            return Err(GpgError::NoRecipient);
        }
        validate_key_id(recipient_key_id)?;

        let mut child = Command::new("gpg")
            .args([
                "--encrypt",
                "--armor",
                "--batch",
                "--no-tty",
                "--trust-model",
                "always",
                "--recipient",
                recipient_key_id,
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(plaintext.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        let ciphertext = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(ciphertext)
    }

    /// Decrypts ASCII-armored ciphertext. Returns a SecureString to prevent accidental leakage.
    pub fn decrypt(&self, ciphertext: &str) -> Result<SecureString, GpgError> {
        let mut child = Command::new("gpg")
            .args(["--decrypt", "--batch", "--no-tty"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(ciphertext.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(GpgError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr,
            });
        }

        let plaintext = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(SecureString::new(plaintext))
    }

    /// Parses the colon-delimited output of `gpg --list-keys --with-colons`.
    fn parse_key_listing(output: &str, _is_secret: bool) -> Result<Vec<PgpKey>, GpgError> {
        let mut keys: Vec<PgpKey> = Vec::new();
        let mut current: Option<PgpKey> = None;
        // Track fingerprints for deduplication
        let mut seen_fingerprints: HashMap<String, usize> = HashMap::new();

        for line in output.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.len() < 10 {
                continue;
            }

            match fields[0] {
                "pub" => {
                    if let Some(key) = current.take() {
                        let idx = keys.len();
                        seen_fingerprints.insert(key.key_id.clone(), idx);
                        keys.push(key);
                    }
                    current = Some(PgpKey {
                        key_id: fields[4].to_string(),
                        fingerprint: String::new(),
                        uid: String::new(),
                        has_secret: false,
                        algorithm: Self::algo_name(fields[3]),
                        key_length: fields[2].parse().unwrap_or(0),
                        creation_date: fields[5].to_string(),
                        expiration_date: if fields[6].is_empty() {
                            None
                        } else {
                            Some(fields[6].to_string())
                        },
                        trust: TrustLevel::from_colon_field(fields[1]),
                    });
                }
                "fpr" => {
                    if let Some(ref mut key) = current {
                        if key.fingerprint.is_empty() {
                            key.fingerprint = fields[9].to_string();
                        }
                    }
                }
                "uid" => {
                    if let Some(ref mut key) = current {
                        if key.uid.is_empty() {
                            key.uid = fields[9].to_string();
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(key) = current.take() {
            if !seen_fingerprints.contains_key(&key.key_id) {
                keys.push(key);
            }
        }

        Ok(keys)
    }

    fn algo_name(code: &str) -> String {
        match code {
            "1" => "RSA".to_string(),
            "16" => "ElGamal".to_string(),
            "17" => "DSA".to_string(),
            "18" => "ECDH".to_string(),
            "19" => "ECDSA".to_string(),
            "22" => "EdDSA".to_string(),
            other => format!("algo-{}", other),
        }
    }
}
