create-dmg \
--volname "Etcd Workbench" \
--volicon "./icons/macos/icon.icns" \
--background "./icons/macos/dmg-bg.png" \
--window-pos 200 200 \
--window-size 900 600 \
--icon-size 128 \
--icon "Etcd Workbench" 232 338 \
--hide-extension "Etcd Workbench.app" \
--app-drop-link 688 338 \
"./target/release/bundle/macos/Etcd Workbench.dmg" \
"./target/release/bundle/macos/Etcd Workbench.app"
