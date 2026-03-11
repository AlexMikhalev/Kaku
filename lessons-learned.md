# Lessons Learned: Kaku Linux Port

## Date: 2026-03-10
## Branch: linux-port
## Status: In Progress - X11 Implementation

## Key Discoveries

### 1. Dependency Version Critical
WezTerm uses **patched** versions of xcb/xkbcommon that are NOT the latest available:
- xcb = "1.3" (NOT 1.7)
- xkbcommon = "0.7.0" (NOT 0.9)
- x11 = "2.21"

Using newer versions causes 74+ API errors because the API changed significantly.

### 2. xcb 1.3 API Differences
- No `.checked()` / `.unchecked()` builder methods - requests are synchronous by default
- `Screen` is in `xcb::x::Screen` not `xcb::Screen`
- `Atom::from_reply()` doesn't exist - use `get_atom` directly
- `ConvertSelection` requires `time` field (CurrentTime)

### 3. Stub Window Approach
Initial stub compilation worked but panics on WebGPU init (needs real window handles).

### 4. X11 Implementation Complexity
The `window` crate requires:
- Connection management
- Window creation/destruction
- Event loop integration
- Keyboard handling with xkbcommon
- Clipboard operations
- All traits: Copy, Clone, Debug, Eq, Hash, Ord

## What Worked

1. **Stub compilation** - Created basic stubs that compiled
2. **Dependency matching** - Updated Cargo.toml to use WezTerm's exact versions
3. **Connection rewrite** - Fixed connection.rs for xcb 1.3 API

## Current Issues

Still 73+ errors in X11 implementation files:
- Trait implementations missing (Copy, Debug, Clone, Eq, Hash)
- API method mismatches (xcb 1.3 vs 1.7)
- Type mismatches in window/keyboard/clipboard code

## Next Steps

Option A: Simplify to stubs that compile, test binary
Option B: Continue fixing full X11 implementation

## References
- WezTerm source: https://github.com/wez/wezterm
- xcb crate: https://docs.rs/xcb/1.3.0/
