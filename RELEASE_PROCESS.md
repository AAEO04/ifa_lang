# Release & Update Process

When you push a version tag (e.g., `v1.2.3`), GitHub Actions will:
1. Spin up **Ubuntu**, **Windows**, and **macOS** runners
2. Build `ifa` and `ifa-installer-gui` for each platform
3. Generate SHA256 checksums
4. Upload everything to **GitHub Releases** with release notes

## 🔄 How Users Update

### Option 1: CLI Self-Update (Recommended)
```bash
ifa oja upgrade
```

### Option 2: Re-download
Download the latest binary from the [Releases Page](https://github.com/AAEO04/ifa_lang/releases).

### Option 3: Re-run Installer (Windows)
Run the installer again — it always fetches the latest release.

## 🚀 How to Publish a New Release

### 1. Bump Version
Update the version in `Cargo.toml` (workspace section):
```toml
[workspace.package]
version = "1.2.3"  # Increment this
```

### 2. Commit & Push
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: Bump version to 1.2.3"
git push
```

### 3. Tag & Trigger Build
This triggers the cross-platform build:
```bash
git tag v1.2.3
git push origin v1.2.3
```

### 4. Watch the Build
Go to **GitHub > Actions** to watch the binaries getting built and uploaded.

## 📦 Release Artifacts

Each release includes:
- `ifa-v*-linux-x86_64` — Linux x64 binary
- `ifa-v*-linux-arm64` — Linux ARM binary
- `ifa-v*-macos-x86_64` — macOS Intel binary
- `ifa-v*-macos-arm64` — macOS Apple Silicon binary
- `ifa-v*-windows-x86_64.exe` — Windows binary
- `ifa-installer-v*-windows-x86_64.exe` — Windows GUI installer
- `ifa-installer-v*-macos-*` — macOS GUI installer
- `ifa-installer-v*-linux-x86_64` — Linux GUI installer
- `checksums-v*.sha256` — SHA256 checksums for verification
