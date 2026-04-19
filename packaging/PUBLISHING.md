<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Package Publishing Guide

Instructions for submitting SSG to platform-specific package repositories.

## Arch Linux (AUR)

The PKGBUILD is at `packaging/arch/PKGBUILD`.

### First-time submission

1. Create an AUR account at https://aur.archlinux.org
2. Clone the AUR package base:
   ```sh
   git clone ssh://aur@aur.archlinux.org/ssg.git aur-ssg
   cd aur-ssg
   ```
3. Copy `packaging/arch/PKGBUILD` into the AUR repo
4. Generate `.SRCINFO`:
   ```sh
   makepkg --printsrcinfo > .SRCINFO
   ```
5. Commit and push:
   ```sh
   git add PKGBUILD .SRCINFO
   git commit -m "Initial upload: ssg 0.0.37"
   git push
   ```

### Updating

1. Update `pkgver` and checksums in `PKGBUILD`
2. Regenerate `.SRCINFO`:
   ```sh
   makepkg --printsrcinfo > .SRCINFO
   ```
3. Commit and push

### Testing

```sh
cd packaging/arch
makepkg -si    # builds and installs locally
```

## Scoop (Windows)

The manifest is at `packaging/scoop/ssg.json`.

### Submitting to the Scoop bucket

1. Fork the Scoop extras bucket or maintain your own bucket repo
2. Copy `packaging/scoop/ssg.json` to the bucket
3. Update `version`, `url`, and `hash` fields to match the latest release
4. Submit a PR to the bucket repository

### Users install with

```powershell
scoop bucket add ssg https://github.com/sebastienrousseau/static-site-generator
scoop install ssg
```

### Updating

Update `version`, `url`, and `hash` in `ssg.json` for each release.

## winget (Windows)

The manifest is at `packaging/winget/ssg.yaml`.

### Submitting to winget-pkgs

1. Fork https://github.com/microsoft/winget-pkgs
2. Create the manifest directory:
   ```
   manifests/s/sebastienrousseau/ssg/0.0.37/
   ```
3. Copy `packaging/winget/ssg.yaml` as the installer manifest
4. Validate:
   ```powershell
   winget validate manifests/s/sebastienrousseau/ssg/0.0.37/
   ```
5. Submit a PR to `microsoft/winget-pkgs`

### Using wingetcreate

```powershell
wingetcreate submit --id sebastienrousseau.ssg --version 0.0.37 --urls https://github.com/sebastienrousseau/static-site-generator/releases/download/v0.0.37/ssg-windows-amd64.zip
```

### Updating

For new versions, use `wingetcreate update`:

```powershell
wingetcreate update sebastienrousseau.ssg --version 0.0.37 --urls <new-release-url>
```

## Homebrew

The formula is at `packaging/homebrew/ssg.rb` in the repository root.

### Submitting to Homebrew core

1. Fork https://github.com/Homebrew/homebrew-core
2. Add the formula to `Formula/s/ssg.rb`
3. Test locally:
   ```sh
   brew install --build-from-source ./packaging/homebrew/ssg.rb
   brew test ssg
   brew audit --strict ssg
   ```
4. Submit a PR to `Homebrew/homebrew-core`

### Self-hosted formula

Users can install directly from the repo:

```sh
brew install --formula https://raw.githubusercontent.com/sebastienrousseau/static-site-generator/main/packaging/homebrew/ssg.rb
```

### Updating

Update the `url`, `sha256`, and `version` in `packaging/homebrew/ssg.rb` for each release.

## Debian (.deb)

The build script is at `packaging/deb/build.sh`.

### Building

```sh
./packaging/deb/build.sh
```

This produces `ssg_<version>_amd64.deb`.

### Distribution

Attach the `.deb` file to the GitHub Release. Users install with:

```sh
sudo dpkg -i ssg_0.0.37_amd64.deb
```

## Release Checklist

1. Update version in `Cargo.toml`
2. Update `packaging/scoop/ssg.json` (version, url, hash)
3. Update `packaging/winget/ssg.yaml` (version, url)
4. Update `packaging/arch/PKGBUILD` (pkgver, checksums)
5. Update `packaging/homebrew/ssg.rb` (version, url, sha256)
6. Build `.deb` with `packaging/deb/build.sh`
7. Tag the release and push
8. Attach binaries and `.deb` to the GitHub Release
9. Submit PRs to AUR, winget-pkgs, Homebrew core, and Scoop bucket
