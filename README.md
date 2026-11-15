<p align="center"><img src="docs/logo.png" width="128" alt="Kraken Mount"></p>

A PS5 game mounter. It finds games on the internal SSD, the M.2 drive and USB drives, and adds
them to the home screen without installing them.

Supported formats: debug FPKG (`.pkg`) and game folders (plain, or packed with zstd or LZ4).
Other formats, such as exFAT images, are obsolete and will not be supported.

## How to

1. Download `kraken-mount.elf` from [Releases](https://github.com/quer3q/kraken-mount/releases)
   and send it to your jailbroken PS5 with the ELF loader (elfldr, port 9021) or payload-manager:
   `nc -w 3 <ps5-ip> 9021 < kraken-mount.elf`. kstuff must be loaded, and ShadowMount+ must not be
   running: Kraken Mount waits until both are true.
2. Put each game in a scan root:
   - internal SSD: `/data/homebrew`, `/data/etaHEN/games`
   - M.2 or extended drive (`/mnt/ext0`, `/mnt/ext1`): `homebrew`, `etaHEN/games` or the drive root
   - USB drive (`/mnt/usb0` to `/mnt/usb7`): `homebrew`, `etaHEN/games` or the drive root
3. Each game gets a "<name> registered" notification and its own home-screen tile.

[ampr-emu-ng](https://github.com/quer3q/ampr-emu-ng) comes bundled and is installed into
`/data/krakenmount/ampr/` at start. To run another build, put its `libSceAmpr.sprx` there; it is
kept until you delete it.

Caches live in `/data/krakenmount/caches/<TITLE_ID>`, settings in
`/data/krakenmount/config.toml` and the log in `/data/krakenmount/kraken.log`.

### Source format filter (fork, opt-in)

In Settings, use **All: PKG and folders** (default), **Only PKG**, or **Only game folders**.
The config key `source_mode = "all" | "pkg" | "folder"` controls discovery only; games and
cached data are not deleted when toggling the filter. **PKG backports in folders still work
when Only PKG is selected.** Cache Purge is disabled while filters hide games.

**PT-BR:** Em *Configurações → Formatos de jogos detectados*, selecione
*Todos*, *Somente PKG* ou *Somente pastas*. O backport em pasta continua funcionando.

### Removable-drive cache safety (fork)

The Cache Purge operation now **refuses to remove caches when a previously indexed
source resides on a disconnected USB or extended drive**. Reconnect the drive
before using Purge. This is a safety guard, not a guarantee against unplugging
storage during an active operation. No package, backport or save is changed.

### Diagnostics for external SATA USB SSDs (fork)

Settings → Scan and cache diagnostics (PT-BR: Configurações → Diagnóstico de
varredura e cache) displays the **last scan time** and **last cache-build time**
plus title/drive and result. These values are RAM-only and reset on restart.

**These are not AMPR read latencies, in-game throughput, game FPS or USB transfer
rates.** See [L04 baseline](docs/L04-DIAGNOSTICS-BASELINE.md).

To build it yourself: `ps5/build.sh` (needs Docker), output `target/ps5/kraken-mount.elf`.

## Thanks

- [drakmor/ShadowMountPlus](https://github.com/drakmor/ShadowMountPlus): home-screen
  registration, the fakelib sandbox overlay and backports are ported from it.
- [ps5-payload-sdk](https://github.com/ps5-payload-dev/sdk): the PS5 toolchain.

## License

GPL-3.0-or-later (`LICENSE`). Third-party notices are in `THIRD_PARTY_NOTICES`.
