# PenumbraOS (xiaoyu-work fork)

Personal fork of [`PenumbraOS/pinitd`](https://github.com/PenumbraOS/pinitd) —
the rootless init system for the Humane Ai Pin — bundled with snapshots of
the rest of the PenumbraOS ecosystem so the whole stack lives in one repo
and can be rewritten as needed.

## Layout

```
.
├── pinitd/                  pinitd daemon (Rust)
├── pinitd-cli/              pinitd CLI control utility
├── pinitd-common/           shared types
├── android/                 the helper APK that pinitd ships with
├── build.sh                 build everything for the Pin
└── components/              other PenumbraOS projects (snapshots)
    ├── sdk/                 Bridge services (Core / Settings / Shell / System)
    ├── mabl/                Modular Assistant + plugins (OpenAI, Gemini, …)
    ├── installer/           Tauri + CLI installer + penumbra.yml manifest
    ├── adbd/                Custom TCP adbd service
    ├── adb_remote_auth/     ADB client with remote auth
    ├── adb_signer/          Companion ADB token signer
    ├── system-injector/     Promote regular APKs to system apps
    ├── humane-system-hook/  Runtime hook to keep original Humane software working
    ├── app_process-mocks/   Android Context mocks for app_process JVM services
    ├── ai_pin_logger-rs/    Rust `log` backend that pipes to Android logcat
    ├── center/              Web dashboard + WebUSB installer UI
    └── docs/                Humane Ai Pin reverse-engineering notes
```

Each `components/<name>/` is a snapshot of the upstream
`https://github.com/PenumbraOS/<name>` repo at clone time, with the `.git`
and `.github` directories removed so the source becomes part of this fork.

## Upstream

| Component             | Upstream                                                       |
| --------------------- | -------------------------------------------------------------- |
| pinitd (root)         | https://github.com/PenumbraOS/pinitd                           |
| components/sdk        | https://github.com/PenumbraOS/sdk                              |
| components/mabl       | https://github.com/PenumbraOS/mabl                             |
| components/installer  | https://github.com/PenumbraOS/installer                        |
| components/adbd       | https://github.com/PenumbraOS/adbd                             |
| components/adb_remote_auth | https://github.com/PenumbraOS/adb_remote_auth             |
| components/adb_signer | https://github.com/PenumbraOS/adb_signer                       |
| components/system-injector | https://github.com/PenumbraOS/system-injector             |
| components/humane-system-hook | https://github.com/PenumbraOS/humane-system-hook       |
| components/app_process-mocks | https://github.com/PenumbraOS/app_process-mocks         |
| components/ai_pin_logger-rs | https://github.com/PenumbraOS/ai_pin_logger-rs           |
| components/center     | https://github.com/PenumbraOS/center                           |
| components/docs       | https://github.com/PenumbraOS/docs                             |

The `upstream` git remote on this repo still tracks
`PenumbraOS/pinitd` (root only) for pulling future fixes to pinitd itself.

## Goal

Rewrite the OS layer (pinitd + sdk + humane-system-hook + system-injector)
so a Pin boots into a stack that I control end-to-end.

## Original upstream READMEs

See each `components/<name>/README.md` for the upstream project's own docs.
