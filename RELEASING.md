# Release Process

This project uses [cargo-dist](https://opensource.axo.dev/cargo-dist/) for automated releases.

## How to Create a Release

1. **Update version** in `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "0.1.0"  # Update this
   ```

2. **Commit the version change**:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.1.0"
   ```

3. **Create and push a git tag**:
   ```bash
   git tag v0.1.0
   git push origin main --tags
   ```

4. **GitHub Actions will automatically**:
   - Build binaries for all supported platforms
   - Create installers (shell, PowerShell, MSI)
   - Generate checksums
   - Create a GitHub Release with all artifacts

## Supported Platforms

- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Linux**: x86_64 (glibc), x86_64 (musl)
- **Windows**: x86_64

## Installation Methods

After a release is published, users can install via:

### Shell (macOS/Linux)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/crrow/rsketch/releases/latest/download/rsketch-installer.sh | sh
```

### PowerShell (Windows)
```powershell
irm https://github.com/crrow/rsketch/releases/latest/download/rsketch-installer.ps1 | iex
```

### MSI Installer (Windows)
Download the `.msi` file from the [releases page](https://github.com/crrow/rsketch/releases).

## Testing Release Workflow

You can test the release workflow on pull requests without creating a real release:
```bash
git tag v0.1.0-test
git push origin v0.1.0-test
```

The workflow will run but won't publish artifacts.
