# Disciplined Research + Design Task List: Kaku Linux Fork Continuation

**Status**: Draft
**Date**: 2026-03-11
**Branch**: `linux-port`
**Research Inputs**:
- `research-linux-port.md`
- `lessons-learned.md`
- current `cargo check -p window` baseline on 2026-03-11
**Design Input Reviewed and Narrowed**:
- `implementation-plan-x11-wayland.md`

## Skill Use

This task list applies:
- **disciplined-research** to restate the active problem, constraints, assumptions, and risks from the current codebase rather than from the earlier stub-only milestone.
- **disciplined-design** to reduce scope to the minimum compile-first slice and sequence the next work into reviewable implementation batches.

## Phase 1 Research Refresh

### Executive Summary

The Linux fork is past the original "stub compiles" milestone but is not yet at a stable backend boundary. The current branch contains a partial X11 implementation, and the main problem is no longer broad Linux-port feasibility. The active problem is narrower: the Linux facade and X11 modules disagree on types, traits, and API boundaries, which blocks `cargo check -p window`.

### Essential Questions Check

| Question | Answer | Evidence |
|----------|--------|----------|
| Energizing? | Yes | Linux remains the target platform for daily use |
| Leverages strengths? | Yes | Existing Kaku/WezTerm knowledge and ongoing X11 branch work |
| Meets real need? | Yes | Current Linux backend still cannot serve as a real platform foundation |

**Proceed**: Yes

## Problem Statement

### Description

Continue the Linux fork by stabilizing the X11-first implementation path already on the branch. The immediate goal is not full Linux parity. The goal is to make the `window` crate compile cleanly with a coherent Linux facade and a minimal X11 backend skeleton.

### Impact

- `kaku-gui` cannot advance until `window` has a valid native backend boundary.
- The current branch already contains real work in `window/src/os/x11/*`, so re-planning from scratch would waste progress.
- The current errors are small enough to address deliberately, but broad enough that continuing without an explicit task list risks drifting back into over-scoped work.

### Success Criteria for This Iteration

1. `cargo check -p window` passes.
2. Linux routes through one clear ownership path.
3. X11 connection and window layers are coherent with pinned dependency APIs.
4. Keyboard and clipboard stop blocking compile progress.

## Current State Analysis

### Existing Implementation

| Component | Location | Current State |
|-----------|----------|---------------|
| Linux facade | `window/src/os/linux/window.rs` | Wraps X11 directly but uses incorrect geometry assumptions |
| X11 connection | `window/src/os/x11/connection.rs` | Mostly close to target shape, but stores unsized `Screen` and has one root visual mismatch |
| X11 window | `window/src/os/x11/window.rs` | Thin wrapper, missing trait support expected by Linux facade |
| X11 events | `window/src/os/x11/events.rs` | Mostly compileable; only integer conversion mismatches remain |
| X11 keyboard | `window/src/os/x11/keyboard.rs` | Present but should be treated as deferred unless it blocks compile |
| X11 clipboard | `window/src/os/x11/clipboard.rs` | Present but should be treated as deferred unless it blocks compile |
| OS exports | `window/src/os/mod.rs` | Ambiguous glob re-exports between `linux::*` and `x11::*` |

### Fresh Compiler Baseline

`cargo check -p window` on 2026-03-11 currently fails with **13 errors** plus **3 ambiguous glob re-export warnings**.

#### Error Buckets

| Bucket | Count | Notes |
|--------|-------|-------|
| X11 connection type issues | 3 | unsized `Screen`, borrowed `Screen`, `resource_id()` misuse |
| Linux facade geometry mismatch | 4 | uses nonexistent `Parameters.width/height` and wrong `RequestedWindowGeometry` field access |
| Missing trait support | 2 | `Debug` missing on `XWindow` and `XConnection` |
| X11 event numeric conversions | 4 | `u8` to `u32` for key/button details |

### Assumptions Explicitly Stated

| Assumption | Basis | Risk if Wrong | Verified? |
|------------|-------|---------------|-----------|
| Linux should continue delegating to X11 internally for now | Current branch already uses `os::linux::window` over `os::x11` | Medium: export strategy may need cleanup | Partially |
| Full keyboard/clipboard correctness is not required for this iteration | Current compiler baseline does not depend on finishing those subsystems first | Low | Yes |
| Matching WezTerm dependency versions remains mandatory | Confirmed in `lessons-learned.md` | High if ignored | Yes |

### Multiple Interpretations Considered

| Interpretation | Implications | Why Chosen/Rejected |
|----------------|--------------|---------------------|
| Continue full X11/Wayland plan from earlier design doc | Larger surface, slower feedback, more speculative code | Rejected for this iteration |
| Narrow to compile-first X11 slice | Faster recovery of build signal and clearer sequencing | Chosen |

## Vital Few

