# Icon sources

Master artwork for ImAlive's icons. Edit the SVGs here, then regenerate.

- `app-icon.svg` — full-color app icon (green squircle + heartbeat trace).
- `menu-icon.svg` — monochrome **template** glyph for the macOS menu bar.

## Regenerate the app icons

Generates the full platform set into `../src-tauri/icons/` (icns, ico, PNGs,
Windows Square logos, iOS/Android assets). Tauri rasterizes the SVG with resvg.

```bash
pnpm tauri icon design/app-icon.svg
```

## Regenerate the menu-bar (tray) icon

Tauri's `icon` command only emits the full set, so render to a temp dir and
copy the 32px PNG into place. `src-tauri/src/lib.rs` embeds it via
`include_image!("icons/tray.png")` with `.icon_as_template(true)`.

```bash
pnpm tauri icon design/menu-icon.svg -o /tmp/imalive-menu
cp /tmp/imalive-menu/32x32.png src-tauri/icons/tray.png
```
