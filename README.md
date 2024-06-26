# hotload

[![CI](https://github.com/bates64/hotload/actions/workflows/ci.yml/badge.svg)](https://github.com/bates64/hotload/actions/?workflow=CI)
[![Releases](https://img.shields.io/github/v/tag/bates64/hotload&label=release)](https://github.com/bates64/hotload/releases)
[![Nix flake](https://img.shields.io/badge/flake-github%3Abates64%2Fhotload-blue?logo=nixos&labelColor=white)](https://nixos.wiki/wiki/Flakes)

`hotload` implements **function-level** [dynamic software updating](https://wikipedia.org/wiki/Dynamic_software_updating), or "hot code (re)loading" for arbitrary programs.

## Use cases

`hotload` allows you to make source code changes and see the results without having to restart the program or lose any state.

This is useful in cases such as:

- You are developing a cutscene in a game, and you want to see the changes immediately without having to restart the game, navigate to the cutscene, and trigger it.
- You are developing a server application, and you want to avoid downtime when deploying new code.

`hotload` is not suitable for architectures that do not support code modification at runtime through a debugger. This includes frontend web applications. For this purpose, try [hot module replacement](https://vitejs.dev/guide/api-hmr.html).

## What it does

`hotload` is a command-line tool that:

1. Builds a program (using a user-provided build command such as `make`)
2. Starts the program
3. Watches the program's source code for changes, and when it detects a change:
    1. Rebuilds the program
    2. Loads the new code into the already-running program

The focus for `hotload` is the Nintendo 64 target, but its ideas are portable to other targets. If you are developing for another target and would like to see support for it, please [open an issue](https://github.com/bates64/hotload/issues/new). You may want to consider the following alternatives:
- Dynamic linking, and reloading the library when it changes
  * This is not possible on embedded systems such as the Nintendo 64.
- Embed a scripting language such as Lua and develop a system to reload scripts when they change
- [Use an Erlang VM language such as Elixir](https://www.elixirwiki.com/wiki/Erlang_Hot_Code_Reloading)

All of the alternatives listed above require changes to the program source code, some more than others. `hotload` requires **no changes to source code**. In fact, it does not require source code at all, although the use cases for this situation are less clear.

## Requirements

- An ELF executable with mips32 or mips64 architecture, unstripped (with symbols).
- An emulator or game which implements the [GDB Remote Serial Protocol](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Remote-Protocol.html).
    - The [ares](https://ares-emu.net) emulator supports GDB for Nintendo 64.
    - Games can implement GDB over flashcart serial ports. [Video tutorial](https://www.youtube.com/watch?v=Fr8rSqsFuWk).

## Usage

With [nix](https://zero-to-nix.com/),

```shell
nix run github:bates64/hotload [OPTIONS] --build <BUILD> --elf <ELF> --emulator <EMULATOR>
```

```
Options:
  -b, --build <BUILD>        Build system command to run (e.g. `make`, `ninja`, `libdragon build`)
  -e, --elf <ELF>            ELF file that is output from build command
  -s, --src <SRC>            Source files and/or directories to recursively watch for changes
  -x, --emulator <EMULATOR>  Emulator command (e.g. `ares rom.z64`)
  -h, --help                 Print help
  -V, --version              Print version
```
