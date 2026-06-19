# sgdkx

`sgdkx` は、[SGDK](https://github.com/Stephane-D/SGDK)（Sega Mega Drive / Genesis Development Kit）開発を支援するネイティブなクロスプラットフォーム CLI です。

SGDK・m68k gcc ツールチェーン・同梱 JRE・m68k-elf-gdb・BlastEm エミュレータを **自己完結** した環境としてユーザーごとのディレクトリにダウンロード・管理します。システムの gcc / Java / Wine / compiledb は不要です。

## インストール

```sh
cargo install sgdkx
```

sgdkx 自体の更新は、再度 `cargo install sgdkx` を実行してください（または [cargo-update](https://crates.io/crates/cargo-update) で `cargo install-update -a`）。

### 必要なもの

- **macOS / Linux:** `make`（例: `brew install make` / `apt install make`）。それ以外は `sgdkx install` がダウンロードします。
- **Windows:** 不要 — `make`・ツールチェーン・シェルはすべてダウンロードされる SGDK バンドルに含まれます。

引数なしで `sgdkx` を実行すると環境チェック（`doctor`）を表示します。

## クイックスタート

```sh
sgdkx install                  # 環境をダウンロード（端末ではバージョンを対話選択）
sgdkx new mygame               # プロジェクト雛形を生成（端末ではテンプレートを対話選択）
cd mygame
sgdkx make                     # ビルド -> out/rom.bin
sgdkx blastem out/rom.bin      # BlastEm で実行
```

非対話・スクリプト（CI）向け:

```sh
sgdkx install --sgdk v2.11 --blastem latest
sgdkx new mygame --template basics/hello-world
sgdkx make
```

## コマンド一覧

| コマンド | 説明 |
|---|---|
| `sgdkx install [-s/--sgdk <ver>] [-b/--blastem <ver>]` | 環境のインストール/更新（SGDK・ツールチェーン・JRE・gdb・BlastEm）。冪等で、再実行が更新になります。バージョン未指定時は端末で対話選択、非対話では最新。 |
| `sgdkx new <name> [-t/--template <path>]` | SGDK サンプル（例 `basics/hello-world`）から雛形生成。端末ではテンプレートを対話選択、非対話では `--template` が必須。 |
| `sgdkx make [args...]` | `make` の薄いラッパー（引数はそのまま渡す。例 `debug`・`clean`）。`GDK` とツールチェーン `PATH` を export するため、生成される `Makefile` はパス非依存で**コミット可能**。 |
| `sgdkx blastem [args...]` | 同梱 BlastEm を実行（例 `sgdkx blastem out/rom.bin`）。 |
| `sgdkx gdb [args...]` | `m68k-elf-gdb` を実行（例 `sgdkx gdb out/rom.out` の後、BlastEm の gdb スタブへ `target remote :1234`）。 |
| `sgdkx compile-commands [-p/--path <dir>]` | ソース構成変更後に `compile_commands.json`（clangd / IDE 用）を再生成。 |
| `sgdkx doc` | SGDK ドキュメントをブラウザで開く。 |
| `sgdkx open` | インストールディレクトリを開く。 |
| `sgdkx uninstall [-y/--yes]` | 環境と設定を削除。`--yes` で確認を省略（非対話では必須）。 |
| `sgdkx` | 環境チェック + 設定表示（`doctor` の既定動作）。 |

`compile_commands.json` は `sgdkx new` 時に自動生成されます。後から `sgdkx compile-commands` で再生成できます（`make -nwB` のドライランを解析。外部 `compiledb` は不要）。

環境と `config.toml` は `~/.sgdkx/data` 配下にあります（macOS / Linux / Windows で共通。`sgdkx` / `sgdkx open` で確認可能）。

## 謝辞

- [SGDK (by Stephane-D)](https://github.com/Stephane-D/SGDK)
- [BlastEm (by Michael Pavone)](https://www.retrodev.com/blastem/)

## 備考

- 本ツールは活発に開発中です。
- **0.3.0 の破壊的変更:** `setup` → `install`、`setup-emu` を `install` に統合、実験的な `setup-web` / `web-export` / `web-server` を削除しました。詳細は [CHANGELOG.md](./CHANGELOG.md) を参照。