### Essential Constraints

| Constraint | Why It's Vital | Evidence |
|------------|----------------|----------|
| Keep pinned dependency versions | API drift was the original failure source | `lessons-learned.md` |
| Restore compileability before deepening runtime behavior | No meaningful integration progress without a green `window` crate | fresh compiler baseline |
| Choose one ownership path for Linux -> X11 | Current exports and wrappers create ambiguity | `window/src/os/mod.rs`, `window/src/os/linux/window.rs` |

### Eliminated from Scope

| Eliminated Item | Why Eliminated |
|-----------------|----------------|
| Full Wayland backend | Not needed to unblock the current crate |
| Full xkbcommon integration | Wrong level of work for the current blocker set |
| Full clipboard ownership/selection protocol | Can be deferred behind placeholders |
| Rendering/EGL/WebGPU completion | Depends on a stable native window boundary first |
| Feature parity with macOS | Not required for this milestone |

## Risks and Unknowns

### Known Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Fixing compile errors without reducing export ambiguity leads to repeated churn | High | Medium | Resolve module ownership path early |
| Linux geometry API is misunderstood and patched ad hoc | Medium | Medium | Read the existing types before editing |
| Deferred keyboard/clipboard code still leaks into the compile path | Medium | Low | Replace with explicit placeholders if necessary |

### Open Questions

1. Which Linux-facing constructors are actually used by higher layers: `new`, `create_terminal_window`, or `new_window`?
2. Should `window/src/os/mod.rs` stop re-exporting `x11::*` publicly on Linux once the facade is stable?

## Phase 2 Design: Compile-First Implementation Plan

### Summary

Stabilize the Linux/X11 slice already present on the branch. Fix only the mismatches blocking `window` compilation, then stop and take a new baseline before resuming deeper backend work.

### Approach

Use the simplest architecture that matches the current code:
- `os::linux` remains the public Linux platform facade.
- `os::x11` remains an internal implementation detail used by Linux.
- `connection.rs` owns connection primitives.
- `window.rs` owns a thin wrapper over a created X11 window.
- `events.rs` performs decoding only.
- keyboard and clipboard remain intentionally shallow until the window crate is green.

### Scope

**In Scope**
- Resolve the 13 current compiler errors in `window`
- Remove or reduce Linux/X11 export ambiguity
- Align Linux geometry handling with real types in the repository
- Defer nonessential X11 subsystems explicitly

**Out of Scope**
- Wayland
- IME
- drag and drop
- complete clipboard
- complete keyboard translation
- render-path enablement

**Avoid At All Cost**
- Expanding into full backend work before the crate compiles
- Updating `xcb` or `xkbcommon` again
- Treating all existing X11 files as equally urgent
- Adding abstractions for future multi-backend selection now

### Simplicity Check

The easy version is:
- one Linux facade,
- one compileable X11 connection type,
- one compileable X11 window type,
- event decoding with correct primitive conversions,
- placeholders where the branch is not ready for more.

If a change does not directly reduce the current error count or clarify module ownership, it is probably not part of this iteration.

## File Change Plan

### Likely Modified Files

| File | Planned Change |
|------|----------------|
| `window/src/os/mod.rs` | reduce ambiguous public re-exports |
| `window/src/os/linux/window.rs` | fix constructor geometry handling and trait expectations |
| `window/src/os/x11/connection.rs` | replace owned `Screen`, fix root visual storage, derive/debug support |
| `window/src/os/x11/window.rs` | add trait support expected by facade |
| `window/src/os/x11/events.rs` | fix numeric conversions |
| `window/src/os/x11/keyboard.rs` | only if needed, replace compile blockers with placeholder |
| `window/src/os/x11/clipboard.rs` | only if needed, replace compile blockers with placeholder |

### No New Files Required for This Iteration

This is intentionally a stabilization pass, not a feature expansion.

## Reviewable Task List

### Task Group A: Re-baseline and remove ambiguity

1. Confirm the Linux ownership path.
   - Keep `os::linux` as the platform entrypoint.
   - Treat `os::x11` as internal for now.

2. Remove ambiguous Linux/X11 glob export overlap in [`window/src/os/mod.rs`](/home/alex/infrastructure/Kaku/window/src/os/mod.rs).
   - Goal: one stable public surface on Linux.

3. Re-check which geometry types are used by constructors in [`window/src/os/linux/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/linux/window.rs).
   - Fix against real types, not assumptions from earlier stubs.

### Task Group B: Make connection and window layers coherent

4. Fix `XConnection` storage in [`window/src/os/x11/connection.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/connection.rs).
   - Stop storing `xcb::x::Screen` by value.
   - Store the minimal root/screen fields actually needed.
   - Add `Debug` support in a way that does not fight foreign types.

