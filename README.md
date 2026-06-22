# sgdkx

`sgdkx` is a native, cross-platform CLI for developing with [SGDK](https://github.com/Stephane-D/SGDK) (Sega Mega Drive / Genesis Development Kit).

It downloads and manages a **self-contained** SGDK environment — SGDK, the m68k gcc toolchain, a bundled JRE, m68k-elf-gdb, and the BlastEm emulator.

## Installation

```sh
cargo install sgdkx
```

To update sgdkx itself, run `cargo install sgdkx` again (or `cargo install-update -a` with [cargo-update](https://crates.io/crates/cargo-update)).

### Requirements

- **macOS / Linux:** `make` (e.g. `brew install make`, `apt install make`). Everything else is downloaded by `sgdkx install`.
- **Windows:** nothing.

Run `sgdkx` with no command for an environment check (`doctor`).

## Quick start

```sh
sgdkx install                  # download the environment (interactive version pick on a terminal)
sgdkx new mygame               # scaffold a project (interactive template pick on a terminal)
cd mygame
sgdkx make                     # build -> out/rom.bin
sgdkx blastem out/rom.bin      # run in BlastEm
```

## Commands

| Command                                                | Description                                                                                                                                                                                         |
| ------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `sgdkx install [-s/--sgdk <ver>] [-b/--blastem <ver>]` | Install/update the environment (SGDK, toolchain, JRE, gdb, BlastEm). Idempotent — re-run to update. Omitted versions prompt on a terminal, use latest when non-interactive.                         |
| `sgdkx new <name> [-t/--template <path>]`              | Scaffold a project from an SGDK sample (e.g. `basics/hello-world`). Prompts for a template on a terminal; `--template` is required when non-interactive.                                            |
| `sgdkx make [args...]`                                 | Thin wrapper around `make` (args passed straight through, e.g. `debug`, `clean`). Sets `GDK` and prepends the SGDK build tools to `PATH`. |
| `sgdkx blastem [args...]`                              | Run the bundled BlastEm (e.g. `sgdkx blastem out/rom.bin`).                                                                                                                                         |
| `sgdkx gdb [args...]`                                  | Run `m68k-elf-gdb` (args passed straight through, e.g. `sgdkx gdb out/rom.out`).                                                                                                                    |
| `sgdkx compile-commands [-p/--path <dir>]`             | Regenerate `compile_commands.json` (for clangd / IDEs) after adding or removing source files.                                                                                                       |
| `sgdkx doc`                                            | Open the SGDK documentation in your browser.                                                                                                                                                        |
| `sgdkx open`                                           | Open the installation directory.                                                                                                                                                                    |
| `sgdkx uninstall [-y/--yes]`                           | Remove the environment and configuration. `--yes` skips the confirmation (required when non-interactive).                                                                                           |
| `sgdkx`                                                | Environment check + configuration (the `doctor` default).                                                                                                                                           |

`compile_commands.json` is generated automatically by `sgdkx new`; run `sgdkx compile-commands` to refresh it later (it parses `make -nwB` output — no external `compiledb`).

The environment and `config.toml` live under `~/.sgdkx/data` (the same on macOS, Linux, and Windows; shown by `sgdkx` / `sgdkx open`).

## Acknowledgements

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [BlastEm (by Michael Pavone)](https://www.retrodev.com/blastem/)

## Notes

- This tool is under active development.
- **Breaking change in 0.4.0:** `setup` → `install`; `setup-emu` folded into `install`; the experimental `setup-web` / `web-export` / `web-server` commands were removed.
