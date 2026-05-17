# ADB TCP Service

A setup to provide ADB over TCP (WiFi) hosted through a [`pinitd`](https://github.com/PenumbraOS/pinitd/) service. Installing this APK and unit file will register the `adbd-service` in `pinitd`. Enabling and starting it will open a customized [`adbd_wifi`](https://github.com/agg23/adbd_wifi) server at `localhost:5555`, which can be used via `adb connect [IP]:5555`.
