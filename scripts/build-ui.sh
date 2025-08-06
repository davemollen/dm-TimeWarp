#!/bin/bash
package_name="dm_time_warp"
move_from="./target/bundled/$package_name.vst3"
move_to="/Library/Audio/Plug-Ins/VST3/dm-TimeWarp.vst3"

cd nih-plug
cargo xtask bundle $package_name --release
open target/bundled/dm_time_warp.app