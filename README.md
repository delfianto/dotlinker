# dotlinker

> *Because copying config files by hand is a lifestyle choice, not a workflow.*

**dotlinker** (`dot-rs` at the cargo altar) is a small Rust tool that turns a pile of
module directories and a YAML file into a forest of symlinks. It is not a package
manager, not a framework, and not a “dotfiles platform.” It is a polite script that
learned to compile.

If you have ever:

- typed `ln -s` seventeen times and still typo’d the path,
- re-cloned your rice and wondered why half of `~/.config` is a haunted mirror,
- or opened someone else’s stow/chezmoi setup and felt your soul leave your body,

then congratulations. You are the target audience. You may sit down.

---

## What it actually does

You keep a **dotfiles root**. Inside it, each **module** is a directory. Each module
owns a `mod.yaml` that lists what should be linked into XDG-ish base dirs under your
home. You point the binary at that root, name the modules, and it creates symlinks.

That is the product. No plugins. No cloud sync. No “profile system.” Just paths and
hubris.

```
your-dotfiles/
├── hypr/
│   ├── mod.yaml
│   └── hypr/
│       └── hyprland.conf
├── nvim/
│   ├── mod.yaml
│   └── nvim/
│       └── init.lua
└── ...
```

Run it. Watch logs pretend everything is fine. Move on with your life.

---

## Install

You need a recent Rust toolchain. Edition is `2024`. If `cargo` refuses to talk to you,
that is a you problem.

```bash
git clone https://github.com/delfianto/dotlinker.git
cd dotlinker
cargo install --path .
```

Binary name: **`dot-rs`**. Repo name: **dotlinker**. Package name: **dot-rs**.
Consistency is for people who write design docs.

---

## Quick start (the “I trust nothing” edition)

```bash
# pretend first — always pretend first
dot-rs --dry-run --verbose --dotfiles-dir ~/dotfiles hypr nvim

# when the dry run stops insulting you
dot-rs --dotfiles-dir ~/dotfiles hypr nvim

# when an old file is in the way and you want drama
dot-rs --force --dotfiles-dir ~/dotfiles hypr
```

Default `--dotfiles-dir` is the **current working directory**, despite what the help
text mumbles about “parent of this script.” Believe the code, not the marketing.

---

## CLI

```
dot-rs [OPTIONS] [MODULES]...
```

| Flag | What it claims to do | What it actually does |
|------|----------------------|------------------------|
| `MODULES...` | Module names to process | Looks for `<dotfiles-dir>/<module>/mod.yaml` |
| `--dry-run` | No linking | Logs what *would* happen; filesystem remains innocent |
| `-v`, `--verbose` | More output | More output. Bring snacks. |
| `--relative` | Relative symlinks | Relative symlinks. Absolute is default, like a normal person. |
| `--force` | Overwrite | Backs up the existing path, then overwrites. See below. |
| `--dotfiles-dir PATH` | Dotfiles root | Defaults to `cwd` if omitted |

Failed modules are logged and skipped so one cursed package does not halt the
entire rice. Failed *entries* inside a module get the same mercy. This is either
resilient design or cowardice. We do not pick.

---

## Module format (`mod.yaml`)

Root key is **`dot_rs`**. Not `mod_info`. Not `packages`. Not your feelings.
(`mod.sample.yml` in the repo is a charming historical document from a timeline
where the schema was different. Do not copy it. It will hurt you.)

### Base directory keys

| YAML key | Resolves to |
|----------|-------------|
| `home` | `$HOME` |
| `config_home` | `$HOME/.config` |
| `cache_home` | `$HOME/.cache` |
| `data_home` | `$HOME/.local/share` |
| `local_bin` | `$HOME/.local/bin` |
| `local_dir` | `$HOME/.local` |

These are hardcoded XDG-ish paths. Environment variables like `$XDG_CONFIG_HOME`
are not consulted. Your custom XDG layout can write a strongly worded letter.

### Example

Module directory: `~/dotfiles/hypr/`

