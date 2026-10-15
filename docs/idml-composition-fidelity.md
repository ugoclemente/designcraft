# IDML composition fidelity: newspaper import

Date: 2026-10-15. Comparison against InDesign CS6 on Windows through its public scripting interface, without reading application internals or modifying source documents. User documents and diagnostic outputs remain outside version control.

## Observations and changes

- INDD and IDML report a 315 × 440 mm page and a 10 pt baseline grid starting at the page top. Imported headline frame dimensions match IDML exactly. Changing document geometry is not the remedy.
- Times New Roman in InDesign reports 1420/2048 em ascent and 442/2048 em descent (Italic ascent 1422). The previous font path selected 1825/2048 and 443/2048 clipping extents. Font faces now prefer valid OS/2 typographic extents, retaining MVAR variation deltas and falling back when unavailable. This also fixes caret/selection extents consistently.
- Text frames admit the last baseline on the bottom edge, as observed in CS6. Descenders still reserve ink clearance above footnotes. Table cell height logic is unchanged.
- Text wrap previously pushed snapped baselines downward by 1 pt, producing a first baseline between grid lines and an irregular gap to the next. On-grid lines now advance by the grid increment while searching for free space.
- Italian was recognized as a text language but its hyphenator was not compiled. Enable the existing hypher Italian feature, MIT-licensed Beccari patterns, including elisions without breaks beside apostrophes. Existing per-paragraph limits and disabled hyphenation still apply. Text content is never rewritten.
- Optical kerning was serialized but composed as Metrics. Latin runs now derive spacing from facing glyph contours and the font's own `nn` rhythm, with bounded caches, bounded input geometry and sampled collision clearance. Spaces, marks and joining scripts retain shaping. This original algorithm is not Adobe's optical algorithm. The observed main headline now measures 778.461 pt within its 779.528 pt frame; all previously truncated headings fit.
- Tracking and manual kerning scale with horizontal glyph scale. AkiAbove/AkiBelow em-edge leading references no longer offset purely Latin lines unless their language requests CJK composition.
- The paragraph composer previously merged paths of different line counts, even where later lines meet different widths around text wrap. Retain these states separately with bounded work/memory, and do not prune a viable path merely because an unused discretionary hyphen overruns. On overset, retry shorter total-fit alternatives and accept them only if they place more source text. Font size, frame geometry, paragraph attributes, hyphenation settings and text bytes remain unchanged. Genuine overset still reports normally.
- Embedded WMF-tagged graphics in the observed IDML contain EMF bytes. Detect their actual MIME rather than naming them PNG. The image layer decodes a bounded bitmap-only EMF fallback: complete SRCCOPY or binary-transparent SRCPAINT/SRCAND pairs. Unknown drawing/transform records, unsupported raster operations, malformed offsets, truncated streams and oversized bitmaps are rejected, not partially displayed. This is not general vector WMF/EMF support. Original asset bytes are retained; native exports use .emf for recognized assets. Existing native documents with mislabeled image/png bytes also render by signature.
- Customized floating panels previously exposed closing only in the tab context menu and resizing in an unmarked outer corner. Visible close and resize controls now use the existing window.panel commands; geometry is persisted and restored. New Text Wrap floating windows start at 220 pt UI height instead of 400. Stored user sizes are respected. This panel belongs to the application workspace, not IDML document settings.
- Every customized dock group now has a close button in its tab strip, including docked panels. Utility icons remain available whether the panel is visible or hidden. Clicking toggles the active panel; first opening a utility uses a floating window instead of appending another main-dock tab. Reopening restores its placement. The default dock also has a visible close button and moves into the existing customizable layout when it is used.
- Horizontal and vertical canvas scrollbars share the existing view through the `view.pan` UI command. Thumb size represents the visible extent, clicks page the view, dragging preserves the grab point, and focused keyboard navigation reaches the limits. Bounds derive from the actual pasteboard and view rotation; split panes have separate widget IDs. Presentation hides the bars. Canvas tool keys retain focus, including the first arrow press after a click.

## Verification

Synthetic regressions cover font metric selection, baseline fit and real overset, text wrapping on the grid, Italian syllables/elisions and composition, embedded EMF MIME, bitmap color/transparency and hostile records, floating panel pointer resizing/closing/reopening and serialization. Existing CJK, Arabic, tables, footnotes, selection and editing tests are rerun. Real-page comparisons and final command results are recorded locally during verification.

## Remaining fidelity limits

