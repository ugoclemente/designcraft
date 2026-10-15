# Text Wrap offsets and selection outline

The document and `object.textWrap` already supported top/left/bottom/right offsets, but
the dedicated panel exposed a single offset field hard-coded to zero. Properties exposed
only the mode and Invert controls. Its ellipsis assigned the legacy `open_panel` state,
which could not reveal an existing floating panel or a customized docking layout.

Both surfaces now use the same four live measurement fields, reading the selected item's
actual offsets. The link button changes `UiState.wrapOffsetsLinked` through the UI command
`window.textWrapLink {on?: bool}`. Unlinked edits change the chosen side; linked edits set
all four offsets to the edited value. Linking itself does not overwrite imported asymmetric
values. The initial mode is unlinked and is saved with the existing application UI settings.
Document edits continue to use `object.textWrap`, so undo/redo and document persistence
remain in the engine. Fields accept the document's measurement units and are disabled for
No Wrap, no object selection, or a locked selection.

The ellipsis now uses `window.panel.activate {panel: textWrap}`, revealing the panel through
the existing docking system without duplicating an already floating/customized panel.

`Item::text_wrap_bounds` shares the existing axis-aligned exclusion geometry between the
composer and selection display. It uses the same top/left/bottom/right expansion and skips
nonfinite or inverted bounds. The selected frame shows a non-printing outline in its layer
color. Offset edits and undo update it immediately; deselecting removes it. This does not add
artwork to the document or to exports. Existing wrap modes and their composition semantics
are retained; this change does not implement new contour detection or wrap-side algorithms.

## Verification (Windows x64, 2026-10-15)

- UI: actual spinner edits on every side in both Properties and the dedicated panel;
  linking from all four fields; undo/redo; UI-state serialization; No Wrap disables edits.
- The chain command distinguishes an explicit true/false assignment from a toggle, is
  idempotent when assigning the same state again, and rejects non-boolean state values.
- More-options clicks reveal the panel in legacy and customized/floating layouts,
  preserving unique panel ownership.
- Pixel regression: selected wrap outline at the actual offset, moving the top margin,
  undo restoring the old outline, and deselection removing it.
- Engine: asymmetric offsets invalidate the composition cache, produce the expected
  exclusion rectangle and move text, survive native document serialization, and undo/redo.
- Document geometry: asymmetric and negative offsets, disabled wrapping, and invalid bounds.
- Suites: document 70 passed; composer 166 passed (one ignored); engine 302 passed plus
  three text-decoration integration tests; UI 191 passed (one ignored). The previously
  documented Windows symlink-privilege test and three font-environment tests were excluded.
  Full unfiltered CI is not claimed.

- Formatting, workspace Clippy with warnings denied, asset attribution and dependency
  layering passed. WASM checks passed for all 15 selected crates, with the existing
  unused-variable warning in `panels/library.rs`.
- Windows x64 release build passed. Native trials with a placed synthetic SVG and surrounding
  text verified asymmetric 3/2/6/4 mm offsets, real spinner edits on individual sides, linking
  and unlinking through the chain button, undo/redo and actual saved values. Reopening the
  document retained the offsets; restarting the app retained the chain preference.
- A real click on Properties' ellipsis restored the closed floating Text Wrap panel exactly
  once. Italian screenshots showed the shared values in both panels and the external outline
  around the image. Screenshots were inspected; the test application closed normally.
  Tests used a separate profile and synthetic documents under `target/`.
- Executable: `C:/Users/Admin/Progetti/designcraft/target/release/designcraft.exe`.
  Inspected screenshot: `target/text-wrap-verified.png`.

## Changed files for this correction

- `crates/doc/src/item.rs`: shared wrap exclusion bounds and geometry tests.
- `crates/compose/src/lib.rs`: use the shared exclusion geometry.
- `crates/engine/src/tests.rs`: actual reflow, persistence and undo/redo regression.
- `crates/ui-egui/src/panels/mod.rs`: expose selected offsets to the UI.
- `crates/ui-egui/src/panels/properties.rs`: shared four-field editor, chain, Invert in the
  dedicated panel, working ellipsis and UI regressions.
- `crates/ui-egui/src/canvas.rs`: selected wrap outline and pixel regression.
- `crates/ui-egui/src/lib.rs`, `crates/ui-egui/src/menus.rs`: persisted chain mode and UI command.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/text-wrap-offsets.md`: behavior and validation.

## Try it

In `target/release/designcraft.exe`, select an image frame and choose Wrap Around Bounding
Box in Text Wrap. Enter different top, bottom, left and right offsets; surrounding text and
the external rectangle update. Activate the chain and edit any field to give all four the
same value. Unlink to edit one side again. Try undo/redo, deselect/reselect, save/reopen and
the Properties ellipsis. The original installation and user documents are unchanged.
