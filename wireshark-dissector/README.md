wireshark-dissector
===================

1. Run `cargo build --release` to get a `libwireshark_dissector.so` (Linux),
   `libwireshark_dissector.dylib` (macOS) or `wireshark_dissector.dll`
   (Windows). On macOS, rename it to `libwireshark_dissector.so`, else
   Wireshark will not recognize it as a plugin.

2. Place the above mentioned file into your plugin folder, on Linux and macOS,
   it's `~/.local/lib/wireshark/plugins/4.0/epan/`, on Windows it's
   `%APPDATA%\Wireshark\plugins\4.0\epan\`. You'll likely have to create these
   folders.

3. Start Wireshark, go to Help → About Wireshark, click on the Plugins tab. You
   should see the previously copied file in the list.

4. Start capturing Teeworlds traffic. It should automatically get dissected.
