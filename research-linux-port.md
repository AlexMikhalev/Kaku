# Research Document: Kaku Linux Port

**Status**: ✅ COMPLETE (Stub Implementation)
**Author**: Alex Mikhalev
**Date**: 2026-03-09
**GitHub Branch**: https://github.com/AlexMikhalev/Kaku/tree/linux-port

## Executive Summary

Kaku is a macOS-only terminal emulator forked from WezTerm, customized for AI coding workflows. This research assesses the effort to create a Linux port. Since WezTerm already has excellent Linux support, the core terminal functionality should be reusable. The main effort involves creating Linux-specific window/UI bindings and adapting build scripts.

## Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | Terminal is my daily driver, Linux is my primary OS |
| Leverages strengths? | Yes | Existing Rust/WezTerm knowledge, Linux sysadmin skills |
| Meets real need? | Yes | Want Kaku's AI-coding features on Linux |

**Proceed**: Yes (3/3 YES)

## Problem Statement

### Description
Port Kaku terminal emulator (currently macOS-only) to Linux, preserving AI-coding features while adapting to Linux windowing systems (X11/Wayland).

### Impact
- Users gain access to Kaku's curated AI-coding terminal on Linux
- Differentiates from WezTerm by maintaining Kaku's shell defaults

### Success Criteria
1. Terminal launches and displays correctly on Linux (X11 and Wayland)
2. Basic shell integration works (zsh with starship, z, delta)
3. Kaku-specific features (AI assistant, config) function
4. Build produces Linux binaries

## Current State Analysis

### Existing Implementation

| Component | Location | Purpose | Linux Status |
|-----------|----------|---------|--------------|
| Terminal core | `term/`, `termwiz/` | Terminal emulation | ✅ Cross-platform |
| Multiplexer | `mux/` | PTY/tab management | ✅ Cross-platform |
| GUI framework | `kaku-gui/` | Main application | ⚠️ macOS-heavy |
| Window management | `window/` | OS window integration | ❌ macOS only |
| Build scripts | `scripts/build.sh` | Packaging | ❌ macOS only |
| Assets | `assets/` | Fonts, icons, shell integration | ✅ Reusable |

### Code Locations - Platform-Specific

**macOS-specific directories:**
- `window/src/os/macos/` - Window implementation
- `assets/macos/` - macOS app bundle, entitlements

**Platform conditionals in Cargo.toml:**
- `window/Cargo.toml` lines 51-58: macOS-only dependencies (cocoa, core-foundation)
- `kaku-gui/Cargo.toml` lines 102-118: macOS/Windows targets only

### Key Platform-Specific Code

1. **window/src/os/mod.rs** - Currently only exports macos:
```rust
mod macos;
pub use macos::*;
```

2. **kaku-gui/src/main.rs** - Contains macOS-specific app initialization

3. **scripts/build.sh** - Line 6: `if [[ "${OSTYPE:-}" != darwin* ]]; then` - explicitly rejects non-macOS

## Constraints

### Technical Constraints

| Constraint | Description | Source |
|------------|-------------|--------|
| Windowing systems | Must support X11 and Wayland | Linux desktop requirement |
| GPU rendering | Must use OpenGL/Vulkan on Linux | wgpu supports Linux |
| Shell defaults | Kaku assumes macOS paths | Need Linux equivalents |
| Binary size | Target < 50MB (like Kaku) | Performance requirement |

### Dependencies - External

| Dependency | Current | Linux Support | Risk |
|------------|---------|---------------|------|
| wgpu | Latest | ✅ Good | Low |
| tiny-skia | Latest | ✅ Good | Low |
| portable-pty | Latest | ✅ Good | Low |
| wezterm-* crates | Forked | ✅ Cross-platform | Low |
| cocoa/core-foundation | macOS only | ❌ Not available | **High** - needs replacement |

### Business Constraints
- **Timeline**: No fixed deadline
- **Resources**: Single developer (myself)
- **Maintenance**: Must keep sync with upstream Kaku

## Vital Few (Essentialism)

### Essential Constraints (Max 3)

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Window crate port | Core UI functionality | No window = no app |
| Build system | Produce Linux binaries | Can't ship without it |
| Shell integration | Kaku's value proposition | Starship, z, delta need Linux paths |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| Native notifications | Can use fallback (dunst) |
| macOS-style menu bar | Different paradigm on Linux |
| DMG packaging | Use AppImage/deb instead |
| First-run setup automation | Manual config acceptable initially |

## Dependencies

### Internal Dependencies

| Dependency | Impact | Risk |
|------------|--------|------|
| WezTerm upstream | Core terminal code is cross-platform | Low - already works |
| Kaku customizations | AI features, config system | Medium - needs testing |
| Shell defaults | Starship, z, delta configs | Low - text files |

### External Dependencies - New for Linux

