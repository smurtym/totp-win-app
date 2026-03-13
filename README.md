# TOTP Authenticator

Minimalistic Windows desktop app for TOTP two-factor authentication codes. Native Win32 UI, dark theme, near-zero CPU usage.

## Features

- 6-digit TOTP codes with live 30-second countdown
- One-click copy to clipboard per account
- Always-dark grey theme (`#2D2D2D`) — no system theme dependency
- Custom icon in taskbar, title bar, and File Explorer
- File-based config — no database, no UI for managing accounts

## Build

### On Windows (native)

**1. Install Microsoft C++ Build Tools**

The MSVC linker and Windows SDK are required. You have two options:

- **Option A (recommended):** Install [Visual Studio](https://visualstudio.microsoft.com/downloads/) (Community edition is free) and select the **"Desktop development with C++"** workload during setup. This includes the MSVC compiler, linker, and Windows SDK.
- **Option B (lighter):** Install the standalone [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and select the same **"Desktop development with C++"** workload.

> Note: A valid Visual Studio license (Community, Pro, or Enterprise) is required even for the standalone Build Tools.

**2. Install Rust**

Download and run the installer from [rustup.rs](https://rustup.rs). It will detect Windows and default to the MSVC toolchain (`stable-x86_64-pc-windows-msvc`). When prompted about installing MSVC prerequisites, select the default options.

After installation, verify with:
```powershell
cargo --version
```

**3. Build**

```powershell
cargo build --release
# output: target\release\totp-win-app.exe
```

### On Linux (cross-compile for Windows)

You can produce a Windows `.exe` from a Linux machine using MinGW-w64.

**1. Install the MinGW-w64 cross-compiler** (Debian/Ubuntu):

```bash
sudo apt install gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
```

**2. Add the Windows target to your Rust toolchain:**

```bash
rustup target add x86_64-pc-windows-gnu
```

**3. Build:**

```bash
cargo build --release --target x86_64-pc-windows-gnu
# output: target/x86_64-pc-windows-gnu/release/totp-win-app.exe
```

The `.cargo/config.toml` in this repo already configures Cargo to use `x86_64-w64-mingw32-gcc` as the linker, and `build.rs` handles the Windows resource (icon) embedding via `x86_64-w64-mingw32-windres` automatically.

> **Note:** The output `.exe` is a standalone PE32+ GUI binary — no Wine or Windows emulator needed to build it. You do need Windows to _run_ it.

## Setup

Create `secrets.txt` in the same folder as the exe:

```
# One account per line: Name=BASE32SECRET
GitHub=JBSWY3DPEHPK3PXP
AWS=HXDMVJECJJWSRB3H
Google=MFRGGZDFMZTWQ2LK
```

Lines starting with `#` and empty lines are ignored. Invalid secrets are skipped and counted in the footer. See `secrets.txt.example` for reference.

**To get a secret**: when enabling 2FA on a service, choose "Manual entry" / "Can't scan QR code" and copy the base32 key shown.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| "Failed to load secrets.txt" | Create the file next to the exe |
| "No accounts configured" | Add at least one `Name=SECRET` line |
| Codes don't match | Sync your system clock (TOTP is time-based) |
| N invalid lines shown | Check those lines are `Name=VALIDBASE32` format |

## Architecture

```
src/
├── main.rs      # startup, message loop
├── ui.rs        # Win32 window, painting, buttons
├── totp.rs      # TOTP generation (RFC 6238)
├── file.rs      # secrets.txt parsing
└── clipboard.rs # Win32 clipboard
assets/
└── app.ico      # key-themed icon (embedded at build time via build.rs)
```

**Stack**: Rust · windows-rs 0.52 · totp-lite 2.0 · winres 0.1

## Performance

CPU is near-zero between 30-second rollovers — only the countdown strip (~32px) is repainted each second. HMAC-SHA1 runs once per 30s period, with codes cached in a thread-local. Base32 decoding happens once at startup per account.

## Security

`secrets.txt` contains the raw TOTP shared secrets — treat it like a password file. Do not commit it to source control (it is in `.gitignore`). Back it up to a secure location.