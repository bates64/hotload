# hotload64 with libdragon

This is an example of how to use hotload64 with libdragon.

## Setup

Download the toolchain with `npm install` and make sure you have hotload64 installed.

## Watch

```
hotload64 --build "npm build" --elf build/hello.elf --src src --emulator "ares hello.z64"
```
