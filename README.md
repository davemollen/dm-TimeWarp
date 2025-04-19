## dm-TimeWarp

A granular effect written in Rust.
The effect can be compiled to a [Mod devices](https://moddevices.com/), lv2, vst3, CLAP or vst plugin.

## Table of contents:

- [Mod devices installation](#Mod-devices-installation)
- [Other plugin format installation](#Other-plugin-format-installation)
  - [LV2 installation](#LV2-installation)
  - [VST3 installation](#VST3-installation)
  - [CLAP installation](#CLAP-installation)
  - [VST installation](#VST-installation)

## Mod devices installation

You can find the plugin for the Mod Dwarf [here](./lv2/dm-TimeWarp.lv2/).

To build the plugin for your MOD device see [these instructions](https://github.com/moddevices/mod-plugin-builder).

## Other plugin format installation

A prerequisite to install any plugin is to have Rust installed on your machine.
Follow [these instructions](https://www.rust-lang.org/tools/install) to install Rust.

Below you can find the additional instructions per plugin format. These instructions might not be complete. Please let me know if anything's missing.

### LV2 installation

Go into the lv2 directory and run the `cargo build --release` command.
Once finished, copy the compiled plugin from [/target/release](./lv2/target/release) into your plugin folder.

### VST3 installation

If you want to compile the vst3 plugin for mac you should run `./scripts/build-vst3-for-mac.sh`. This will compile and copy the plugin to the default plugin folder.

If you want to compile the vst3 plugin on other systems you should go into the nih-plug directory and run `cargo xtask bundle dm_time_warp --release`.
Once finished, copy the compiled plugin from [/target/bundled](./nih-plug/target/bundled) into your plugin folder.

### CLAP installation

If you want to compile the CLAP plugin you should go into the nih-plug directory and run `cargo xtask bundle dm_time_warp --release`.
Once finished, copy the compiled plugin from [/target/bundled](./nih-plug/target/bundled) into your plugin folder.

### VST installation

The following scripts can be used to compile the plugin for mac. Prefix the command with `sudo ` if you get a `Permission denied` error.

- For intel macs:

  Running the `./scripts/build-vst-for-mac.sh` command will compile and copy the plugin to the default plugin folder.

- For M1 macs:

  Running the `./scripts/build-vst-for-mac-m1.sh` command will compile and copy the plugin to the default plugin folder.

If you want to compile the vst plugin on other systems you should go into the vst directory and run `cargo build --release`.
Once finished, copy the compiled plugin from [/target/release](./vst/target/release) into your plugin folder.
