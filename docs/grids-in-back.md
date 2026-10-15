# Grids in Back

The existing Italian checkbox **Griglie sullo sfondo** in Edit › Preferences › Grids already
saved `document.settings.grid.in_back`, but the canvas did not read it. The currently drawn
baseline grid was always an overlay on top of the document texture.

The UI now draws the baseline grid separately from guides. With the option enabled and the
grid visible, it draws paper, then the grid, then the document content, using the renderer's
existing `paper: false` option for a transparent content texture. With the option disabled,
the existing paper-filled texture is followed by the foreground grid. The independent
[Guide sotto](guides-in-back.md) option controls the guide plane; selected outlines and handles
remain overlays, while [unselected frame edges](frame-edge-occlusion.md) obey stacking order.
Preview and presentation keep grids hidden. The
renderer and exported documents do not gain any grid decoration.

The texture cache records the background-grid mode. A mode change forces a complete repaint;
small edits keep the same paper mode in their partial texture repaints. This also covers
show/hide and baseline-grid zoom-threshold transitions. The preference remains document
settings, using the existing command, serialization and undo rather than a second preference.

## Verification (Windows x64, 2026-10-14)

- Reproduced the previous release: the pixel on a grid line over an opaque object has green
  channel 192 with either checkbox value. The background setting had no visual effect.
- New pixel regression covers foreground/background ordering over an opaque object, visible
  grid on blank paper, live toggling, partial repaints, grid hiding and preview mode.
- An actual Italian-dialog click updates the preference and reopening the dialog preserves it.
- Engine regression covers both values through document save/reopen and preference undo.
- Native release passed: grid-line green over the filled object changes from 206 (front)
  to 11 (back) and returns to 206 when toggled again; over a placed SVG it changes from 204
  to 0. Foreground/background screenshots were inspected. The test app used a separate
  profile and closed normally.
- Engine: 297 passed; tools: 41 passed; UI: 181 passed, one ignored.
- Four previously failing environment-specific tests were excluded: the Windows directory
  symlink privilege test and three font-menu expectations (`Source Sans 3` / `Source Sans Pro`).
  Full `cargo xtask ci` was not rerun; its components checked here were formatting, workspace
  Clippy with warnings denied, pertinent tests, assets (58 attributed files), layers (20 crates)
  and WASM (15 crates). All passed. The existing WASM warning in `panels/library.rs` remains.

## Files changed

- `crates/ui-egui/src/canvas.rs`: grid pass, paper/content composition, cache invalidation,
  partial repaint mode, pixel regression.
- `crates/ui-egui/src/lib.rs`: texture cache mode key.
- `crates/ui-egui/src/dialogs.rs`: Italian checkbox regression.
- `crates/engine/src/cmd/prefs.rs`: existing preference persistence/undo regression.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/grids-in-back.md`: behavior and verification.

## Trying the build

The Windows executable is `target/release/designcraft.exe`. In Git Bash, from this checkout:

```bash
./target/release/designcraft.exe
```

Show the baseline grid, then toggle Edit › Preferences › Grids › Grids in Back. It should
disappear under an opaque filled object or placed image when enabled, and reappear over the
object when disabled. On blank paper it stays visible in either mode. Save the document to
retain the option with it. The official installation is unchanged; no commit or push is made.
