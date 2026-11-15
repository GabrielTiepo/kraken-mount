# AGENTS.md

Instructions for coding agents working in this repo. Read this first, then `DESIGN.md`.

## What this is

Kraken Mount: a Rust ELF payload for the PS5 (jailbroken, elfldr). It finds games on the internal SSD
and USB/extended drives, keeps a per-game cache on the internal SSD, registers each game on the home
screen without installing it, and at launch mounts the game's fakelib plus ampr-emu-ng into the game
sandbox. Web UI on port 8097 (opened from the "Kraken Mount" home-screen tile).

- Sources: debug FPKG `.pkg` files and game folders (plain, zstd or LZ4 AMPR packs). Nothing else.
- ampr-emu-ng ("ng") is NOT built here: it is its own project. Its released `libSceAmpr.sprx` is
  bundled from `vendor/ampr-emu-ng/`; to update, replace the file, `SHA256SUMS` and `VERSION` from
  the new GitHub release. A file put into `/data/krakenmount/ampr/` by hand overrides it. Do not edit the ng project from here unless the task asks for it.
- References (read-only): ShadowMountPlus (SMP, the C original of the register/fakelib/backport
  logic) and ps5-dump-forge (toolchain, self copy, tile, toasts, UI patterns).

## Layout

| Path | What |
|---|---|
| `src/main.rs` | start-up, scan loop (pauses while a game runs), self copy |
| `src/config.rs` | `/data/krakenmount/config.toml` settings |
| `src/storage.rs` | drives and default scan roots |
| `src/scan.rs` | game discovery, backports, duplicates |
| `src/cache.rs` | per-game cache `/data/krakenmount/caches/<TID>`, freshness, `.built`, purge |
| `src/backport_inventory.rs` | file metadata inventory distinguishing non-library backport files from fakelib-only changes (L03) |
| `src/ampr_index.rs` | writes `ampr_emu.index` + `ampr_assets.index` for pkg caches |
| `src/register.rs` | home-screen registration, `/system_ex/app/<TID>` mounts, the tile |
| `src/launch.rs` | launch watcher: the fakelib overlay, writable `/app0`, `/app0/kraken` |
| `src/web.rs`, `src/ui.html` | HTTP server and the single-file UI |
| `src/sys.rs` | log, toasts, mount syscalls, kstuff probe |
| `src/net.rs` | HTTPS GET over the console's sceHttp/sceSsl (ampr-emu-ng auto-update) |
| `src/diagnostics.rs` | RAM-only scan/cache timers; not AMPR runtime read timings |
| `src/removable_guard.rs` | pure USB/ext mount checker for safe cache purge (L03) |
| `src/i18n.rs` | toast texts in the console language (the UI's 11 languages) |
| `crates/kraken-mount-pkg` | FPKG reader and its Kraken decoder (`decoder` module) |
| `ps5/` | Docker build, linker, C shims (`compat.c` holds PS5 libc adapters) |
| `DESIGN.md` | the contract: paths, settings, cache rules, flow, module interfaces, web API |

## Build and check

- Fast check (host, no Docker): `cargo check --offline --target x86_64-unknown-freebsd`. Must be 0 errors and
  0 warnings.
- Payload: `ps5/build.sh` (Docker, two stages: stage 1 is the saved copy, stage 2 embeds it) ->
  `target/ps5/kraken-mount.elf`. A link error like `undefined symbol: X@FBSD_1.0` means the PS5 libc
  lacks that versioned symbol: add an adapter in `ps5/compat.c` (see `statfs`, `getmntinfo`).
- The **payload** builds only for PS5/FreeBSD. The pure `kraken-mount-pkg` reader also has
  native host tests: `cargo test -p kraken-mount-pkg --lib --locked`. Synthetic FIH/CNT fixtures
  exercise error handling but **do not** prove end-to-end PFS/AMPR or game compatibility.
- Backport inventory host tests: `rustc --edition=2021 --deny warnings --test src/backport_inventory.rs -o /tmp/backport-inventory-tests && /tmp/backport-inventory-tests`.
- USB guard host tests: `rustc --edition=2021 --deny warnings --test src/removable_guard.rs -o /tmp/removable-tests && /tmp/removable-tests`.
- L04 host tests: `rustc --edition=2021 --deny warnings --test src/diagnostics.rs -o /tmp/diag-tests && /tmp/diag-tests`, UI smoke: `node scripts/test-diagnostics-ui.cjs`.
- Changes to pure parser logic need native tests. `cargo check --workspace --locked --target
  x86_64-unknown-freebsd` remains mandatory for target compatibility. UI script: extract the
  `<script>` and run `node --check`.
- The CI runs fast checks/tests **before** the Docker PS5 ELF build; changes to runtime behavior
  must still be tested on real hardware and labelled NOT HOMOLOGATED when not tested.
  See `docs/L00-BASELINE.md` and `docs/PLANO-KRAKEN-MOUNT-SATA-SSD.md`.

## Rules

- Naming: "Kraken Mount" in anything the user sees (UI, toasts, tile); `kraken-mount` for identifiers.
  Paths on the PS5 stay `krakenmount` (`/data/krakenmount/...`, `/user/data/krakenmount/...`): never
  rename them.
- `DESIGN.md` is the contract. Change it together with the code, and keep module interface names.
- Mount safety: never `MNT_FORCE` from register/purge; a busy mount means a running game. The scan loop
  must not build, register or purge while a game runs (`running` set from the launch watcher).
- `/app0` is writable. Game-written files in a cache survive Rebuild (`.built`); only Purge deletes them.
- Old-layout caches (`/data/krakenmount/<TID>`) are deleted by hand: no cleanup code for them.
- Pkg-specific behaviour in ng goes behind `pkg_mode()`; zstd/LZ4 behaviour stays unchanged. Pkg perf
  changes need an A/B launch of at least two pkg games, with numbers.
- Licenses: a dependency added or bumped in `Cargo.lock` gets its license text in `THIRD_PARTY_NOTICES`.
- Style: smallest correct diff, no new dependencies, `ponytail:` comment on deliberate shortcuts and
  their limits, errors logged with the title id and path.
- UI: plain JS + CSS in one file, works on the PS5 browser (old WebKit, D-pad, 1920x1080) and phones;
  no external requests.
