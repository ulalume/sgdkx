# SGDKTool

🇯🇵 日本語版 README は[こちら](./README.ja.md)をご覧ください。

**This tool is under active development. There is a significant lack of tests. Use at your own risk!**

SGDKTool is a CLI tool to support development with SGDK (Sega Genesis Development Kit).

---

## 1. Installation

### Install SGDKTool (via cargo)

```sh
cargo install --git https://github.com/ulalume/sgdktool
```

### Required Tools (macOS)

The following tools are required. You can install them with Homebrew:

```sh
brew install make openjdk compiledb

brew tap gcenx/wine
brew install --cask --no-quarantine wine-crossover

brew install doxygen # options
```

- `git` is usually pre-installed, but if not, install it with `brew install git`.
- Running `sgdktool` with no command will perform an environment check and show if all required tools are installed.

---

## 2. Usage

Main commands:

- `sgdktool`<br>
  Show environment check, SGDK/emulator configuration, and help message.

- `sgdktool setup [--version <version>]` <br>
  Download and install SGDK (Sega Genesis Development Kit).<br>
  You can specify the installation directory with `--dir` (default: config directory), and the version with `--version` (default: master).<br>
  The `--version` option accepts a branch name, tag, or commit ID.<br>
  Examples:<br>

  - `--version V2.11` for tag V2.11
  - `--version ef9292c0` for commit ID ef9292c0
    The SGDK path and version are saved in config.toml.<br>
    Additionally, **if doxygen is installed and SGDK documentation does not exist, documentation will be generated automatically.**

- `sgdktool setup-emu [gens|blastem]`<br>
  Download and install an emulator (Gens or BlastEm).<br>
  You can specify the installation directory with `--dir` (default: config directory).<br>
  The path to the emulator is saved in config.toml.

- `sgdktool new <project_name>`<br>
  Create a new project from the SGDK sample.

- `sgdktool run [--emulator <gens|blastem>] [--rom <path>]`<br>
  Run the ROM file with the specified emulator (default: gens or installed emulator).<br>
  You can specify the emulator with `--emulator` and the ROM file path with `--rom` (default: `out/rom.bin`).

- `sgdktool uninstall`<br>
  Uninstall SGDK, remove configuration, and also delete any emulators (Gens/BlastEm) installed via `setup-emu` at the paths recorded in config.toml.

- `sgdktool doc`<br>
  If SGDK documentation exists, it will be opened in your browser.

---

#### Experimental Features

- `sgdktool web-export [--rom <path>] [--dir <parent-dir>]`<br>
  **[Experimental]** Export your ROM and a web emulator template for browser-based play.<br>
  This command copies a web emulator template (HTML/JS/WASM) and your ROM into a new `web-export` directory under the specified parent directory (default: current directory).<br>
  You can then serve this directory to play your game in a browser.

- `sgdktool web-server [--dir <directory>] [--port <port>]`<br>
  **[Experimental]** Serve the `web-export` directory with a built-in HTTP server (with COOP/COEP headers for WASM compatibility).<br>
  By default, serves the `web-export` directory on `localhost:8080`. You can change the directory and port with options.<br>
  Example: `sgdktool web-server --dir web-export --port 9000`

### Simple Example

```sh
sgdktool setup --version v2.11 # stable
sgdktool setup-emu
sgdktool new your_project
cd your_project
make
sgdktool run
```

### Reference: Output when run without any command

```
A CLI tool for SGDK-based development

Usage: sgdktool [COMMAND]

Commands:
  setup       Setup SGDK for development
  doc         Show SGDK documentation status
  setup-emu   Setup emulator for running ROM files
  new         Create a new SGDK project
  run         Run ROM file with emulator
  uninstall   Uninstall SGDK installation and configuration
  web-export  Export ROM and web emulator template for web deployment
  web-server  Serve web-export directory with HTTP server (with COOP/COEP headers)
  open        Open SGDK installation directory
  setup-web   Setup web export template
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

🩺 SGDKTool Environment Check
✅ git: /opt/homebrew/bin/git
✅ make: /usr/bin/make
✅ java: /opt/homebrew/opt/openjdk/bin/java
✅ compiledb: /opt/homebrew/bin/compiledb
✅ doxygen: /opt/homebrew/bin/doxygen
✅ wine: /opt/homebrew/bin/wine

📝 SGDKTool Configuration: /Users/[user]/.sgdktool/data/config.toml
SGDK Path   : /Users/[user]/.sgdktool/data/SGDK
Version     : v2.11
Commit ID   : ef9292c03fe33a2f8af3a2589ab856a53dcef35c
Gens Path   : /Users/[user]/.sgdktool/data/gens/gens.exe
blastem Path: Not installed

📄 SGDK documentation: /Users/[user]/.sgdktool/data/SGDK/doc/html/index.html
```

---

## 3. Acknowledgements / Dependencies

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)
- [jgenesis (by James Groth)](https://github.com/jsgroth/jgenesis)

Special thanks to these excellent projects.

---
