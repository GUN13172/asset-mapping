# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Summary

Asset Mapping Tool (资产测绘工具) — a cross-platform desktop app for querying network asset search engines (Hunter, FOFA, Quake, DayDayMap). Built with Tauri 2 + React 18 + TypeScript + Rust.

## Commands

```bash
# Development (full app with hot-reload)
npm run tauri dev

# Frontend only (Vite dev server on port 1420)
npm run dev

# Production build (outputs to src-tauri/target/release/bundle/)
npm run tauri build

# Type-check + frontend production bundle
npm run build

# Rust checks
cd src-tauri && cargo check
cd src-tauri && cargo test
cd src-tauri && cargo clippy

# Run a single Rust example
cd src-tauri && cargo run --example test_hunter_api
```

No dedicated lint or test scripts in package.json. TypeScript checking is via `tsc` (called by `npm run build`). Rust tests use `cargo test` with quickcheck for property-based testing.

## Architecture

### Frontend ↔ Backend Communication

- Frontend calls Rust via Tauri's `invoke()` IPC. All commands are registered in `src-tauri/src/main.rs`.
- Long-running operations (export, vulnerability scan) stream progress via Tauri event emission (`window.emit()`), consumed by frontend event listeners.
- All Rust structs shared with the frontend use `#[serde(rename_all = "camelCase")]`.

### Backend (src-tauri/src/)

- `main.rs` — App bootstrap, all `#[tauri::command]` handlers (~30), and the `dispatch_platform!` macro for routing calls to the correct platform module.
- `api/` — One module per platform (hunter, fofa, quake, daydaymap) each implementing `search`, `export`, `validate_api_key`. `key_manager.rs` handles multi-key rotation.
- `config/` — Settings persistence and platform-specific config via `ConfigManager`.
- `converter/` — Query syntax translation between platforms (fields, operators, validation).
- `history.rs` — Query/scan history persisted as JSON files in the OS config directory.
- `pocs.rs` — Nuclei POC template discovery and vulnerability scanning orchestration.
- `error/` — Unified error types.

### Frontend (src/)

- `App.tsx` — Sidebar navigation shell with `React.lazy()` views. Predictive preloading via `likelyNextViews` adjacency map.
- Components are page-level (AssetQuery, ApiKeyManagement, ExportData, QueryConverter, Resender, PocManager, VulnerabilityScan, HistoryRecords, Settings).
- Theming via CSS Variables (`theme.css` for dark, `theme-light.css` for light) + Ant Design `algorithm` switching + `data-theme` attribute on `<html>`.

### Key Patterns

- **`dispatch_platform!` macro**: Eliminates repeated match arms when routing to platform-specific functions.
- **Cooperative cancellation**: Vulnerability scans use a global `AtomicBool` (`SCAN_CANCEL_FLAG`) checked between iterations.
- **Retry with backoff**: Export operations retry failed pages up to 3 times with 5-second delays.
- **Proxy support**: `create_http_client()` applies proxy/TLS settings from user config to all outgoing requests.
- **Config storage**: API keys and settings stored as JSON in `dirs::config_dir()/asset-mapping/`. Query syntax config bundled as a Tauri resource (`config.json`).

## Adding a New Platform

1. Create `src-tauri/src/api/<platform>.rs` implementing `search`, `export`, `validate_api_key`
2. Register the module in `src-tauri/src/api/mod.rs`
3. Add the platform string to `dispatch_platform!` match arms in `main.rs`
4. Add frontend UI support in the relevant components

## Prerequisites

- Node.js >= 18
- Rust >= 1.70
- Linux only: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
