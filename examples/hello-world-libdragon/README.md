# hotload with libdragon

This is an example of how to use hotload with libdragon, an open-source toolchain for developing N64 homebrew.

## Setup

Download the toolchain with `npm install` and make sure you have hotload installed.

## Watch

```
hotload --build "npm build" --elf build/hello.elf --src src --emulator "ares hello.z64"
```
