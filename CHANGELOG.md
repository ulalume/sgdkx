# Changelog

## 0.4.3

### Fixed

- **Generated `.vscode/c_cpp_properties.json` now resolves `<genesis.h>` in rescomp-generated
  `res/*.h` headers.** Those headers aren't compile units, so they're absent from
  `compile_commands.json` and cpptools reported `cannot open source file "genesis.h"`. Added an
  `includePath` (+ `SGDK_GCC` define) fallback at `~/.sgdkx/data/SGDK/inc` for files not covered
  by `compile_commands.json`. Home-relative paths only — stays portable. Existing projects: re-run
  `sgdkx new` or copy the new `.vscode/c_cpp_properties.json` (then reload the C/C++ extension).

### Changed

- **`sgdkx` (any no-arg run) now prints its version** in the doctor header: `🩺 sgdkx v0.4.3`.
- **Reworded the crate description / `--help` about / README** to lead with the value prop:
  *"One-command native SGDK dev environment. Unofficial, cross-platform CLI."*

## 0.4.2

### Added

- **`sgdkx new` can now debug into SGDK source from VS Code.** Next to the default
  "Debug ROM (BlastEm)" (lean `libmd.a` — debug your own code, small/fast), there is a new
  **"Debug ROM (BlastEm) + SGDK source"** launch config and a `build-debug-sgdk` task that
  rebuilds with `SGDK_DEBUG=1` (`libmd_debug.a`), so you can step into SGDK functions. Before,
  F5 always rebuilt the lean ROM, so a manual `SGDK_DEBUG=1` build got overwritten and SGDK
  source-level debugging was unreachable from the GUI.

### Changed

- Generated launch configs trimmed to the two cpptools entries with clearer labels
  (dropped the separate webfreak.debug "Native Debug" config; pick it back up by copying the
  pattern if you use that extension). Existing projects: re-run `sgdkx new` or copy the new
  `.vscode/launch.json` + `tasks.json` to get the SGDK-source config.

## 0.4.1

### Changed

- **Trimmed the generated project `Makefile` comments** — dropped self-evident notes (e.g.
  "safe to commit; no personal paths") and the redundant "plain `make` works too" hint, kept a
  concise usage block and the `make debug` tuning rationale. The build logic is unchanged.
- **Docs:** simplified the `sgdkx make` description in the README to match.

Verified on Linux (Docker, x86_64 + arm64): `install`, `new`, `make debug`, and source-level
gdb debugging via the patched BlastEm all work.

## 0.4.0

CLI cleanup toward a thin, consistent, scriptable tool.

### Breaking

- **`setup` → `install`.** The command that downloads/configures the environment is now
  `sgdkx install`, paired symmetrically with `uninstall`. It is idempotent — re-running it is
  the supported way to *update* the environment.
- **`setup-emu` removed.** BlastEm is now downloaded as part of `sgdkx install` (no separate
  command).
- **Web export removed.** The experimental `setup-web`, `web-export`, and `web-server` commands
  were removed (to be redesigned). This also drops the `tokio`/`hyper` async dependency stack.
- **Install directory unified to `~/.sgdkx/data`** on all platforms (was `~/Documents/.sgdkx/data`
  on Windows and `~/.config/sgdkx/data` on Linux). Existing Windows/Linux installs are orphaned —
  re-run `sgdkx install`.

### Added

- **`sgdkx compile-commands [-p/--path <dir>]`** — regenerate `compile_commands.json` after
  adding/removing sources (parses a `make -nwB` dry-run; no external `compiledb`).
- **Version selection for `install`:** `-s/--sgdk <ver>` and `-b/--blastem <ver>`. When omitted,
  an interactive picker opens on a terminal; the latest is used when non-interactive.

### Changed

- **Non-interactive friendly (TTY-aware):**
  - `sgdkx new <name>` gains `-t/--template <path>` — interactive pick on a terminal, required
    (errors instead of hanging) when non-interactive.
  - `sgdkx uninstall` gains `-y/--yes` — required (errors instead of hanging) when non-interactive.
- Dependencies updated (toml_edit 0.25, reqwest 0.13, zip 8, dialoguer 0.12, + `cargo update`);
  removed unused `sevenz-rust`, `regex`, and `serde`.
- **The generated project `Makefile` is now portable and committed** (no machine-specific paths;
  removed from the project `.gitignore`). `sgdkx make` exports `GDK` + the toolchain `PATH`; the
  `Makefile` is just `GDK ?= $(HOME)/.sgdkx/data/SGDK` + `include $(GDK)/makefile.gen`. This also
  lets a cloned project build without re-scaffolding.
- **Corrected generated IDE configs:** `.vscode/c_cpp_properties.json` uses `cStandard: gnu17`
  (gcc's default; SGDK uses GNU/MS extensions) and `intelliSenseMode: gcc-x86` (m68k is ILP32);
  `.clangd` uses `-std=gnu17`.

### System requirements

- No Gens, no Wine, no system `compiledb`, no system Java, no system gcc. On Unix only `make` is
  required; on Windows nothing (all bundled). `git`/`doxygen` are no longer needed.

## 0.3.0

### Added

- **`sgdkx gdb [args...]`** — thin pass-through to `m68k-elf-gdb`. gdb is downloaded per platform
  by `install`.

### Changed

- `--version` now tracks the crate version automatically.