5. Fix root visual/window metadata handling in [`window/src/os/x11/connection.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/connection.rs).
   - Remove the invalid `resource_id()` call.

## Post-Review Follow-Up Plan

**Date**: 2026-03-11
**Source**: latest Linux review findings

### Scope

Address the remaining Linux regressions that are still blocking a reviewable backend:
- clipboard ownership persistence
- WM-driven activation/focus behavior
- integrated-chrome drag hooks

### Implementation Sequence

1. Fix clipboard ownership persistence in [`window/src/os/linux/clipboard.rs`](/home/alex/infrastructure/Kaku/window/src/os/linux/clipboard.rs).
   - Keep Linux clipboard objects alive for the lifetime of selection ownership instead of creating-and-dropping them per write.
   - Handle both `Clipboard` and `PrimarySelection`.
   - Verify copy/paste continues to work when there is no external clipboard manager.

2. Fix X11 activation behavior in [`window/src/os/x11/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/window.rs) and, if needed, [`window/src/os/x11/connection.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/connection.rs).
   - Prefer `_NET_ACTIVE_WINDOW` / WM activation flow.
   - Do not rely on direct `SetInputFocus` as the primary mechanism for restoring/focusing a toplevel window.
   - Preserve map/raise behavior only as WM-compatible supporting logic.

3. Implement Linux drag hooks in [`window/src/os/linux/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/linux/window.rs).
   - Wire `request_drag_move` and `set_window_drag_position`.
   - Use the appropriate X11/EWMH mechanism for client-side decorated window drags so integrated chrome can move the window and participate in WM snap/maximize affordances.

### Acceptance Criteria

1. Clipboard writes remain available long enough for pastes into external applications on Linux/X11/XWayland.
2. Refocus/activate requests succeed under EWMH-compliant window managers without depending on forbidden client-side focus stealing.
3. Integrated title/tab bar dragging works on Linux again.
4. `cargo check -p window`, `cargo check -p kaku-gui`, and a Linux smoke run remain green after the fixes.
   - Keep only metadata needed by the current branch.

6. Add minimal trait support in [`window/src/os/x11/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/window.rs).
   - Goal: satisfy Linux facade expectations without forcing inappropriate traits like `Copy`.

### Task Group C: Correct decoding and deferred subsystems

7. Fix integer conversions in [`window/src/os/x11/events.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/events.rs).
   - Explicit `u8` to `u32` conversions for key/button events.

8. If keyboard or clipboard enters the error set after the above changes, reduce them to compile-safe placeholders.
   - Do not deepen those implementations in this iteration.

### Task Group D: Verification gate

9. Run `cargo check -p window`.
   - This is the primary acceptance gate.

10. If green, run a broader follow-up compile.
   - `cargo check -p kaku-gui`
   - Record the next blocker list separately instead of expanding scope immediately.

## Implementation Order

### Step 1
**Goal**: eliminate public-surface ambiguity  
**Files**: [`window/src/os/mod.rs`](/home/alex/infrastructure/Kaku/window/src/os/mod.rs), [`window/src/os/linux/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/linux/window.rs)

### Step 2
**Goal**: make `connection.rs` compile with the pinned X11 API  
**Files**: [`window/src/os/x11/connection.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/connection.rs)

### Step 3
**Goal**: make the X11 window wrapper satisfy Linux facade needs  
**Files**: [`window/src/os/x11/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/window.rs)

### Step 4
**Goal**: clean remaining event conversion issues  
**Files**: [`window/src/os/x11/events.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/events.rs)

### Step 5
**Goal**: stabilize any deferred module compile leaks  
**Files**: [`window/src/os/x11/keyboard.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/keyboard.rs), [`window/src/os/x11/clipboard.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/clipboard.rs) only if needed

### Step 6
**Goal**: verify the crate and capture the next milestone  
**Checks**: `cargo check -p window`, then `cargo check -p kaku-gui`

## Acceptance Criteria

- `lessons-learned.md` is reflected in the plan.
- The older full-backend plan is intentionally narrowed for the current branch state.
- The next work is sequenced around the actual 2026-03-11 error baseline.
- The primary gate for this iteration is explicit: `cargo check -p window`.

## Recommended Immediate Next Batch

Start with:
1. export cleanup in [`window/src/os/mod.rs`](/home/alex/infrastructure/Kaku/window/src/os/mod.rs)
2. geometry fixes in [`window/src/os/linux/window.rs`](/home/alex/infrastructure/Kaku/window/src/os/linux/window.rs)
3. `Screen`/visual cleanup in [`window/src/os/x11/connection.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/connection.rs)
4. event conversion fixes in [`window/src/os/x11/events.rs`](/home/alex/infrastructure/Kaku/window/src/os/x11/events.rs)

That batch attacks the current error set directly without pulling keyboard, clipboard, Wayland, or rendering into scope.
