package_name="dm_grain_stretch"
move_from="./target/bundled/$package_name.vst3"
move_to="/Library/Audio/Plug-Ins/VST3/dm-GrainStretch.vst3"

cd nih-plug
cargo xtask bundle $package_name --release

if [ -d "$move_to" ]; then
    rm -r "$move_to"
fi

if mv "$move_from" "$move_to"; then
    echo "Copied VST3 bundle to $move_to"
fi