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

## GitHub Branches

The repository has two branches:

- **`main`** — the regular source tree. Building from this branch will pull all Rust dependencies from `crates.io` during the first build.
- **`vendor`** — identical to `main` but with the full `vendor/` directory and a matching `.cargo/config.toml` committed. Cargo will then build fully offline, without contacting `crates.io` at all.

The `vendor` branch exists specifically for **Tails OS** users. On Tails, all network traffic goes through Tor, and `cargo fetch` against `crates.io` is often unreliable (timeouts, partial downloads, index churn). Using the `vendor` branch sidesteps the problem entirely — you clone once and build without any further network access.

```bash
# Tails / offline-friendly build
git clone --branch vendor https://github.com/dczerwinski/SimplePGP.git
cd SimplePGP
cargo build --release --offline
```

```bash
# Regular build (main branch)
git clone https://github.com/dczerwinski/SimplePGP.git
cd SimplePGP
cargo build --release
```

### Tails: Cargo network stability (only if building from `main`)

If for some reason you are building from `main` on Tails and see intermittent fetch errors from `crates.io`, the following exports help the Cargo client survive flaky Tor circuits:

```bash
cat >> ~/.bashrc << 'EOF'
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
export CARGO_HTTP_MULTIPLEXING=false
export CARGO_HTTP_TIMEOUT=600
export CARGO_NET_RETRY=20
EOF

source ~/.bashrc
```

Then:

```bash
cargo fetch -vv
cargo build --release -vv
```

If this keeps failing, switch to the `vendor` branch instead.

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

## Desktop integration (Linux)

To register the application icon and `.desktop` entry so SimplePGP shows up correctly in GNOME / the dock / Alt-Tab, use the helper scripts in `scripts/`:

```bash
# Per-user install (no root required)
scripts/install-linux.sh

# System-wide install
sudo scripts/install-linux.sh --system
```

To remove the installed entries:

```bash
scripts/uninstall-linux.sh
# or
sudo scripts/uninstall-linux.sh --system
```

The scripts install files from `data/` (`.desktop` file and the hicolor icons) and, if a release binary exists, rewrite `Exec=` to point at `target/release/simplepgp`.

### Tails OS (persistent install)

On Tails, `~/.local/share/` is wiped on every reboot, so `install-linux.sh` alone is not enough. Use `install-tails.sh` instead — it writes the `.desktop` file and icons into the Dotfiles area of Persistent Storage (`/live/persistence/TailsData_unlocked/dotfiles/`), so Tails restores them on every boot.

Prerequisites:

- Persistent Storage is unlocked.
- The **Dotfiles** feature is enabled in *Applications → Tails → Persistent Storage*.
- The project lives inside `~/Persistent/` (e.g. `~/Persistent/SimplePGP`) and is already built with `cargo build --release`.

```bash
# Persistent install on Tails (run as the amnesia user; it will sudo internally)
scripts/install-tails.sh

# Custom binary path (if you built somewhere else inside Persistent)
scripts/install-tails.sh --binary /home/amnesia/Persistent/SimplePGP/target/release/simplepgp

# Remove the persistent entry
scripts/install-tails.sh --uninstall
```

The script also activates the entry in the current session, so SimplePGP appears in Activities / the dock without a reboot.

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
│   ├── about.rs            # About dialog
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
