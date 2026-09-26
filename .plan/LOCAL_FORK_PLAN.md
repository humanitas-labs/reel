# Local-only fork plan

Goal: turn this checkout of Cap into a private, fully offline macOS screen recorder for one user. Studio recording, screenshots, the editor, captions and export stay. Accounts, sign-in, uploads, share links, instant mode, telemetry, crash reporting, update checks, licensing and every other code path that reaches a server are deleted, not disabled. The app is renamed and re-iconed. Nothing else in the monorepo (web app, mobile, CLI, extension, bots, infra) is kept.

Names: `NEWNAME` = `Reel` (provisional), `NEWID` = `com.andjones.reel`. Deep-link scheme `reel://`.

Working rules for every phase: delete rather than stub wherever the compiler lets us. Keep the studio recording, screenshot, editor, rendering and export crates untouched unless a removal forces a change. Commit at the end of each phase so any phase can be bisected or reverted. Follow `AGENTS.md` for lint shape; comments stay out unless they capture a non-obvious decision.

## Phase 0. Baseline

- [x] Branch `local-fork` from `main`. Keep `origin` for reference only; never push there.
- [x] Confirm the current app builds and runs from source on this machine before touching anything: `bun install`, `bun run dev:desktop`. Rust 1.88 is pinned by `rust-toolchain.toml`. Bun must be 1.4.x: 1.3.6 rewrites `bun.lock` to an older format and re-resolves dependencies, which produces about 400 spurious type errors. Use `bun install --frozen-lockfile`. If the dev build fails on stock `main`, fix that first so later breakage is attributable.
- [x] Note the current app data dir (`~/Library/Application Support/so.cap.desktop.dev`) and the recordings folder so existing recordings can be copied into the renamed app later.

## Phase 1. Prune the monorepo

Delete everything that is not the desktop app or a crate it depends on.

- [x] Delete apps: `apps/web`, `apps/mobile`, `apps/chrome-extension`, `apps/discord-bot`, `apps/media-server`, `apps/desktop-gpui`, `apps/src`. `apps/cli` stays: the desktop app's export worker (`cap-exporter` sidecar) is that binary, and its local subcommands (record, screenshot, targets, recordings, export, project, doctor, mcp) are worth keeping. Its server-backed subcommands are stripped instead.
- [x] Delete packages: `packages/database`, `packages/web-api-contract`, `packages/web-api-contract-effect`, `packages/web-backend`, `packages/web-domain`, `packages/s3`, `packages/sdk-embed`, `packages/sdk-recorder`, `packages/local-docker`, `packages/env`, `packages/utils` (the desktop frontend imports none of these; `@cap/utils` and `@cap/database` are listed in `apps/desktop/package.json` but never imported). Keep `packages/ui-solid`, `packages/tsconfig`, `packages/config`, `packages/recorder-core` and `packages/ui` only if the desktop build still needs them; check with a grep after deletion.
- [x] Delete `emails/`, `infra/`, `docker-compose*.yml`, `crates/api` (a 3-line stub nobody depends on), `crates/cli-install` if only the CLI used it.
- [x] Root `package.json`: trim `workspaces` to `apps/*` and `packages/*`, remove web, docker, db and email scripts. `turbo.json`: keep only the `@cap/desktop#build` task and its dependencies.
- [x] Root `Cargo.toml`: workspace members are `apps/cli`, `apps/desktop/src-tauri` and `crates/*`. cargo-hakari and `crates/workspace-hack` are removed (CI build-time optimisation only). The `sentry` workspace dependency goes in Phase 2.
- [x] Scripts: delete `scripts/run-gpui-build.mjs`, `build-gpui-binary.sh`, `bundle-gpui-mac`, `prepare-gpui-dependency.*`, `verify-gpui-release-inputs.mjs`, `scripts/emails`, `scripts/analytics`, `scripts/loops`, the instant-mode benchmark scripts, and every Linux and Windows packaging script. In `apps/desktop/package.json` remove `build:gpui*` from `dev` and `build:tauri`.
- [x] Sidecars: `tauri.conf.json` `externalBin` lists `cap-muxer`, `cap-exporter`, `cap-cli`. Keep the first two. Drop `cap-cli` and remove the CLI settings page (`settings/cli.tsx`) and the `cli.rs` handling in the desktop crate that installs it.
- [ ] gpui handoff (moved to Phase 2, compile-driven): delete `apps/desktop/src-tauri/src/gpui_app.rs` and its call sites in `lib.rs` (module at l.29, commands at l.6752, startup redirect around l.7196, update handoff and deep-link forwarding around l.7944). Remove `enable_gpui_app` from `general_settings.rs` (l.257) and the Experimental settings page toggle.
- [x] `bun install` and `cargo check -p cap-desktop -p cap` pass. Four TypeScript errors remain in files Phase 3 deletes (`web-api.ts`, `organization-branding.ts`, `license.tsx`, `s3-config.tsx`) because `@cap/web-api-contract` is gone.
- [x] CLI stripped to local subcommands: export, export-preview, project, record, screenshot, recordings, targets, doctor, selftest, version, guide, automations, completions. 20.9k lines down to 8.8k; clippy and 76 tests pass. The `mcp` subcommand was removed entirely because all 77 of its tools called the web API.
- [x] Scripts: root `scripts/` is down to the eight files the build uses; `apps/desktop/scripts` keeps only `prepare.js`. The dSYM release hook (`prodBeforeBundle.js`) and `.agents/skills/building` are gone.

