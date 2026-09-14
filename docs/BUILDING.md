# Building MagicSpot

MagicSpot pins Rust in `rust-toolchain.toml`. Clone the repository and keep
`Cargo.lock` unchanged for reproducible builds.

## Windows

Install Rust with rustup and the Microsoft C++ Build Tools. A normal release
does not need Visual Studio, CMake, LLVM, or vcpkg:

```powershell
git clone https://github.com/FallenG101/MagicSpot.git
cd MagicSpot
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Release
```

Run the command from the repository directory. From elsewhere, pass the full
path to `build.ps1`. `Blocking waiting for file lock on build directory` means
another Cargo process is using `target`; it is informational and the build
continues when that process releases the lock.

The result is `target/release/magicspot.exe`. `build.ps1` deliberately uses
`--no-default-features`, which keeps the ordinary build small and avoids the
optional MilkDrop native toolchain.

## macOS

Install the Xcode command-line tools and Rust, then run:

```sh
cargo build --locked --release --no-default-features
```

`packaging/macos/bundle.sh` can place a built binary in a MagicSpot application
bundle on a Mac.

## Linux

Install Rust plus the audio and display development libraries. Debian or
Ubuntu:

```sh
sudo apt install libasound2-dev libpulse-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev
cargo build --locked --release --no-default-features
```

Arch:

```sh
sudo pacman -S --needed alsa-lib libpulse libxkbcommon wayland
cargo build --locked --release --no-default-features
```

Fedora:

```sh
sudo dnf install alsa-lib-devel pulseaudio-libs-devel libxkbcommon-devel wayland-devel
cargo build --locked --release --no-default-features
```

The Nix flake provides the pinned development environment and a MagicSpot
package:

```sh
nix develop
nix build
```

## Optional MilkDrop support

Default features include projectM. Building them requires CMake and libclang.
On Windows, projectM also requires Visual Studio 2022 and vcpkg with
`VCPKG_INSTALLATION_ROOT` configured. Run `cargo build --locked --release`
after those dependencies are available.

## Validation

```sh
cargo fmt --all --check
cargo test --locked --no-default-features --features demo --all-targets
cargo clippy --locked --no-default-features --features demo --all-targets -- -D warnings
```

The **CI** workflow can run those checks on Windows, macOS, and Linux from the
GitHub Actions page. It remains manual so full cross-platform runs are
deliberate and milestone releases stay easy to audit. Standard hosted runners
are free for the public repository; local checks are sufficient during normal
iteration.

The Windows and macOS release workflows are also manual. Create and push a
version tag, then run **Release** followed by **macOS release** with that tag
when a downloadable milestone is ready.

For the complete version bump, tagging, packaging, and verification sequence,
see the release runbook in `docs/HANDOFF.md`.
