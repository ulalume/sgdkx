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

- `sgdktool` : Environment check and help
- `sgdktool setup` : Setup SGDK (clone and register path)
- `sgdktool new` : Create a new project from SGDK template
- `sgdktool make` : Build SGDK project
- `sgdktool uninstall` : Uninstall SGDK and remove configuration

### Simple Example

```sh
sgdktool setup
sgdktool new your_project
cd your_project
sgdktool make
```

### Example: Output when run without any command

```
A CLI tool for SGDK-based development

Usage: sgdktool [COMMAND]

Commands:
  setup      Setup SGDK (clone and register path)
  new        Create new project from SGDK template
  make       Build project using make
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
SGDK Path   : /Users/[name]/Library/Application Support/sgdktool/SGDK
Branch      : master
Commit ID   : 60c99ea912387d6f5f014673d9760ef8a79e1339
```

---

## 3. Acknowledgements / Dependencies

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [SGDK_wine (by Franticware)](https://github.com/Franticware/SGDK_wine)

Special thanks to these excellent projects.

---
