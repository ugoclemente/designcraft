# Frame double-click: text editing and image-content selection

Status: implemented; relevant tests and native Windows x64 release trial passed (2026-10-12).

With Selection active, a single click selects a frame. A primary-button double-click inside
an unlocked text frame activates Type and places a collapsed caret using `text.placeCaret` and
the existing composer hit testing. Typing inserts at that caret. Graphic frames do not activate
Type. Guide gestures retain their existing input handling.

A double-click inside a frame containing a placed graphic (raster image, SVG or PDF) now
selects that frame's content through the existing `selection.set` and `select.content`
commands. Selection stays active; orange handles now belong to the graphic bounds, while
the crop frame remains outlined in its layer color.
An empty graphic frame has no content to select. The text preference gates only text
frames, so disabling it does not block image-content selection.

The initial image implementation only set the content-selection flag and drew an additional
outline. It left the frame handles active, and `transform.move` still moved the entire frame.
The user's failed dragging trial was reproduced in the previous Windows release. Movement,
handle resizing and rotation now transform the selected graphic in its container space;
the frame path and transform stay fixed. Existing preview/commit and undo are reused. Content
drags stay in the frame's spread and retain their selected child when the frame is grouped.
Empty and text frames retain their existing Direct Selection movement. Singular container
transforms produce a command error rather than invalid coordinates.

The canvas dispatches double-clicks on release, after completing the current pointer gesture.
It then refreshes `wants_text` before processing keyboard input. Previously the canvas checked
egui's release-time double-click flag inside the press branch, so ordinary double-clicks never
reached the Selection tool's existing editing action.

Preferences › Type contains **Double-click a frame to edit text**, enabled by default. The
Italian label is **Doppio clic sulla cornice per modificare il testo**; other untranslated
languages use the existing English fallback. `prefs.set {"doubleClickTextEdit": false}` disables
automatic entry from Selection/Direct Selection without disabling manual Type editing or
Type-tool word selection. The existing native `prefs.json` save/load path persists the option;
older preferences without the key default to enabled.

## Verification

Windows results (Rust 1.95.0, x86_64 MSVC):

- Document: 69 passed.
- Engine: 296 passed; one packaging test excluded because Windows refuses its symbolic link
  creation without the required privilege (OS error 1314).
- Tools: 41 passed.
- UI: 179 passed, one ignored, three font-menu tests excluded. Their expectations use
  `Source Sans 3`, while this Windows environment resolves the family as `Source Sans Pro`.
- The new double-click, immediate-input, preference and localization regressions passed.
- Replacing only `handle_input` with its pre-change implementation makes the new mouse-event
  regression fail (Selection remains active). Restoring the fix makes it pass. The same three
  font-menu failures also occur with that original handler; they are independent of this fix.
- Native release: single-click selection, double-click Type activation, actual keyboard input,
  empty-frame editing, graphic/guide double-clicks, and preference persistence/disabled behavior
  after normal exit and restart passed using an independent profile. The window screenshot was
  inspected. The test profile was left with the option enabled.
- Workspace formatting and Clippy with warnings denied passed. Asset and layer checks passed.
  `cargo xtask ci` stops at the packaging test above, so the complete CI gate is not green.
- Release executable: `target/release/designcraft.exe`.
- WASM checks passed for all 15 selected crates. An existing unused-variable warning in
  `panels/library.rs` remains outside this change.

- UI regression: separate mouse presses/releases, normal single selection, double-click entry,
  immediate text on the second release, populated and empty text frames, graphic frames and
  disabled preference.
- Engine regression: several hit-test positions, collapsed caret, immediate insertion, undo,
  disabled shortcut and explicit Type editing.
- Image regression: actual placed SVG content, single-click frame selection, double-click
  content selection, unchanged document and undo history, return to container selection,
  enabled/disabled text preferences, and actual UI dragging (including grouped frames).
  Engine regressions cover move/resize/rotate with undo, transformed containers and singular
  transforms; ordinary frame movement after returning to container selection is preserved.
  Native Windows release also passed with an image cropped inside a frame; the screenshot
  shows orange content handles and the graphic shifted beyond the unchanged blue crop frame.
  The former build fails the same native dragging assertion (it moves the frame). Text trials
  were repeated and passed.
  The definitive release was retested with normal mouse dragging (no keyboard modifiers),
  both ungrouped and grouped images and both text-preference values: all passed. The final
  release executable is 61,399,040 bytes, built on 2026-10-12 at 16:53 local time.
- Preference regressions: old-file default, JSON round trip, invalid value rejection, Italian
  checkbox rendering/application and English fallback.

Required commands (run from the repository with Rust 1.95+ and Windows MSVC build tools):

```powershell
cargo fmt --all -- --check
cargo test -p designcraft-tools -p designcraft-engine -p designcraft-ui-egui --lib
cargo xtask ci
cargo build --release -p designcraft
```

The locally installed toolchain is isolated under `target/rust-tools`. To use it in a new
PowerShell session:

```powershell
$env:CARGO_HOME = 'C:/Users/Admin/Progetti/designcraft/target/rust-tools/cargo'
$env:RUSTUP_HOME = 'C:/Users/Admin/Progetti/designcraft/target/rust-tools/rustup'
$env:PATH = "$env:CARGO_HOME/bin;$env:PATH"
```

## Isolated Windows trial

After a successful release build, the executable is `target/release/designcraft.exe`. Launch
it with an independent profile to keep the installed application's settings and recovery files
separate:

```powershell
$env:DESIGNCRAFT_CONFIG_DIR = 'C:/Users/Admin/Progetti/designcraft/target/text-edit-profile'
& 'C:/Users/Admin/Progetti/designcraft/target/release/designcraft.exe' --sample
```

Select Pointer/Selection and click a text frame once, then double-click inside its text and type.
Repeat at another location and in an empty text frame. Double-click a graphic frame. Disable
the checkbox in Preferences › Type and repeat; Selection should remain active. Close normally,
restart with the same profile and verify that the checkbox remains disabled. Re-enable it for
normal use.

For an image, place it inside a frame, select Pointer/Selection and double-click inside.
The image's handles should be orange. Drag inside the image: the crop frame must stay fixed
while the image moves. Use Object › Select › Container to return to moving the frame.

## Modified files (text and image work)

- `crates/doc/src/item.rs`: shared unclipped graphic bounds.
- `crates/tools/src/lib.rs`, `crates/tools/src/select.rs`: selection bounds, double-click
  actions and preservation of selected image content during grouped-frame dragging.
- `crates/engine/src/cmd/object.rs`: transform the selected graphic inside its container.
- `crates/engine/src/lib.rs`, `crates/engine/src/cmd/prefs.rs`, `crates/engine/src/tooling.rs`:
  persistent text preference and text-only gating.
- `crates/engine/src/tests.rs`: text and graphic-content behavior regressions.
- `crates/ui-egui/src/canvas.rs`: double-click dispatch, immediate typing, content handles
  and real mouse-event regressions.
- `crates/ui-egui/src/dialogs.rs`, `crates/ui-egui/src/i18n.rs`,
  `crates/ui-egui/src/i18n/it.rs`: preference checkbox, Italian translation and fallback.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/double-click-text-edit.md`: delivery and verification.
