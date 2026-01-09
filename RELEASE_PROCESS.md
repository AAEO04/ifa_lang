# Release & Update Process



When you push a version tag (e.g., `v1.2.1`), GitHub servers will:
1.  Spin up **Ubuntu**, **Windows**, and **macOS** runners.
2.  Build the `ifa` binary for each platform.
3.  Upload them to the **GitHub Releases** page.

## 🔄 How Users Update
Since `ifa` is distributed as a single binary, updating is simple:

### Option 1: CLI Self-Update (Recommended)
You can update Ifá-Lang directly from the command line:

```bash
ifa oja upgrade
```

### Option 2: Re-run Installer
Users simply run the installation command again. It always fetches the **latest** release and overwrites the old binary.

**Unix**:
```bash
curl -sSL https://raw.githubusercontent.com/AAEO04/ifa-lang/main/install.sh | bash
```

**Windows**:
```powershell
iwr https://raw.githubusercontent.com/AAEO04/ifa-lang/main/install.ps1 -useb | iex
```

### Option 2: Manual Download
Users can download the new executable from the [Releases Page](https://github.com/AAEO04/ifa-lang/releases) and replace their existing one.

## 🚀 How to Publish a New Release (Developer Guide)

### 1. Bump Version
Update the version in `Cargo.toml` (workspace section):
```toml
[workspace.package]
version = "1.2.1"  # Increment this
```

### 2. Commit & Push
```bash
git add Cargo.toml
git commit -m "chore: Bump version to 1.2.1"
git push
```

### 3. Tag & Trigger Build
This is what triggers the cross-platform build:
```bash
git tag v1.2.1
git push origin v1.2.1
```

### 4. Watch the Build
Go to **GitHub > Actions** to watch the binaries getting built and uploaded.
