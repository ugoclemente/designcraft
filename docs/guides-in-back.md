# Guide sotto / Guides in Back

Edit › Preferences › Guides & Pasteboard now has **Guide sotto** in Italian, **Guides in Back**
in English and as the fallback in untranslated languages. It controls margins, column
boundaries and ruler guides. When enabled they are drawn behind document content: opaque
objects and placed images cover them, while blank paper and transparent areas still show them.
When disabled they remain foreground guides.

This is independent of Preferences › Grids › Grids in Back. All four combinations are
supported. Selection outlines, handles, text carets and interactive guide-drag feedback remain above
the document. [Unselected frame edges](frame-edge-occlusion.md) follow document stacking.
Existing snapping and guide dragging are unchanged; objects already have
precedence over ruler guides in hit testing.

The new application preference `prefs.set {"guidesInBack": true}` is off by default to retain
the old appearance. Old `prefs.json` files without the field load as false. The existing
application-preference save/load path persists the setting across restarts. It is not a
second grid preference or a document-content change.

The UI reuses the transparent content texture and paper pass from the grid fix. Each kind
of decoration chooses its own foreground/background pass. The cache records whether either
background pass needs a transparent texture; mode transitions trigger complete repainting,
and partial repainting retains that mode. Exported documents do not contain guides or grids.

## Verification (Windows x64, 2026-10-14)

- Pixel regression: margins, column boundaries and ruler guides under opaque content; guides
  remain visible on blank paper; all four combinations with the baseline-grid preference.
- Italian dialog: click Guide sotto, confirm, reopen and check the stored value.
- Engine: old-file default, JSON round trip and invalid-value rejection.
- Engine: 298 passed; tools: 41 passed; UI: 183 passed, one ignored. Existing text/image
  double-click regressions and background-grid regressions still pass.
- Four previously failing environment-specific tests remain excluded: Windows symlink
  privilege and three font-menu expectations. No full CI claim is made.
- Formatting, workspace Clippy with warnings denied, WASM, asset and dependency-layer checks
  passed. The existing WASM warning in library.rs remains.
- Windows release build passed. Native screenshot/pixel tests passed for all four independent
  grid/guide combinations, including margin/column lines, ruler guides and a placed SVG.
  A ruler guide crossing actual text glyphs was covered by the glyphs with the option enabled.
  The preference survived a normal exit/restart. Screenshots were visually inspected.
  Testing used a separate profile and synthetic documents, then closed the test app normally.

## Files for the guide change

- `crates/engine/src/lib.rs`, `crates/engine/src/cmd/prefs.rs`: preference, command parameters,
  backward-compatible default and persistence regression.
- `crates/ui-egui/src/dialogs.rs`: checkbox, command application and Italian dialog regression.
- `crates/ui-egui/src/i18n.rs`, `crates/ui-egui/src/i18n/it.rs`: label, Italian and fallback.
- `crates/ui-egui/src/canvas.rs`, `crates/ui-egui/src/lib.rs`: independent passes, shared cache
  mode and pixel regression.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/grids-in-back.md`, `docs/guides-in-back.md`: docs.

## Try it

Launch `target/release/designcraft.exe` from this checkout. Open Edit › Preferences ›
Guides & Pasteboard and enable **Guide sotto**, then confirm. It applies immediately, with
no document reopen required. Toggle Grids in Back separately to choose the grid plane.
The official installation and user documents are unchanged; no commit or push is made.