All 26 stories on the supplied newspaper page now fit, compared with nine residual overset stories before this follow-up. Body line counts match the observed CS6 page: 96, 47, 35, 59 and 42 for the five article stories. Identical document settings still do not guarantee pixel-identical layout between different composition engines: individual glyph spacing, paragraph break choices and hyphenation dictionaries differ. Broader optical spacing, especially mixed-style boundaries and non-Latin scripts, needs separate coverage. No font compression, frame growth, hidden text deletion or document-specific corrections are used to mask differences. This is a fix for the reported page, not a blanket claim of print-production parity.

The decoder handles complete bitmap fallbacks, not arbitrary EMF vector drawings. Unsupported graphic formats still need better import/preflight diagnostics. Full first-baseline cap/x-height and per-frame grid reference parity also require separate coverage.

Original INDD, IDML and DesignCraft files remain unchanged. No automatic commit or GitHub push is part of this task.

Implementation references: [Microsoft MS-EMF record specification](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/89c0d808-0dea-413f-be40-2e9e51fa36ac) and [Italian pattern attribution and MIT licence](https://github.com/hyphenation/tex-hyphen/blob/master/hyph-utf8/tex/generic/hyph-utf8/patterns/tex/hyph-it.tex).

Measured on the supplied page after the corrections: first article column starts at page baselines 340, 350, 360, 370 pt; the first column below the image has 540, 550, 560, 570, 580 pt, matching the CS6 baseline positions. All eight placed assets decode (three JPEGs and five embedded EMF bitmap fallbacks). These results do not imply identical paragraph break positions.

## Final checks (Windows x64)

- Final follow-up workspace test run: 1,225 passed, zero failures, three ignored tests; four environment-dependent tests filtered explicitly, across 47 suites. New optical, paragraph-fit, scroll thumb/keyboard, rotation, split-pane IDs, persistent utility icons and close/reopen tests pass. The existing multicolumn caret navigation and immediate typing regression also passes.
- Filters: package_refuses_windows_directory_redirects (Windows error 1314: no symlink privilege); typing_in_the_font_menu_searches_and_never_reaches_the_story, font_fields_in_panels_and_dialogs_search, every_font_field_searches_in_every_workspace (pre-existing installed-font naming differences).
- cargo xtask ci: formatting and Clippy pass, then the unfiltered engine suite stops at the Windows symlink test (302 passed, one failed). The complete CI command is not green.
- Formatting, final workspace Clippy with -D warnings, asset attribution (58 assets), layering (20 crates) and WASM checks (15 libraries) pass separately. The existing WASM unused-path warning in panels/library.rs remains.
- The floating-panel regression exposed during development was fixed; all final UI tests in the filtered workspace run pass, including close, resize, reopening and persistence.

## Changed files

- Typography and composition: crates/fonts/src/fontdb.rs, crates/fonts/src/lib.rs; crates/compose/Cargo.toml, crates/compose/src/lib.rs, crates/compose/src/shape.rs, crates/compose/src/optical.rs, crates/compose/src/breaker.rs, crates/compose/src/tests.rs, crates/compose/src/hyphen/mod.rs.
- Embedded graphics: crates/images/src/emf.rs, crates/images/src/lib.rs; crates/idml/src/lib.rs, crates/idml/src/tests/assets.rs; crates/format/src/lib.rs.
- Navigation and panels: crates/ui-egui/src/canvas.rs, crates/ui-egui/src/canvas/scroll.rs, crates/ui-egui/src/dock.rs, crates/ui-egui/src/menus.rs, crates/ui-egui/src/panel_docking.rs, crates/ui-egui/src/panel_docking_tests.rs.
- Attribution and status: ASSETS.md, NOTICE, ROADMAP.md, docs/gaps.md, docs/target-app-parity.md, docs/typography-parity.md and this report.

## Release and native verification

Final Windows x64 release: C:/Users/Admin/Progetti/designcraft/target/release/designcraft.exe, 61,583,360 bytes, built 2026-10-15 at 18:19 local time with cargo build --offline --release -p designcraft. The official installation is unchanged.

Final native smoke test with an isolated profile passes: original IDML opens with all 26 stories fitting and no font substitution; horizontal and vertical scrollbar clicks move the actual view; the compact Italian Text Wrap panel closes and reopens at the same placement; opening another utility produces a floating panel. Saving a separate native copy and reopening it preserves the 26 story summaries exactly. Opening the user's original .designcraft also reports zero overset. SHA-256 hashes of the original INDD, IDML and DesignCraft files are unchanged. The page rendering and interface screenshot were visually inspected at target/idml-analysis/navigation-page.png and navigation-ui.png. The test application was closed normally. Its saved workspace remains in target/idml-analysis/profile-navigation, separate from the user's preferences.

The earlier nine-story residual overset result is superseded by the follow-up: zero overset in the supplied page's 26 stories.