```yaml
dot_rs:
  config_home:
    - hypr
    - dunst
  home:
    - :.Xresources
    - .docker/config.json:docker/config.json
  local_bin:
    - scripts/my-tool:my-tool
```

Rough translation:

| Entry | Source (in module) | Destination |
|-------|--------------------|-------------|
| `hypr` | `.../hypr/hypr` | `~/.config/hypr` |
| `dunst` | `.../hypr/dunst` | `~/.config/dunst` |
| `:.Xresources` | `.../hypr/.Xresources` | `~/.Xresources` |
| `.docker/config.json:docker/config.json` | `.../hypr/docker/config.json` | `~/.docker/config.json` |
| `scripts/my-tool:my-tool` | `.../hypr/scripts/my-tool` | `~/.local/bin/my-tool` |

Source files must exist. The tool will not invent configs for you. It is a linker,
not a therapist.

---

## Entry syntax (the fun part)

Each list item is a string. Parsing rules, in order of increasing regret:

### Simple path

```yaml
- dunst
```

- Source: `<module_dir>/dunst`
- Dest: `<base_dir>/dunst`

### Leading colon (optional cosplay)

```yaml
- :dunst
- :.Xresources
```

A leading `:` is stripped and ignored. It is decorative. Someone, somewhere,
thought it looked cool. It does.

### Mapped entry (`source:destination`)

```yaml
- link_name:dest_name
- docker/config.json:.docker/config.json
```

- Source: `<module_dir>/<left>`
- Dest: `<base_dir>/<right>`

Note: if you also used a *leading* colon, that is stripped first. After that, the
first remaining `:` splits map left/right. Yes, this means `:.foo:bar` is a mapped
entry for real. No, we will not be taking questions.

### Module name via `$`

```yaml
- $
- $:hyprland
```

`$` alone becomes the module name (directory name under the dotfiles root).
A leading `$` in a segment is replaced with the module name. Handy when the folder
and the install name match and you refuse to type it twice.

### Leading asterisk

```yaml
- *special
```

`*` is stripped. Historical residue. Treat it like lint on a sweater.

---

## Conflict handling

| Destination state | Without `--force` | With `--force` |
|-------------------|-------------------|----------------|
| Missing | Create parent dirs + symlink | Same |
| Correct symlink already | Skip, claim victory | Same |
| Wrong symlink / file / dir | Warn and skip | Backup, remove, link |
| Broken symlink | Remove, then link | Same |

Backups use a timestamped path next to the victim. The tool will not ask for
confirmation. `--force` means you said “yes” with your whole chest.

---

## Dry runs and absolute vs relative

- **`--dry-run`**: full parse, full logging, zero filesystem writes for links.
  Ideal for CI, demos, and people who have been burned before.
- **Absolute links** (default): destination points at the real absolute source path.
- **`--relative`**: destination points at a relative path from dest → source.
  Useful if you like relocating trees and living dangerously.

---

## Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

CI does the same four rituals on `main` and PRs: fmt, clippy, build, test. Renovate
automerge minor/patch updates so you can pretend maintenance is free. Major and
0.x-landmine crates are held at arm’s length, as they should be.

---

## Non-goals

- Windows support as a first-class citizen (symlinks on Windows are a whole genre)
- Secret management
- Templating engines
- “Multi-machine profiles with inheritance”
- Replacing your shell, your editor, or your personality

If you need those, other tools exist. Some of them even have more stars.

---

## License

MIT. See [LICENSE](LICENSE). Copyright line says *“What? WHO?”* which remains the
correct reaction to most software.

---

## Philosophy

Symlinks are a confession: *this file lives over there, I just want it to look like
it lives here.* dotlinker automates that confession across modules so you can
rebuild a machine without reopening seventeen browser tabs titled “best way to
manage dotfiles 2019.”

Keep your modules small. Keep your YAML honest. Run `--dry-run` like you mean it.
And if something breaks, check whether the source path actually exists before you
blame the language.

Rust didn’t make your config better. It just made the linker faster while it
judges you.
