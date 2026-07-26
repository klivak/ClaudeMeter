# ClaudeMeter promo video

Remotion project generating two cuts of the ClaudeMeter promo from one set of scene components: a 16:9 long cut (`PromoEN`) and a 9:16 Shorts cut (`ShortsEN`). Both are English-only masters (see notes below) rendered at 1080p and 4K, plus three A/B thumbnail stills.

Every screen shown — the dashboard popup, tooltip, notification toast, settings list, Task Manager row — is a **vector JSX recreation** (`src/components/Mock.tsx`) rather than a rasterized screenshot. The source screenshots in `screenshots/` are only ~380px wide; upscaled 5x for a 4K render they turn to mush, while text and shapes drawn as DOM/SVG stay pixel-crisp at any render scale. Colours, copy, layout and numbers are all taken 1:1 from the real screenshots (Catppuccin Mocha dark palette, the red/amber/green threshold scheme, the teal Codex accent, the exact same percentages and reset times) — nothing invented, just redrawn instead of photographed.

## Commands

```bash
npm install
npm run studio            # live preview at localhost:3000 (do not run as a blocking check)

npx remotion compositions  # list PromoEN / ShortsEN / ThumbnailA-C
npx tsc --noEmit           # typecheck

npm run render:4k          # out/claudemeter-promo-4k.mp4            — 3840x2160  (DELIVERABLE)
npm run render:shorts:4k   # out/claudemeter-shorts-2160x3840.mp4    — 2160x3840  (DELIVERABLE)
npm run render:all         # both 4K renders + all three thumbnail stills

npm run render             # 1080p preview of the long cut (optional, not a deliverable)
npm run render:shorts      # 1080p preview of the Shorts cut (optional, not a deliverable)
npm run still:thumb        # out/thumbnail-{a,b,c}.png
```

## Storyboard — 16:9 long cut (`PromoEN`, 1920x1080 @ 30fps, 1680 frames = 56.0s)

| Frames | Time | Scene | What's on screen |
|---|---|---|---|
| 0–150 | 0:00–0:05 | Hook | A prompt typed live, then the 5-hour-limit wall hits — recreated in JSX so the typing can animate |
| 150–270 | 0:05–0:09 | Brand | Icon pop, "ClaudeMeter", tagline, "Built in Rust · under 10 MB RAM" |
| 270–450 | 0:09–0:15 | Step 01 · Tray | Vector tooltip recreation + an animated tray strip counting the % up, colour-coded |
| 450–690 | 0:15–0:23 | Step 02 · Dashboard | Vector dashboard recreation (`MockDashboard`, dark theme) with 4 labelled callouts |
| 690–1080 | 0:23–0:36 | Examples | 4 cards, ~3.25s each: tray icon styles, the 4 themes (vector, one per theme), the notification toast, the Codex panel |
| 1080–1230 | 0:36–0:41 | Settings | Vector settings-list recreation + the setting names as chips |
| 1230–1380 | 0:41–0:46 | Languages | All 40 shipped locale names in their own script, in a staggered grid |
| 1380–1530 | 0:46–0:51 | Stats | Count-up tiles (1.9 MB RAM, 3 MB exe, 40 languages, 0 telemetry) + a vector Task Manager recreation |
| 1530–1680 | 0:51–0:56 | CTA | Icon, tagline, separate Windows 10/11 and macOS 12+ buttons (each with its own OS icon), repo link, studio credit |

Every `from` is derived from `DURATIONS` in `src/theme.ts` — retiming or inserting a scene is a one-line edit there.

## Storyboard — 9:16 Shorts cut (`ShortsEN`, 1080x1920 @ 30fps, 825 frames = 27.5s)

Cut, not compressed: drops Settings and Languages (no reading time at this pace), keeps 2 of the 4 example cards, and every beat is faster than the long cut.

| Frames | Time | Scene |
|---|---|---|
| 0–105 | 0:00–0:04 | Hook |
| 105–180 | 0:04–0:06 | Brand |
| 180–300 | 0:06–0:10 | Step 01 · Tray |
| 300–435 | 0:10–0:15 | Step 02 · Dashboard |
| 435–615 | 0:15–0:21 | Examples (tray icon styles, themes) |
| 615–705 | 0:21–0:24 | Stats |
| 705–825 | 0:24–0:28 | CTA |

