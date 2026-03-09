# Implementation Plan: Kaku Linux Port

**Status**: Draft
**Research Doc**: research-linux-port.md
**Author**: Alex Mikhalev
**Date**: 2026-03-09
**Estimated Effort**: 16-24 hours

## Overview

### Summary
Port Kaku terminal emulator from macOS-only to Linux, supporting both X11 and Wayland windowing systems.

### Approach
Leverage WezTerm's existing Linux support by adapting the window crate, creating Linux-specific platform bindings, and modifying build scripts. Use WezTerm as reference implementation for platform-specific code.

### Scope
**In Scope:**
- Basic terminal functionality (terminal + shell)
- X11 and Wayland support via window crate
- Core keybindings (remapped from macOS Cmd to Ctrl)
- Build to standalone binary
- Basic shell integration

**Out of Scope:**
- Native Linux notifications (use system fallback)
- macOS-style menu bar
- DMG packaging (use AppImage/deb instead)
- First-run automation
- Kaku config TUI (Phase 2)
- AI assistant integration (Phase 2)

**Avoid At All Cost** (from 5/25 analysis):
- Building from scratch - use WezTerm as reference
- Creating new windowing code - adapt existing patterns
- Heavy feature additions - keep MVP focused

## Architecture

### Component Diagram
```
┌─────────────────────────────────────────────────────┐
│                    kaku-gui                         │
│              (main application)                      │
├─────────────────────────────────────────────────────┤
│  kaku-gui/src/    │  window/   │   mux/           │
│  - frontend.rs     │  - os/     │   - server.rs    │
│  - commands.rs     │  - linux/  │   - pty.rs      │
│  - tabbar.rs      │  - egl.rs  │                  │
├─────────────────────────────────────────────────────┤
│                    termwiz                          │
│              (terminal emulation)                    │
├─────────────────────────────────────────────────────┤
│  Platform Layer:                                     │
│  ┌─────────────┐ ┌─────────────┐ ┌──────────────┐ │
│  │   X11/      │ │   EGL/      │ │  portable-pty │ │
│  │   Wayland   │ │   wgpu      │ │              │ │
│  └─────────────┘ └─────────────┘ └──────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Data Flow
```
User Input → window/os/linux (key events) 
         → kaku-gui/commands.rs 
         → termwiz (terminal processing)
         → mux/pty (shell communication)
         → Display ← window/render
```

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Use WezTerm window patterns | Proven cross-platform, less code | Writing from scratch |
| Support both X11 and Wayland | Linux desktop diversity | X11-only or Wayland-only |
| Keep Kaku shell defaults | Value proposition | Using system defaults |
| Binary output first | Faster iteration | Flatpak/AppImage initially |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| Flatpak packaging | Adds complexity | Delay ship date |
| Native notifications | Can use fallback later | Scope creep |
| Config TUI | Not MVP | Phase 2 only |
| AI assistant | Platform-agnostic but needs testing | Phase 2 |

### Simplicity Check

The simplest approach: adapt window crate to Linux, build binary, test core functionality. WezTerm already does this - we follow their patterns.

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `window/src/os/linux/mod.rs` | Linux window implementation |
| `window/src/os/linux/x11.rs` | X11 window backend |
| `window/src/os/linux/wayland.rs` | Wayland window backend |
| `scripts/build-linux.sh` | Linux build script |
| `assets/linux/kaku.desktop` | Desktop entry |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Add Linux targets |
| `window/Cargo.toml` | Add Linux dependencies |
| `window/src/os/mod.rs` | Add Linux exports |
| `kaku-gui/Cargo.toml` | Add Linux deps |
| `scripts/build.sh` | Add Linux branch |

### Deleted Files
None in MVP

## Implementation Steps

### Step 1: Prepare Build Environment
**Files:** N/A
**Description:** Install Linux build dependencies, configure Rust
**Tests:** N/A
**Estimated:** 1 hour

```bash
# Install dependencies (from WezTerm)
sudo apt install build-essential cmake libfreetype-dev libfontconfig1-dev \
  libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libegl-dev libgl-dev \
  libpango1.0-dev libatk1.0-dev libgtk-3-dev
```

### Step 2: Add Linux Targets to Cargo.toml
**Files:** `Cargo.toml`, `window/Cargo.toml`, `kaku-gui/Cargo.toml`
**Description:** Add Linux-specific dependencies
**Tests:** cargo check passes
**Estimated:** 2 hours

```toml
# window/Cargo.toml - Add Linux dependencies
[target.'cfg(target_os="linux")'.dependencies]
xcb = { version = "1.15", optional = true }
wayland-client = { version = "0.31", optional = true }
```

### Step 3: Create Linux Window Implementation
**Files:** `window/src/os/linux/mod.rs`, `x11.rs`, `wayland.rs`
**Description:** Implement Linux window using patterns from WezTerm
**Tests:** Binary launches, window appears
**Estimated:** 8-16 hours

Key code structure:
```rust
// window/src/os/mod.rs
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
```

### Step 4: Adapt Build Script
**Files:** `scripts/build.sh`
**Description:** Create Linux build that produces standalone binary
**Tests:** Binary builds successfully
**Estimated:** 2 hours

```bash
# Replace macOS-specific build with Linux version
if [[ "${OSTYPE}" == "linux-gnu"* ]]; then
    cargo build --release
    # Package as AppImage or just use binary
fi
```

### Step 5: Test Core Functionality
**Description:** Verify terminal works with shell
**Tests:** 
- Terminal launches
- Shell spawns
- Basic keybindings work
- Text renders correctly

**Estimated:** 2 hours

### Step 6: Keybinding Remapping
**Files:** `kaku-gui/src/inputmap.rs`
**Description:** Map Cmd → Ctrl, adjust macOS shortcuts
**Tests:** All shortcuts work
**Estimated:** 1 hour

## Test Strategy

### Unit Tests
| Test | Location | Purpose |
|------|----------|---------|
| `test_linux_window_creation` | linux/mod.rs | Window creates |
| `test_wayland_connection` | linux/wayland.rs | Connects to compositor |
| `test_x11_connection` | linux/x11.rs | Connects to X server |

### Integration Tests
| Test | Location | Purpose |
|------|----------|---------|
| `test_terminal_launch` | kaku-gui | Terminal opens |
| `test_shell_spawn` | mux | Shell process starts |
| `test_keybinding_linux` | inputmap | Shortcuts work |

## Dependencies

### New Dependencies

| Crate | Version | Justification |
|-------|---------|---------------|
| xcb | 1.15 | X11 window support |
| wayland-client | 0.31 | Wayland support |
| xkbcommon-x11 | 0.5 | Keyboard handling |

### No New Dependencies (Using Existing)
- wgpu - already in dependencies
- portable-pty - already supports Linux
- termwiz - already cross-platform

## Performance Considerations

### Expected Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Launch time | < 1s | Benchmark |
| Binary size | < 50MB | ls -lh |
| Memory usage | < 100MB | /usr/bin/time -v |

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Clean disk space for build | Pending | Alex |
| Clone git submodules | Pending | Alex |
| Test X11 build | Pending | Alex |
| Test Wayland build | Pending | Alex |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Performance targets agreed
- [ ] Human approval received