## Phase 2. Strip the Rust side

All paths under `apps/desktop/src-tauri/src` unless stated. Method: delete the module, remove `mod` and `use` lines, run `cargo check -p cap-desktop`, fix every error at the call site by removing the feature rather than stubbing it. Iterate.

Delete outright:

- [x] `upload.rs` and the `upload/` directory (lifecycle, preparation, recovery_age, resume).
- [x] `api.rs`, `web_api.rs`, `auth.rs`, `http_client.rs`.
- [x] `telemetry.rs`, `recording_telemetry.rs`, `updates.rs`.
- [x] `crash_sentinel.rs`: it detects unclean exits locally and reports to Sentry at l.294-327. Delete the file unless the local detection is wanted; it is not, so delete.
- [x] Log and diagnostics upload: `upload_log_file` in `logging.rs` (l.252), `upload_diagnostic_report` in `diagnostics.rs` (l.538), the "Upload logs" tray item in `tray.rs` (l.991), `crates/utils/src/log_upload.rs`.

Rework:

- [x] `main.rs`: remove Sentry init (l.54-85), the OTLP exporter (l.152-180), the Sentry user set (l.217). Rename the log directory from `so.cap.desktop` to `NEWID`.
- [x] `lib.rs` (11k lines): remove the upload, share, screenshot-share, plan, license and server-URL commands (`upload_exported_video` around l.5157, screenshot upload commands l.5324-5490, `UploadResult` l.1507, `check_upgraded_and_update` l.5917, `update_auth_plan` l.6551, `open_pricing_page` l.6266, `set_server_url`), the Sentry user set at startup (l.7318), `upload::lifecycle::init` (l.7448) and upload recovery at startup (l.8630-8750), the `upload_session_active()` exit guards (l.3597, l.8347), the `signin` window label (l.8310). Remove the `ShowCapWindow::Upgrade` variant and its route in `windows.rs` (l.1194-1373, 2384-2410, 3530).
- [x] `recording.rs` (10k lines): remove instant mode. Delete the `upload`, `api`, `auth`, `web_api` imports (l.62-85), the sign-in requirement in `recording_start_mode_error` (l.1223, l.2069), the remote video creation and plan-based resolution (l.2285-2340), the upload session and `strict_instant::spawn` start (l.2786-2818), `delete_remote_instant_video` (l.3804), and the `upload::lifecycle::supervise` calls (l.5162, 5763, 6189, 8407). Remove `Instant` from the `RecordingMode` enum so the compiler finds every remaining branch. `crates/recording/src/instant_recording.rs` is local capture code; delete it if nothing references it after the mode is gone.
- [x] `crates/recording`: delete `upload_preparation.rs`, `upload_resume.rs`, `upload_verification.rs` and `is_uploadable`.
- [x] `crates/project/src/meta.rs`: delete `SharingMeta`, `S3UploadMeta`, `UploadMeta` and the `sharing` and `upload` fields on `RecordingMeta` (l.78-134). Old `recording-meta.json` files with those keys still load because serde ignores unknown fields by default; confirm the struct is not `deny_unknown_fields`.
- [x] `general_settings.rs`: delete `commercial_license`, `server_url`, `auto_create_shareable_link`, `upload_individual_files`, `delete_instant_recordings_after_upload`, `instant_mode_max_resolution`, `enable_telemetry`, `update_channel`, `enable_gpui_app`. The new bundle identifier gives a fresh settings file, so no compatibility shims.
- [x] `automation.rs` and `crates/automation`: remove the `upload` action (l.215-275), the `{share_link}` template, the `uploadCompleted` and `instantRecordingFinished` triggers, the `organizationIs` condition, and the `webhook` action (`crates/automation/src/lib.rs` l.238, l.445 and `automation.rs` l.400). Webhooks are user-configured but still outbound network, and the goal is no network code paths.
- [x] `notifications.rs`: drop the `ShareableLinkCopied`, `ShareableLinkFailed`, `UploadFailed` variants.
- [x] `export.rs`: remove the ten `sentry::capture_message` calls. `crates/editor/Cargo.toml` l.42 depends on `sentry` without using it; remove.
- [x] `deeplink_actions.rs` is local (`cap://action?...` for start/stop/open editor). Keep it but rename the scheme in Phase 4, or delete it if the automation hooks are not wanted. Decision: keep.
- [x] `flags.rs` and `crates/flags`: keep, they are local.

