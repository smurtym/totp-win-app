# Feature Specification: TOTP Windows Desktop Application

**Status**: Implemented  
**Branch**: `main`

## Overview

A minimalistic Windows desktop TOTP authenticator. Secrets stored in a plain-text file the user edits manually. No add/edit UI. Shows 30-second countdown, copy-to-clipboard button per account, dark grey theme, custom icon.

---

## User Stories

### US-1 — View TOTP Codes with Countdown (P1)

As a user I want to see all my TOTP codes with a live countdown so I know the current code and when it rotates.

**Acceptance scenarios**:
1. On launch, all accounts show name + 6-digit code + "Next code in: Xs" countdown (value visible immediately, not after first tick).
2. When the 30s period expires, codes update and countdown resets to 30. "0s" is never displayed.
3. Multiple accounts are synchronized to the same 30-second boundary.

---

### US-2 — Copy Code to Clipboard (P2)

As a user I want a Copy button per account so I can paste the code without typing it.

**Acceptance scenarios**:
1. Clicking Copy puts only the 6-digit code on the clipboard (no extra formatting).
2. Clicking Copy for a second account replaces the clipboard with the new code.

---

### US-3 — Dark Theme Always On (P2)

As a user I want the window to always use a dark grey theme including buttons so it is easy on the eyes.

**Acceptance scenarios**:
1. Window background is always `#2D2D2D` regardless of system theme.
2. Copy buttons render with `#4A4A4A` background, `#888888` border, light text.
3. Button shows pressed state (`#666666`) on click.

---

### US-4 — Custom Application Icon (P3)

As a user I want a distinctive icon in the taskbar and title bar so I can identify the app quickly.

**Acceptance scenarios**:
1. Custom key-themed icon shows in title bar, taskbar, alt-tab, and File Explorer.

---

## Requirements

| ID | Requirement |
|----|-------------|
| FR-001 | Display 6-digit TOTP codes for all accounts in secrets.txt |
| FR-002 | Auto-refresh codes at the 30-second TOTP boundary |
| FR-003 | Countdown shows seconds remaining; visible from first paint; never shows 0 |
| FR-004 | Load accounts from `secrets.txt` (same directory as exe) on startup |
| FR-005 | No UI for adding/editing/deleting accounts — user edits file directly |
| FR-006 | RFC 6238 compliant: HMAC-SHA1, 6-digit codes, 30-second intervals |
| FR-007 | Validate base32 secrets; skip invalid entries and report count |
| FR-008 | Copy button per account copies 6-digit code to clipboard |
| FR-009 | Window always uses dark grey (`#2D2D2D`) — no system theme detection |
| FR-010 | Custom key-themed icon embedded in exe (`assets/app.ico`) |
| FR-011 | Fixed window size — no resize, no maximize |
| FR-012 | Native Win32 UI only (no third-party UI framework) |

---

## Success Criteria

- App starts in < 200ms
- CPU usage is near zero between 30-second rollovers (only countdown strip repainted each second)
- TOTP codes match reference apps (Google Authenticator, Authy) for the same secret
- Icon visible in title bar, taskbar, alt-tab, File Explorer
- Invalid secrets.txt lines reported with count; valid accounts still display

---

## Design Decisions

- **Always dark**: `#2D2D2D` background hardcoded — no `DwmGetWindowAttribute` or registry check.
- **BS_OWNERDRAW buttons**: WM_CTLCOLORBTN is silently ignored by UXTheme on Win10/11; owner-draw is the correct solution.
- **Partial invalidation**: `WM_TIMER` invalidates only the ~32px countdown strip each second. Code rows are only re-invalidated on the 30s rollover.
- **Code cache**: HMAC-SHA1 computed once per 30s period, stored in thread-local `CODE_CACHE`. Reads are zero-cost between rollovers.
- **Secret decoded once**: `base32_decode` runs at `Account::new()` time; raw bytes stored in `Account.secret_bytes`.
- **AdjustWindowRect**: Client area is 340×(10 + N×40 + 32)px; Win32 adjusts to outer window size accounting for title bar chrome.
- **Icon resource ID 1**: Embedded by `build.rs` via `winres`; loaded with `PCWSTR(1 as *const u16)`.
