# Frame-edge occlusion

Unselected text/image frame edges were all painted by the UI after the whole document
texture. This showed lower frames' contours over opaque foreground objects, regardless of
stacking order. The former Windows release reproduced the failure: a covered edge still had
blue channel 157 inside a red foreground image.

Screen frame edges now draw immediately after their item, through the existing renderer's
layer/item/group ordering and clipping. Later opaque content covers them; translucent or
transparent areas allow what is below to show through. This uses actual content compositing,
not bounding-box guesses about an object's opacity. An unfilled frame remains transparent
between its glyphs or artwork even when its object opacity is 100%.

Selection chrome stays a separate foreground UI pass, so the whole selected frame contour
and its handles remain visible. Frame-edge display is still controlled by the existing View
command. Parent edges retain their dotted style, empty graphic frames their X, and tagged
frames their tag-colored edges; these decorations follow the same item order.

`RenderOptions.frame_edges` is optional, frontend-independent and disabled by default.
Printing mode rejects it even if supplied. Exports retain document strokes and artwork but
do not gain non-printing frame edges. The document model, file format, tools and selection
commands are unchanged. Edges use device-space strokes so zoom/item transforms do not enlarge
the hairline. The viewport cache records edge/tag visibility and partial repaints retain it.

## Verification (Windows x64, 2026-10-14)

- Renderer pixel regression: opaque overlap hides the covered segments, exposed segments
  stay visible, translucent cover shows underlying contours, groups and different layers
  respect order, and printing output is byte-for-byte unchanged with edges requested.
- UI pixel regression: text/graphic frames under a placed image, selecting/deselecting the
  lower frame, bringing it forward/back, partial repaints and live Show/Hide Frame Edges.
- Engine: 298 passed; renderer: 21 passed; tools: 41 passed; UI: 184 passed, one ignored.
- Four previously failing environment-specific tests remain excluded (Windows symlink
  privilege and three font-menu expectations). Full CI is not claimed.
- The first parallel UI run ended in Windows STATUS_HEAP_CORRUPTION; the isolated contour
  test passed. Offscreen pixel-test device lifetimes are now guarded through harness teardown.
  The complete UI suite then passed with its normal parallel execution. This guard is test-only.
- Formatting, workspace Clippy with warnings denied, WASM (15 crates), assets (58 files) and
  layers (20 crates) passed. The existing WASM warning in library.rs remains.
- Windows release build passed. Native trials passed for actual text and placed-image
  frames: opaque occlusion, selected full contour, deselection, partial opacity, changing
  stacking order and live edge visibility toggles. Hidden/selected screenshots were inspected;
  the test application closed normally. Executable: `target/release/designcraft.exe`.

## Changed files for this correction

- `crates/render/src/lib.rs`: optional screen-edge pass and compositing/export regression.
- `crates/ui-egui/src/canvas.rs`: remove the universal frame-edge overlay, pass rendering
  options, cache invalidation, partial repaint padding and actual UI pixel regression.
- `crates/ui-egui/src/lib.rs`: frame-edge/tag mode in the texture cache key.
- `ROADMAP.md`, `docs/ui-parity.md`, `docs/grids-in-back.md`, `docs/guides-in-back.md`,
  `docs/frame-edge-occlusion.md`: behavior and verification.

## Try it

Launch `target/release/designcraft.exe` from the checkout. Put an opaque image above a text
or image frame and deselect everything: the covered portion of the lower edge must disappear,
while its exposed portion stays visible. Select the lower frame: its full outline must appear.
Then deselect, change object order and try partial opacity. No new checkbox is required.

The official installation is unchanged. No user documents were modified and no commit or
push was made. Native trials use a separate profile and synthetic artwork.
