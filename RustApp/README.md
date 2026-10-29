# Rust desktop App

## Deps

Fedora

```shell
sudo dnf install alsa-lib-devel jack-audio-connection-kit-devel clang pipewire-devel libxkbcommon-devel protobuf-compiler
```

Debian

```shell
sudo apt install libasound2-dev
```

## Adb

You probably need to install [adb](https://developer.android.com/tools/releases/platform-tools).

## Usb

- https://www.reddit.com/r/scrcpy/comments/1ga2nli/solution_get_scrcpy_otg_working_on_windows_a/
- https://github.com/pbatard/libwdi/releases

- Your changing the driver of your Android device, be careful!

- disable ADB (to not change the driver when adb is in use, but if you always use adb, maybe don't disable it)
- select "Options", "List All Devices"
- select your device
- select the WinUSB driver
- click "Reinstall Driver"
- it can take a long time

then, if it still don't work, you can make the same process when your phone is in accessory mode
