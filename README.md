# SimplePGP

A privacy-focused PGP key management desktop application built with Rust, GTK4, and libadwaita. Designed for use on Tails OS and other security-sensitive Debian-based environments.

## Features

- **Key Management** — View all public and secret keys from your GnuPG keyring
- **Key Import** — Import ASCII-armored PGP keys via paste
- **Encrypt** — Encrypt text for any recipient in your keyring
- **Decrypt** — Decrypt PGP-encrypted messages
- **Clipboard** — One-click copy of encrypted/decrypted output
- **Security** — No secrets stored on disk, sensitive buffers zeroized, no network calls

## Requirements

### System Dependencies (Debian/Tails)

```bash
sudo apt install \
    build-essential \
    libgtk-4-dev \
    libadwaita-1-dev \
    gnupg \
    pkg-config \
    curl
```

### Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Build

```bash
cargo build --release
```

The binary will be at `target/release/simplepgp`.

## Run

```bash
cargo run --release
```

Or run the binary directly:

```bash
./target/release/simplepgp
```

## Architecture

```
src/
├── main.rs                 # Entry point
├── app.rs                  # Application setup
├── models/
│   └── key.rs              # PgpKey data model
├── services/
│   └── gpg_service.rs      # GnuPG CLI wrapper
├── viewmodels/
│   ├── key_list_vm.rs      # Key list state management
│   └── crypto_vm.rs        # Encrypt/decrypt state management
├── ui/
│   ├── main_window.rs      # Main window layout
│   ├── key_list_view.rs    # Keys tab
│   ├── encrypt_view.rs     # Encrypt tab
│   ├── decrypt_view.rs     # Decrypt tab
│   └── dialogs.rs          # Alert/error dialogs
├── security/
│   ├── memory.rs           # Input validation
│   └── secure_clear.rs     # Zeroizing string wrapper
└── utils/
    ├── clipboard.rs        # Clipboard operations
    └── async_runtime.rs    # Background task runner
```

## Security Notes

- All GPG operations use the system `gpg` binary — no custom cryptography
- Decrypted text is held in `SecureString` and zeroized on drop
- No temporary files are created
- All subprocess calls use argument arrays (no shell injection)
- No logging of sensitive data
- No network connections or telemetry
- Input validation prevents shell metacharacter injection

## License

Apache License 2.0 (Apache-2.0)