Layout is fully responsive: every component reads `useVideoConfig()` and branches once on `height > width` (see `src/layout.ts`) — nothing is a raw 16:9 pixel value. Content stays inside the platform-safe middle band (~12% clear at top, ~20% at bottom) so YouTube/TikTok/Reels chrome never covers the CTA.

## Assets

- `src/components/Mock.tsx` — vector recreations of every app screen (`MockDashboard` in all 4 themes, `MockCodexPanel`, `MockTooltip`, `MockNotification`, `MockSettings`, `MockTaskManager`). All copy, numbers, colours and layout are taken from the screenshots in `screenshots/`, just redrawn as text/SVG instead of an `<Img>` so they render crisp at 4K instead of upscaling a ~380px-wide PNG. `screenshots/` itself is untouched at the repo root; nothing under `public/` is a raster app screenshot anymore.
- `src/components/OsIcons.tsx` — a plain 2x2-square Windows glyph and a simplified apple-silhouette (body + bite + stem + leaf, drawn from basic shapes rather than a traced trademark path) for the CTA's platform buttons.
- `public/brand/icon.png` — the real app icon (`assets/icon_app.png`).
- `public/audio/music.mp3` — "Precision Launch", the mixed-in Suno track (see Music below).
- All copy quotes README.md / the app's own strings (e.g. "5-hour session", "Weekly (7-day)", threshold percentages, RAM/size numbers) rather than paraphrasing.

## Not a mobile app — one template deviation

The build prompt this project was generated from assumes a mobile app with a Google Play listing (§5a: fetch the official Play Store badge). ClaudeMeter is a **Windows/macOS desktop tray app** distributed via GitHub Releases — there is no store badge. The CTA scene (`src/scenes/Cta.tsx`) instead shows two separate platform buttons — "Windows 10/11" with a Windows glyph and "macOS 12+" with an apple silhouette (`src/components/OsIcons.tsx`) — pointing at the real distribution channel (GitHub Releases + the project website), not an invented app-store presence.

## Music

Mixed in (`WITH_MUSIC = true` in `src/theme.ts`): "Precision Launch", 184.75s, supplied in `public/audio/music.mp3`.

- **Long cut**: starts 3s into the track (`MUSIC_START_SECONDS`) so the track's own ~8s kick-in lands under the Brand-reveal beat at video 0:05.
- **Shorts cut**: starts 4.5s in (`SHORTS_MUSIC_START_SECONDS`) for the same kick-in alignment at its earlier 0:03.5 Brand beat, staying inside the steady groove the whole way through (no dip before track-time 53s).
- Both compositions apply an eased ~1s fade-in and ~3-4s fade-out to true silence via a `volume` callback, scaled to `MUSIC_LEVEL` (0.55) at the plateau.
- Levels were measured, not guessed: the bundled ffmpeg/ffprobe (`node_modules/@remotion/compositor-win32-x64-msvc/`) decoded the track to raw PCM for a per-second RMS scan and a true-peak check (peak -0.62 dBFS raw, ~-5.8 dBFS after the 0.55 level — see the comment block above the constants in `src/theme.ts` for the full envelope notes). The rendered output's tail was re-checked the same way and confirmed to decay to near-silence (-54 dB by the last second) before the file ends.
- To swap in a different track: replace `public/audio/music.mp3`, re-run the same measurement pass, and adjust `MUSIC_START_SECONDS` / `SHORTS_MUSIC_START_SECONDS` / `MUSIC_LEVEL` in `src/theme.ts`.

## Thumbnails

Three variants (`ThumbnailA/B/C`, 1280x720) in `src/scenes/Thumbnail.tsx` for YouTube's built-in A/B test:

| Variant | Concept |
|---|---|
| A (recommended) | Problem → arrow → answer: "DON'T HIT THE WALL" next to the real 94%-red session bar |
| B | The hero number (94%) filling the frame, stamped "LIVE IN TRAY" |
| C | Abstract giant "?" + "HOW MUCH IS LEFT?" + the app icon/name |

Each was checked at sidebar size (210x118) — the size where the click is actually won.

> **Note:** the publishing drafts that accompanied this build — YouTube title/description/tag copy, the Suno beat map and prompts, and image-model fallback prompts for thumbnail art — are kept locally and excluded from the repo via the root `.gitignore`.
