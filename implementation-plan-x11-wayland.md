# Implementation Plan: Kaku Full Linux Backend (X11/Wayland)

**Status**: Draft
**Author**: Alex Mikhalev
**Date**: 2026-03-09
**Based on**: WezTerm Linux Implementation Research

## Overview

This plan outlines implementing a full X11/Wayland backend for Kaku, leveraging WezTerm's proven implementation patterns.

### Approach
Adapt WezTerm's well-tested Linux windowing code rather than writing from scratch. WezTerm has 6+ years of production hardening.

### Scope

**MVP (Phase 1 - 3 weeks)**
- X11 backend with basic windowing
- OpenGL rendering via EGL
- Keyboard input
- Basic clipboard

**Full (Phase 2 - 2-3 months)**
- Wayland backend
- Full IME support
- Clipboard (X11 selections + Wayland data device)
- Drag and drop

**Out of Scope**
- DirectFB/other backends
- Hardware cursors initially

## Architecture

### Component Diagram
```
┌─────────────────────────────────────────────────────────────┐
│                     kaku-gui                                 │
│                   (main binary)                              │
├─────────────────────────────────────────────────────────────┤
│  kaku-gui/src/termwindow/webgpu.rs                         │
│  - Uses wgpu for GPU rendering                               │
│  - Needs valid window/display handles                        │
├─────────────────────────────────────────────────────────────┤
│                    window crate                               │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌──────────────────┐                │
│  │  x_and_wayland │  │   x11/           │  wayland/      │
│  │  (new file)    │  │   connection.rs  │  connection.rs  │
│  │                 │  │   window.rs     │  window.rs     │
│  └─────────────────┘  └──────────────────┘                │
├─────────────────────────────────────────────────────────────┤
│  egl.rs (adapt from WezTerm)                                │
├─────────────────────────────────────────────────────────────┤
│  Platform Dependencies:                                      │
│  ┌──────────┐ ┌─────────────┐ ┌────────────┐ ┌─────────┐ │
│  │  xcb     │ │ wayland-client│ │ xkbcommon │ │ glium  │ │
│  └──────────┘ └─────────────┘ └────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale | Alternative |
|----------|-----------|------------|
| X11 first | More universal, easier debugging | Wayland first |
| Direct xcb | WezTerm proven, full control | winit (less control) |
| mio event loop | WezTerm proven | async-std, tokio |
| glium for OpenGL | WezTerm uses it | wgpu directly |

### Eliminated Options

| Option Rejected | Why | Risk of Including |
|----------------|-----|-------------------|
| winit crate | Less flexible, WezTerm proved custom works | Over-simplification |
| Raw Xlib | xcb is safer and async-friendly | Complexity |
| Wayland first | X11 more universal for testing | Delay |

## File Changes

### New Files

| File | Purpose | Lines Est |
|------|---------|-----------|
| `window/src/os/x_and_wayland.rs` | Unified entry, backend selection | ~200 |
| `window/src/os/x11/mod.rs` | X11 module organization | ~50 |
| `window/src/os/x11/connection.rs` | X11 connection management | ~900 |
| `window/src/os/x11/window.rs` | X11 window implementation | ~1500 |
| `window/src/os/x11/events.rs` | X11 event handling | ~500 |
| `window/src/os/wayland/mod.rs` | Wayland module | ~50 |
| `window/src/os/wayland/connection.rs` | Wayland connection | ~200 |
| `window/src/os/wayland/window.rs` | Wayland window | ~900 |

### Modified Files

| File | Changes |
|------|---------|
| `window/Cargo.toml` | Add xcb, wayland deps |
| `window/src/os/mod.rs` | Add X11/Wayland exports |
| `window/src/os/linux/window.rs` | Delegate to real impl |
| `window/src/os/linux/connection.rs` impl |

## Dependencies

### New | Delegate to real Cargo Dependencies

```toml
[target.'cfg(target_os="linux")'.dependencies]
xcb = { version = "1.15", features = ["render", "randr", "dri2", "xkb"] }
xkbcommon = "0.7"
mio = "0.8"
libc = "0.2"

[features]
wayland = ["wayland-client", "wayland-egl", "wayland-protocols"]
```

## Implementation Steps

### Step 1: Add Dependencies & Skeleton (2 days)
**Files:** `window/Cargo.toml`, `window/src/os/mod.rs`
**Description:** Add xcb, create X11 module structure
**Tests:** cargo check passes
**Dependencies:** None

```rust
// window/src/os/mod.rs - Add
#[cfg(target_os = "linux")]
pub mod x_and_wayland;
#[cfg(target_os = "linux")]
pub use x_and_wayland::*;
```

### Step 2: X11 Connection (3 days)
**Files:** `window/src/os/x11/connection.rs`
**Description:** Implement X11 connection with mio event loop
**Tests:** Connect to X server, handle basic events
**Dependencies:** Step 1

Key implementation:
```rust
pub struct XConnection {
    conn: xcb::Connection,
    setup: xcb::Setup,
    screen: Screen,
    pub(crate) windows: RefCell<HashMap<u32, XWindowInfo>>,
    // ... mio registration
}
```

### Step 3: X11 Window (4 days)
**Files:** `window/src/os/x11/window.rs`
**Description:** Create/destroy windows, handle resize, position
**Tests:** Window appears, responds to resize
**Dependencies:** Step 2

### Step 4: X11 Event Handling (2 days)
**Files:** `window/src/os/x11/events.rs`
**Description:** Keyboard, mouse, expose events
**Tests:** Keyboard input works, mouse clicks work
**Dependencies:** Step 3

### Step 5: EGL Integration (2 days)
**Files:** Update `window/src/egl.rs` 
**Description:** Add X11 EGL surface creation
**Tests:** OpenGL context creates, rendering works
**Dependencies:** Step 3

### Step 6: Keyboard with xkbcommon (2 days)
**Files:** `window/src/os/x11/keyboard.rs`
**Description:** Full keyboard handling with layout support
**Tests:** All keys work, layouts switch
**Dependencies:** Step 4

### Step 7: Clipboard (1 day)
**Files:** `window/src/os/x11/clipboard.rs`
**Description:** X11 selection clipboard
**Tests:** Copy/paste works
**Dependencies:** Step 4

### Step 8: Wayland Backend (optional, 2-3 weeks)
**Files:** `window/src/os/wayland/*`
**Description:** Mirror X11 implementation for Wayland
**Tests:** Works on Wayland compositor

## Test Strategy

### Unit Tests
| Test | Location | Purpose |
|------|----------|---------|
| x11_connection_test | x11/connection.rs | Connection creates |
| x11_window_test | x11/window.rs | Window creates/destroys |
| keymap_test | x11/keyboard.rs | Keycode mapping |

### Integration Tests
| Test | Location | Purpose |
|------|----------|---------|
| launch_test | - | Binary launches window |
| type_test | - | Can type in terminal |
| copy_paste_test | - | Clipboard works |

## Performance Considerations

| Metric | Target | Measurement |
|--------|--------|-------------|
| Launch time | < 500ms | Benchmark |
| Input latency | < 10ms | Event to render |
| Memory | < 50MB | /usr/bin/time -v |

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| WezTerm code licensing | Need to verify | Alex |
| Test environment setup | Pending | Alex |

## Approval

- [ ] Technical review complete
- [ ] WezTerm code licensing verified
- [ ] Test strategy approved
- [ ] Human approval received