Dependencies and Tauri config:

- [x] `apps/desktop/src-tauri/Cargo.toml`: remove `sentry`, `tauri-plugin-sentry`, `tauri-plugin-updater`, `tauri-plugin-oauth`, `tauri-plugin-http`, `tauri-plugin-deep-link` (only if `deeplink_actions.rs` is also dropped; the single-instance plugin's `deep-link` feature goes with it), `reqwest` (after captions decision below), `opentelemetry*`, `tracing-opentelemetry`. Keep `axum` and `tokio-tungstenite`, they serve the local frame websocket.
- [x] `lib.rs` plugin registration (l.7083-7134): remove the deleted plugins.
- [x] `capabilities/default.json`: remove `oauth:allow-start`, `updater:default`, `http:default` and its `http://*` and `https://*` allow list (l.54-72). Remove `deep-link:default` if the plugin is gone.
- [x] `tauri.conf.json`: set `plugins.updater` removed, CSP `connect-src` to `'self' ws: ipc: http://ipc.localhost` only (drop `https://t.cap.so`), set `createUpdaterArtifacts` false.
- [x] `tauri.prod.conf.json`: delete the updater block entirely.
- [x] `Entitlements.plist`: remove `com.apple.security.network.client` and `network.server`.
- [x] Captions (`captions.rs` l.2154-2336) download whisper models from a GitHub release on first use. Decision pending. Default plan: keep as the single documented exception, using `reqwest` only there. If strict zero network is chosen: delete the download code, keep the model loader, and document the path where a `ggml-*.bin` file must be placed by hand.
- [x] Gates: `cargo fmt --all`, `cargo check -p cap-desktop`, then `cargo clippy -p cap-desktop --all-targets -- -D warnings` at the end of the phase. Run `bun run dev:desktop` once so `apps/desktop/src/utils/tauri.ts` regenerates; commit the regenerated file with the Rust changes.

## Phase 3. Strip the TypeScript side

All paths under `apps/desktop/src`. Do this after Phase 2 so the regenerated bindings tell the typechecker which commands no longer exist.

Delete outright:

- [x] `utils/auth.ts`, `utils/web-api.ts`, `utils/analytics.ts`, `utils/env.ts`, `utils/server-url-routing.ts` and its test, `utils/currency.ts`, `utils/plans.ts`, `utils/pricing.ts`, `utils/organization-branding.ts` (after extracting the swatch helper, below).
- [x] `components/SignInButton.tsx`, `components/callback.template.ts`.
- [x] Routes: `(window-chrome)/upgrade.tsx`, `update.tsx`, `settings/license.tsx`, `settings/changelog.tsx`, `settings/feedback.tsx`, `settings/integrations/` (index, s3-config, google-drive-config), `settings/cli.tsx`, `new-main/ChangeLogButton.tsx`, `editor/ShareButton.tsx`, `editor/OrganizationDropdown.tsx`, `utils/updater.ts`.
- [x] Assets: `assets/instant-toggle.jpg`, `assets/illustrations/instant-mode-*.png`, `assets/illustrations/cloud-*.png`.

Rework:

- [x] `store.ts`: delete `authStore` and `userProfileStore` (l.124-126). Change `recordingSettingsStore` default mode from `"instant"` to `"studio"` and drop `organizationId` (l.137-148).
- [x] `app.tsx`: remove the analytics mount hook (l.136-142) and the routes for upgrade, update, license, changelog, feedback, integrations, s3, google-drive (l.59-87, 201-238).
- [x] `routes/(window-chrome)/settings.tsx`: remove the account card, profile image fetch, dashboard link, sign-out, update checker and nav entries for integrations, license, feedback, changelog (l.61-85, 153-177, 240-260, 286-288, 413, 427-542).
- [x] `settings/general.tsx`: remove the server URL field (l.715-731, 968), `TelemetryCard` (l.734, 830), delete-after-upload toggle (l.577), and the "Cap" logo usage at l.1127.
- [x] `settings/experimental.tsx`: remove the gpui toggle.
- [x] `settings/quality.tsx`: remove the instant resolution section and Pro gate (l.84-297) and the "open share links automatically" toggle (l.314).
- [x] `settings/hotkeys.tsx`: remove `startInstantRecording` (l.27).
- [x] `settings/recordings.tsx`: remove the Instant tab (l.53), the upload progress listener (l.109-124), the reupload button (l.466-481), the `trackEvent` calls (l.167-184).
- [x] `settings/screenshots.tsx`: remove `handleCreateShareableLink` (l.128-147) and `trackEvent` (l.100-131).
- [x] `settings/automations.tsx` and `utils/automations.ts`: remove the upload action, upload and instant triggers, `organizationIs` condition and share-link templates (l.70-304, 972-1161; `utils/automations.ts` l.54-239), matching the Rust removals.
- [x] `new-main/index.tsx`: remove `authStore` query and `serverUrl` memo (l.1849-1852), `updateAuthPlan` on mount (l.2458), license query and sign-in mutation (l.2706-2708), the `start-sign-in` listener (l.2990-3009), the dashboard logo link (l.3146-3157, replace with a plain logo), the plan badge (l.3160-3180), sign-in pending UI (l.3229-3247), the update check and update-ready toast (l.1663-1760, 2356-2357), the changelog bell (l.3118), and the upload-progress and reupload plumbing (l.2100-2150) that feeds `TargetMenuGrid.tsx` and `TargetCard.tsx`; strip the matching props from those two files.
- [x] Mode UI: remove "Instant mode" from `components/Mode.tsx` `MODE_BUTTONS` (l.22-49), `components/ModeSelect.tsx`, `routes/mode-select.tsx`, `new-main/ModeInfoPanel.tsx`, and the onboarding instant card and mockup (`onboarding.tsx` l.58-106, 512, 1086-1124, 1195-1445). Consider deleting the mode-select window entirely if only studio and screenshot remain and a two-way toggle suffices.
- [x] `routes/target-select-overlay.tsx`: remove `createOrganizationsQuery` (l.178-188), the instant sign-in emits (l.1991, 2286), the mode toggle's instant branch (l.2165-2168), `ShowCapFreeWarning` (l.2425-2443).
- [x] `routes/in-progress-recording.tsx`: remove the `authStore` read (l.112-119) and the free-plan five-minute auto-stop (l.844-866).
- [x] `routes/recordings-overlay.tsx`: remove the share-link actions and upload flow (l.324-409, 753-861) and upgrade tooltips (l.179, 597).
- [x] `utils/recording.ts`: remove the `InvalidAuthentication` and `UpgradeRequired` handlers (l.62-96).
- [x] `utils/queries.ts`: remove `createLicenseQuery` (l.334-363), `createCustomDomainQuery` (l.435-458), `createOrganizationsQuery` (l.460-471).
- [x] Editor: `editor/context.ts` drop `createCustomDomainQuery` (l.2139, 2482) and the `destination?: "link"` export type (l.157). `editor/Header.tsx` remove `OrganizationDropdown` (l.186), `ShareButton` (l.228) and `trackEvent` (l.241). `editor/ExportPage.tsx` remove the "Shareable Link" destination (l.107, 959-988), auth and org selection (l.191-300), the upgrade gate (l.854-862), upload handling (l.888-909), `trackEvent` (l.847, 1317).
- [x] Brand colours: `ConfigSidebar.tsx` and about six colour pickers import `getOrganizationBrandColorSwatches` and `OrganizationBrandColorSwatch` from `organization-branding.ts` (l.158). Move the swatch type and a local-only swatch source (empty list, or a user-defined palette in settings) into `utils/brand-colors.ts` and repoint the imports.
- [x] Screenshot editor: remove the share destination in `screenshot-editor/screenshotExport.ts` (l.38, 430-515), `useScreenshotExport.ts` (l.142-245), and the "Create shareable link" buttons in its `Header.tsx` (l.105, 165).
- [x] Remaining `trackEvent` call sites after `analytics.ts` is deleted: `new-main/MicrophoneSelect.tsx` (l.54, 182), `new-main/CameraSelect.tsx` (l.265). The typechecker will list any others.
- [x] `routes/debug.tsx`: remove the update check (l.42).
- [x] `apps/desktop/package.json`: remove `@cap/database`, `@cap/utils`, `@cap/web-api-contract`, `@openpanel/web`, `@ts-rest/core`, `@tauri-apps/plugin-http`, `@tauri-apps/plugin-updater`, `@tauri-apps/plugin-deep-link` (if dropped). `app.config.ts`: remove `@openpanel/web` from `optimizeDeps`. `vite-env.d.ts`: drop the `VITE_SERVER_URL`, `VITE_OPENPANEL_*`, `VITE_VERCEL_*`, `VITE_ENVIRONMENT` declarations. `.env` and `scripts/setup.js`: strip the same variables.
- [x] Gates: `bun run biome check --write` on touched files, then `bun run typecheck` for the desktop app.

## Phase 4. Rebrand

- [x] Names and identifiers: `tauri.conf.json` `productName` to `NEWNAME - Development`, `identifier` to `NEWID.dev`, `mainBinaryName`; `tauri.prod.conf.json` `productName`, `mainBinaryName`, `identifier` to `NEWID`. Generate new WiX `upgradeCode` values or delete the Windows and Linux bundle sections entirely since only macOS is built.
- [x] Icons: from one 1024x1024 PNG run `bun run tauri icon path/to/logo.png` inside `apps/desktop`, which regenerates `src-tauri/icons/` (icns, ico, png sizes). Delete the `android/`, `ios/` and `linux/` icon folders and the Linux tray SVG entries in `tauri.conf.json`. Replace `tray-default-icon*.png` and `tray-stop-icon.png` with monochrome template images of the new mark (drop the `-instant` variant).
- [x] Logo SVGs: replace `packages/ui-solid/icons/logo.svg`, `logo-full.svg`, `logo-full-dark.svg`; delete `instant.svg`. They are auto-imported as `IconCapLogo`, `IconCapLogoFull`, `IconCapLogoFullDark`; either keep those identifiers or rename the icon collection prefix in `packages/ui-solid/vite.js` and update the six call sites (`Loader.tsx`, `CapErrorBoundary.tsx`, `onboarding.tsx` l.1915, `new-main/index.tsx` l.3156-3157, plus any survivors).
- [x] `entry-server.tsx` l.11 favicon path, `assets/dmg-background.png`, delete the NSIS and WiX bitmap assets.
- [x] Rust hardcoded `so.cap.desktop`: `main.rs` log dir, `tray.rs` l.45-47, `crates/recording/src/sources/screen_capture/mod.rs`, `crates/export/tests/export_benchmark.rs`, `stop_editor_benchmark.rs`. Grep `so\.cap` and `cap\.so` across the tree to catch the rest.
- [x] Deep-link scheme: `cap-desktop` in `tauri.conf.json` and `cap://` in `deeplink_actions.rs` become `reel://`. Update `Info.plist` document type "Cap Recording" and the usage strings.
- [x] File association: keep the `.cap` project extension so existing recordings open unchanged. Rename only the display name in `fileAssociations`.
- [x] User-visible strings: about 80 occurrences of "Cap" across the frontend, densest in `onboarding.tsx` (17), `settings/general.tsx` (12), `new-main/index.tsx` (9), `editor/ImportProgress.tsx` (4), `settings/experimental.tsx` (4), `settings/automations.tsx` (3), and "Update Cap" dialog titles that disappear with the updater. Grep `\bCap\b` in `.tsx` and `.ts`, review each hit by hand; do not blind-replace, since `cap` also appears in `.cap` and `CapWindowId`.
- [x] Internal identifiers (`@cap/*` package names, `cap-*` crate names, `CapWindowId`, `cap.` localStorage prefix) stay as they are. They are invisible to the user and renaming them is churn with no payoff.
- [x] `README.md`, `CONTRIBUTING.md`, `AGENTS.md`: rewrite the README for the fork, delete CONTRIBUTING, trim AGENTS.md of web, database and Effect sections. Keep `LICENSE` (AGPLv3, private use carries no obligations).

## Phase 5. Verify

- [x] Static audit: grep the remaining tree for `reqwest`, `fetch(`, `tauriFetch`, `https://`, `http://` (excluding `localhost`, `ipc.localhost`, `asset.localhost`, schema URLs and licence text), `cap.so`, `openpanel`, `sentry`, `posthog`, `crabnebula`, `amazonaws`, `googleapis`, `vercel`. The only permitted hits are the whisper model URLs if that exception is kept.
- [x] Dependency audit: `cargo tree -p cap-desktop | grep -iE "reqwest|hyper|sentry|opentelemetry|updater|oauth"` shows nothing beyond what the captions exception needs. `bun pm ls` shows no `@openpanel`, `@ts-rest`, `plugin-http`, `plugin-updater`.
- [ ] Build gates: `cargo clippy --workspace --all-targets -- -D warnings`, `bun run lint`, `bun run typecheck`, `bun run tauri:build` producing a signed-for-local-use `.app` and `.dmg`.
- [ ] Runtime audit: launch the release build, record a studio clip with camera and mic, take a screenshot, edit, generate captions, export to file and clipboard. During the whole session watch `nettop -p <pid>` or Little Snitch. Expected outbound connections: zero, or one GitHub fetch on first caption use if that exception is kept.
- [ ] Copy existing recordings from the old app data dir into the new one and confirm they open in the editor.
- [ ] Squash or keep the phase commits, tag `v1.0-local`.

## Parallel execution (from the Phase 2 checkpoint)

Commit `33947d0` ("wip: remove network, auth, telemetry and update code from the desktop crate") is the checkpoint: `apps/desktop/src-tauri` no longer references the deleted modules anywhere except `recording.rs` (43 errors, instant mode) and `automation.rs` (8 errors, upload and webhook actions). Every remaining unit below owns a disjoint file set, so they run as concurrent agents. Each agent reads `AGENTS.md`, edits only its files, runs its scoped checks, and does not commit. The `tauri.ts` bindings regenerate from the `export_bindings` test in `lib.rs`, so no app launch is needed.

Wave 1 (concurrent), landed in commit fe1596e:

- **A. Recording.** `src/recording.rs`, `src/linux_instant_camera.rs`, `src/recording_settings.rs`, `crates/recording/**`, and the `VideoUploadInfo` / `UploadMode` remnants in `src/lib.rs`. Delete `RecordingMode::Instant` and `InProgressRecording::Instant`, then remove every branch the compiler flags; drop the health accumulator and telemetry emitters; delete the instant recording, upload preparation, upload resume and upload verification modules from the recording crate.
- **B. Automation.** `crates/automation/**`, `src/automation.rs`, `apps/cli/src/automation.rs`. Remove the upload and webhook actions, the `{share_link}` template, the upload and instant triggers, and the organisation condition from the trait and both hosts.
- **D. Windows, exit and config.** `src/windows.rs` (Upgrade window), `src/notifications.rs` (share and upload variants), `src/exit_shutdown.rs` (UploadActive, UpdateInstalling), desktop `Cargo.toml` dependency pruning, `tauri.conf.json` and `tauri.prod.conf.json` (CSP, updater artifacts, product name Reel, identifier com.andjones.reel, deep-link scheme reel), `capabilities/default.json`, `Entitlements.plist`.
- **E. TypeScript strip (Phase 3).** Everything under `apps/desktop/src` except `utils/tauri.ts`, plus `apps/desktop/package.json`, `app.config.ts`, `vite-env.d.ts`. Biome on touched files only; the typecheck gate runs in wave 2 after bindings regenerate.
- **F. Branding assets (Phase 4, non-string part).** `apps/desktop/src-tauri/icons/**`, `packages/ui-solid/icons/*.svg`, `Info.plist`, the `so.cap.desktop` identifiers in `src/main.rs`, `src/tray.rs`, `crates/recording/src/sources/screen_capture/mod.rs`, `crates/export/tests/export_benchmark.rs`, `src/stop_editor_benchmark.rs`; README rewrite, CONTRIBUTING removal, AGENTS.md trim.

Wave 2 (landed in the commit after fe1596e; `cargo check -p cap-desktop -p cap --all-targets` is clean with 12 dead-code warnings for unit G):

- **C. Project metadata.** `crates/project/src/meta.rs` sharing and upload types and their remaining callers in `src/lib.rs`, `src/screenshot_editor.rs`, `apps/cli/src/record.rs`.
- **G. Rust gate.** `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, dead-code removal, `cargo test -p cap-desktop export_bindings` to regenerate `tauri.ts`.
- **H. TypeScript gate and string rebrand.** `bun run typecheck`, fix fallout from the new bindings, then the user-visible "Cap" strings across the frontend and `entry-server.tsx`.
- **I. Phase 5 verification** as written above.

## Status notes

- Platform scope is macOS only. The Linux-only instant camera and clean-capture code paths reference deleted modules and will not compile on Linux; Windows and Linux bundle sections are removed from the Tauri config.
- The `cloud-*.png` onboarding illustrations stay: they are the startup parallax art, not instant-mode art.
- `mode-select.tsx` stays as a two-option window (studio, screenshot).

## Open decisions

1. Product name: provisionally Reel. Revisit before Phase 4.
2. Logo source file (1024px PNG or SVG).
3. Whisper model download: keep as the single exception (decided by default; revisit any time).
4. Local deep-link actions (`reel://action?...`): keep (decided by default).
