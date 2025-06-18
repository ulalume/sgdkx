# SGDKTool

🇯🇵 日本語版READMEは[こちら](./README.ja.md)をご覧ください。

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
```

- `git` is usually pre-installed, but if not, install it with `brew install git`.
- Running `sgdktool` with no command will perform an environment check and show if all required tools are installed.

---

## 2. Usage

Main commands:

- `sgdktool`  
  Show environment check, SGDK/emulator configuration, and help message.  
  If no subcommand is given, displays the current SGDK and emulator setup status.

- `sgdktool setup [--dir <path>] [--branch <branch>]`  
  Download and install SGDK (Sega Genesis Development Kit).  
  You can specify the installation directory with `--dir` (default: config directory), and the branch with `--branch` (default: master).  
  The SGDK path and branch are saved in config.toml.

- `sgdktool setup-emu [gens|blastem] [--dir <path>]`  
  Download and install an emulator (Gens or BlastEm).  
  You can specify the installation directory with `--dir` (default: config directory).  
  The path to the emulator is saved in config.toml.

- `sgdktool new <project_name>`  
  Create a new project from the SGDK template.  
  The project will be created in a new directory named `<project_name>`.

- `sgdktool make [--project <dir>] [<extra options>...]`  
  Build the SGDK project using `make`.  
  You can specify the project directory with `--project` (default: current directory), and pass extra options to `make`.

- `sgdktool run [gens|blastem] [--rom <path>]`  
  Run the ROM file with the specified emulator (default: gens or installed emulator).  
  You can specify the emulator and ROM file path (default: auto-detect/`out/rom.bin`).  
  If the emulator is not installed, a message will prompt you to run setup-emu.

- `sgdktool uninstall [--config-only]`  
  Uninstall SGDK, remove configuration, and also delete any emulators (Gens/BlastEm) installed via `setup-emu` at the paths recorded in config.toml.  
  If `--config-only` is specified, only the configuration file is removed and SGDK itself is kept.

### Simple Example

```sh
sgdktool setup
sgdktool setup-emu
sgdktool new your_project
cd your_project
sgdktool make
sgdktool run
```

### Example: Output when run without any command

```
A CLI tool for SGDK-based development

Usage: sgdktool [COMMAND]

Commands:
  setup      Setup SGDK (clone and register path)
  setup-emu  Setup emulator for running ROM files
  new        Create new project from SGDK template
  make       Build project using make
  run        Run ROM file with emulator
  uninstall  Uninstall SGDK installation and configuration
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

🩺 SGDKTool Environment Check
✅ git: /opt/homebrew/bin/git
✅ make: /usr/bin/make
✅ java: /opt/homebrew/opt/openjdk/bin/java
✅ compiledb: /opt/homebrew/bin/compiledb
✅ wine: /opt/homebrew/bin/wine

📝 SGDK Configuration Info:
SGDK Path   : /Users/[user]/Library/Application Support/sgdktool/SGDK
Branch      : master
Commit ID   : 60c99ea912387d6f5f014673d9760ef8a79e1339
Gens Path   : /Users/[user]/Library/Application Support/sgdktool/gens/gens.exe
blastem Path: /Users/[user]/Library/Application Support/sgdktool/blastem/blastem-win64-0.6.3-pre-b42f00a3a937/blastem.exe
```

---

## 3. Acknowledgements / Dependencies

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)

Special thanks to these excellent projects.

---
