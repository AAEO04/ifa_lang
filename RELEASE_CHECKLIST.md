# Ifá-Lang 1.2 Release Checklist

## Pre-Release

### Code Quality
- [ ] All tests pass on Windows/Linux/macOS
- [ ] Clippy has no warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] No security vulnerabilities (`cargo audit`)
- [ ] Documentation builds cleanly (`cargo doc --no-deps`)

### Testing
- [ ] Unit tests pass (`cargo test --workspace`)
- [ ] Integration tests pass
- [ ] Property tests pass (proptest)
- [ ] Manual testing of `ifa` CLI commands
- [ ] Sandbox testing on each platform

### Documentation
- [ ] README.md updated with latest features
- [ ] TUTORIAL.md reflects current syntax
- [ ] API documentation complete
- [ ] CHANGELOG.md updated

## Release Process

### 1. Version Bump
```bash
# Update version in all Cargo.toml files
# Edit workspace version in root Cargo.toml
```

### 2. Create Tag
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 3. Verify CI
- [ ] CI workflow passes
- [ ] Release workflow triggers
- [ ] Artifacts are created for all platforms
- [ ] Installers are generated

### 4. Test Installation
```bash
# Linux/macOS
curl -sSL https://get.ifa-lang.org | bash
ifa --version

# Windows
irm https://get.ifa-lang.org/windows | iex
ifa --version
```

### 5. Post-Release
- [ ] Verify GitHub Release is published
- [ ] Update documentation site
- [ ] Announce on social media
- [ ] Update package managers (if applicable)

## Rollback Plan

If issues are discovered after release:

1. Mark release as "Pre-release" on GitHub
2. Create hotfix branch: `git checkout -b hotfix/v1.0.1`
3. Fix issues and test
4. Tag new version: `git tag -a v1.0.1`
5. Push and let CI create new release

## Platform-Specific Notes

### Windows
- Requires Visual Studio Build Tools for local builds
- Uses MSVC target (`x86_64-pc-windows-msvc`)
- Installer adds to user PATH

### macOS
- Universal binaries planned for future
- Currently builds for both x64 and ARM64

### Linux
- Builds on Ubuntu latest
- Cross-compiled ARM64 via `cross`
- Installer updates `.bashrc` and `.zshrc`