| Dependency | Purpose | Alternative |
|------------|---------|-------------|
| xcb or wayland-client | Window system bindings | Use existing wezterm dependencies |
| libxkbcommon | Keyboard handling | Already in wezterm |
| libegl | EGL for GPU | Already in wezterm |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Kaku-specific macOS code in GUI | Medium | High | Review all platform conditionals |
| GPU driver issues on Linux | Low | Medium | Use wgpu fallback pipeline |
| Font rendering differences | Low | Low | Test with bundled fonts |

### Open Questions

1. **How to handle Kaku's macOS-specific keybindings?** - Need to map Cmd to Ctrl, etc.
2. **Shell integration path assumptions?** - Kaku assumes `/opt/homebrew/bin`, need Linux equivalents
3. **Config location?** - macOS uses `~/Library/Application Support/kaku`, Linux should use XDG spec

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong |
|------------|-------|---------------|
| WezTerm's terminal core works on Linux | WezTerm has Linux packages | Low - proven |
| window crate can be adapted | WezTerm has similar structure | Medium - needs code |
| Kaku's AI features are platform-agnostic | They seem to be CLI-based | Low - will test |

## Research Findings

### Key Insights

1. **WezTerm already supports Linux** - The core terminal, PTY, and most GUI logic is cross-platform
2. **Window crate is the bottleneck** - Only macOS implementation exists; needs Linux port
3. **Build system is simple to adapt** - Most of `build.sh` can be replaced with Linux equivalents
4. **Kaku adds value on top** - Shell defaults, AI assistant, config TUI are platform-agnostic

### Technical Spikes Needed

| Spike | Purpose | Estimated Effort |
|-------|---------|------------------|
| Compile attempt | See what breaks on Linux | 2 hours |
| Window crate port | Create Linux window impl | 8-16 hours |
| Build script | Linux packaging | 4 hours |

### Relevant Prior Art

- **WezTerm Linux support** - Already exists, can reference
- **Ghostty Linux port** - Community packages available (mkasberg/ghostty-ubuntu)
- **Alacritty** - Mature cross-platform terminal

## Recommendations

### Proceed/No-Proceed

**Proceed** with Linux port. The risk is manageable because:
- Core terminal code is proven cross-platform
- Window crate adaptation follows established patterns
- Scope is well-defined and limited

### Scope Recommendations

**MVP (Phase 1):**
- Basic terminal functionality (terminal + shell)
- X11 and Wayland support
- Core keybindings (remapped from macOS)
- Build to standalone binary

**Phase 2:**
- Kaku-specific config TUI
- AI assistant integration
- Shell defaults (starship, z, delta)
- AppImage/deb packaging

### Risk Mitigation Recommendations

1. **Start with compilation attempt** - Will reveal actual issues quickly
2. **Use WezTerm as reference** - Their Linux code is open source
3. **Keep upstream in sync** - Use git rebase to avoid drift

## Next Steps

If approved:

1. **Attempt cargo build on Linux** - See what platform-specific errors occur
2. **Create Linux window implementation** - Mirror `window/src/os/macos/` for Linux
3. **Adapt build scripts** - Replace macOS bundle with Linux binary
4. **Test core functionality** - Terminal, shell, basic keybindings

## Implementation Results

### What Was Achieved

| Milestone | Status |
|-----------|--------|
| Code compiles on Linux | ✅ Done |
| Release binary builds | ✅ Done (~23MB for kaku-gui) |
| Binary runs | ✅ Runs but panics on WebGPU init |
| Window rendering | ❌ Needs X11/Wayland backend |

### Files Changed

- `crates/wezterm-toast-notification/src/lib.rs` - Added Linux stub
- `crates/wezterm-toast-notification/src/linux.rs` - New file
- `window/Cargo.toml` - Added Linux dependencies
- `window/src/os/mod.rs` - Added Linux module exports
- `window/src/os/linux/mod.rs` - New file
- `window/src/os/linux/connection.rs` - New file
- `window/src/os/linux/window.rs` - New file
- `window/src/os/linux/clipboard.rs` - New file
- `window/src/os/linux/keycodes.rs` - New file

### Current Limitation

The stub implementation returns `Err(HandleError::Unavailable)` for window/display handles, which causes WebGPU initialization to panic. To make the terminal fully functional, the Linux window implementation needs:

1. **X11 backend** - Using xcb crate
2. **OR Wayland backend** - Using wayland-client crate
3. **Event loop** - Main loop that handles window events
4. **OpenGL/EGL** - GPU rendering setup

This is estimated at 40-80 hours of additional work.

## Appendix

### Reference Materials

- WezTerm Linux build: https://wezterm.org/install/source.html
- Ghostty Ubuntu packages: https://github.com/mkasberg/ghostty-ubuntu
- Kaku upstream: https://github.com/tw93/Kaku

### Code Snippets

**Current window/src/os/mod.rs:**
```rust
mod macos;
pub use macos::*;
```

**Need to add:**
```rust
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
```
