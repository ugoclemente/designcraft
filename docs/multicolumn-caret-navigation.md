# Caret navigation across text columns

Up/Down already chose the adjacent composed line in story order. It then called the
frame-wide pointer hit test with the old horizontal coordinate and the target baseline.
At a column boundary this selected a line in the original column, where the same baseline
also exists. The regression reproduced Down returning from bytes 243..272 to byte 2,
instead of entering the next column's line at bytes 272..299.

The composer now exposes its existing line hit test as `hit_line`; pointer hit testing
delegates to it after choosing a line. The engine's `text.move` uses that same function
directly on the already chosen target line. Across columns or threaded frames, x is
translated using the composed column origins. Within a column x remains unchanged,
including when paragraph indentation or text wrapping changes the line's available width.
Existing bidi and stacked-row hit testing are reused. No document format, preference,
keyboard mapping or UI behavior outside caret movement is changed.

## Verification

- The three-column command regression fails before the fix and passes afterward.
- Every adjacent line is checked in both directions, including both column boundaries,
  LTR and RTL story order, preserving horizontal offset, Shift selection and insertion
  immediately after a column transition.
- Threaded frames with different insets, first/last visible lines, empty and single-line
  multicolumn frames, and the existing horizontal/vertical navigation tests pass.
- UI regression clicks with the Type Tool, crosses a column boundary using actual key
  events, returns with Up, crosses again and types at the new insertion point.

Windows x64 checks (2026-10-15): composer 166 passed (one ignored), engine 301 passed,
three engine text-decoration integration tests passed, tools 41 passed, UI 185 passed
(one ignored). The previously documented Windows symlink-privilege test and three
font-menu environment failures were excluded; full CI is not claimed. Formatting,
workspace Clippy with warnings denied, asset attribution and dependency layers passed.
WASM checks passed with the existing unused-variable warning in `panels/library.rs`.

The previous native release also reproduced the defect through actual `ui.key` events:
Down returned from byte 237 to byte 0 at the first column boundary. The smoke test uses
a separate configuration directory and a synthetic three-column document.

The corrected Windows x64 release build passed the same native smoke test: 30 visible
lines through three columns in both directions, stopping at story edges, Shift selection,
and insertion at the destination. The resulting screenshot was inspected and the test
application exited normally. Release build succeeded; executable:
`C:/Users/Admin/Progetti/designcraft/target/release/designcraft.exe`.

## Changed files

- `crates/compose/src/lib.rs`: reuse the line hit test without choosing a line again.
- `crates/engine/src/cmd/text.rs`: column-relative caret movement and command regressions.
- `crates/ui-egui/src/canvas.rs`: Type Tool click/key/text input regression.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/multicolumn-caret-navigation.md`: behavior and validation.

## Try it

Open a text frame with two or more populated columns using the build in
`target/release/designcraft.exe`. With the Type Tool, click near the bottom of the first
column and press Down until the caret enters the top of the next column. Up returns to
the bottom of the previous column. Repeat at another horizontal position, hold Shift to
extend the selection, and type at the destination.

The official installation and user documents are unchanged.
