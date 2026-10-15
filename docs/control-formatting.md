# Control panel formatting selectors and icons

The A and paragraph marks in the text Control panel were labels with no click behavior.
They are now selected-state icon buttons. A puts character controls first; the paragraph
button puts paragraph controls first. Both groups remain available in the horizontally
scrollable bar. Existing controls and formatting commands are reused without duplication.

The UI command `window.controlFormatting {mode: character|paragraph}` changes only the
persisted `UiState.controlParagraph` value. Character mode is the backward-compatible
default. Invalid or missing modes fail without changing state; switching modes preserves
the text selection and does not edit the document. Both selectors use existing translations
and remain accessible through the control protocol.

Font size and leading now use the original `font-size` and `leading` icons already used
in Properties. This replaces the Unicode mathematical T (which could appear as a missing
glyph) and A-with-macron captions. The captions have localized tooltips and accessibility
labels: Italian Corpo and Interlinea. No external or Adobe artwork is introduced.

## Verification (Windows x64, 2026-10-15)

- Real UI clicks switch in both directions at widths 700 and 1440, in English and Italian.
- Group ordering changes, selected text remains unchanged, both icon captions fit within
  the two-row bar, and returning to character mode restores the original field positions.
- UI command validation, serialization, restoration, and old UI settings without the new
  field are covered by regression tests.
- UI suite: 187 passed, one ignored, three previously documented font-environment tests
  excluded. This includes the earlier multicolumn caret, grid/guide order and frame-edge tests.

- Formatting, workspace Clippy with warnings denied, asset attribution and dependency
  layering passed. WASM checks passed for all 15 selected crates, with the existing
  unused-variable warning in `panels/library.rs`. Full unfiltered CI is not claimed.
- Windows x64 release build passed. Native `ui.click` events on the A/paragraph buttons
  switched modes in both directions without changing the text selection. Italian
  screenshots of both modes were inspected: font-size and leading match Properties.
  Paragraph mode was restored after closing and restarting the test application.
  Tests used a separate profile and synthetic document, and closed the app normally.
  Executable: `C:/Users/Admin/Progetti/designcraft/target/release/designcraft.exe`.

## Changed files for this correction

- `crates/ui-egui/src/chrome.rs`: clickable selectors, group ordering, shared icons,
  accessibility captions and regressions.
- `crates/ui-egui/src/lib.rs`: persisted formatting mode and default.
- `crates/ui-egui/src/menus.rs`: validated, automatable UI command.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/control-formatting.md`: behavior and validation.

## Try it

Open `target/release/designcraft.exe`, enable Window > Control and place the caret in a
text frame with the Type Tool. Click the paragraph button beneath A: paragraph controls
move to the left. Click A to return to character controls. Font size has the small/large
T icon; leading has two stacked A letters and a vertical double arrow, matching Properties.
Hover the icons for localized descriptions. The chosen mode is restored after restart.

The official installation and user documents are unchanged.
