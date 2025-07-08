# NPM Deployment Guide

This guide explains how to deploy EasyProject MCP Server as an npm package.

## Overview

The npm package provides a cross-platform distribution of the Rust binary with a Node.js wrapper. This allows users to install the server using `npm install` and use it both programmatically and as a CLI tool.

## Package Structure

```
easyproject-mcp-server/
├── package.json              # Main package manifest
├── lib/
│   └── index.js              # Node.js API wrapper
├── bin/
│   ├── .gitkeep              # Preserves directory structure
│   └── easyproject-mcp-server # CLI wrapper (created during build)
├── scripts/
│   ├── build.js              # Cross-platform build script
│   ├── postinstall.js        # Post-installation setup
│   ├── prepublish.js         # Pre-publish preparation
│   └── test.js               # Package testing
└── npm-deploy.ps1            # Main deployment script
```

## Prerequisites

### Required Tools

1. **Node.js & npm**
   ```bash
   # Download from https://nodejs.org/
   node --version    # Should be >= 14.0.0
   npm --version
   ```

2. **Rust Toolchain**
   ```bash
   # Install from https://rustup.rs/
   rustc --version
   cargo --version
   ```

3. **npm Account**
   ```bash
   # Register at https://npmjs.com/
   npm login
   npm whoami
   ```

### Cross-Platform Build Targets

For building all platforms, install Rust targets:

```bash
# Windows
rustup target add x86_64-pc-windows-msvc

# macOS
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Linux
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

## Deployment Commands

### Basic Deployment

```powershell
# Build for current platform and deploy
.\npm-deploy.ps1

# Test deployment without publishing
.\npm-deploy.ps1 -DryRun

# Deploy with beta tag
.\npm-deploy.ps1 -Beta
```

### Advanced Options

```powershell
# Build for all platforms
.\npm-deploy.ps1 -BuildAll

# Skip building (use existing binaries)
.\npm-deploy.ps1 -SkipBuild

# Override version
.\npm-deploy.ps1 -Version "1.2.3"

# Use custom registry
.\npm-deploy.ps1 -Registry "https://npm.your-company.com"
```

## Manual Steps

If you prefer manual control, you can run individual steps:

### 1. Install Dependencies

```bash
npm install
```

### 2. Build Binaries

```bash
# Current platform only
node scripts/build.js --current

# All platforms
node scripts/build.js --all
```

### 3. Prepare Package

```bash
node scripts/prepublish.js
```

### 4. Test Package

```bash
node scripts/test.js
```

### 5. Publish

```bash
# Standard release
npm publish

# Beta release
npm publish --tag beta

# Dry run
npm publish --dry-run
```

## Package Types

The deployment creates multiple package types:

### Main Package

- **Name**: `easyproject-mcp-server`
- **Purpose**: Main package with Node.js API and CLI
- **Contains**: Binary for current platform or postinstall download logic

### Platform-Specific Packages

When building with `-BuildAll`, creates separate packages for each platform:

- `easyproject-mcp-server-win32-x64`
- `easyproject-mcp-server-darwin-x64`
- `easyproject-mcp-server-darwin-arm64`
- `easyproject-mcp-server-linux-x64`
- `easyproject-mcp-server-linux-arm64`

These are automatically installed as optional dependencies based on the user's platform.

## Installation & Usage

After publishing, users can install and use the package:

### Global Installation

```bash
npm install -g easyproject-mcp-server

# Use as CLI
easyproject-mcp-server --version
```

### Local Installation

```bash
npm install easyproject-mcp-server

# Use programmatically
const { startServer } = require('easyproject-mcp-server');
```

### Cursor Integration

Users can configure Cursor MCP:

```json
{
  "mcpServers": {
    "easyproject": {
      "command": "easyproject-mcp-server",
      "env": {
        "EASYPROJECT_API_KEY": "your-api-key",
        "EASYPROJECT_BASE_URL": "https://your-instance.easyproject.com"
      }
    }
  }
}
```

## Version Management

### Semantic Versioning

Follow semantic versioning (semver):

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

### Version Update Process

1. Update version in `Cargo.toml`
2. Update version in `package.json` (or use `-Version` parameter)
3. Create git tag: `git tag v1.0.0`
4. Deploy: `.\npm-deploy.ps1`

## Troubleshooting

### Common Issues

#### Build Failures

```bash
# Clear Rust cache
cargo clean

# Update Rust
rustup update

# Reinstall targets
rustup target remove x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc
```

#### npm Authentication

```bash
# Login to npm
npm login

# Check current user
npm whoami

# Set registry
npm config set registry https://registry.npmjs.org/
```

#### Binary Not Found

The postinstall script should handle binary installation automatically. If it fails:

1. Check platform support in `scripts/build.js`
2. Ensure binary exists in deployment folder
3. Manually copy binary to `bin/` directory

### Debug Mode

Enable verbose logging:

```powershell
$env:NPM_DEBUG = "true"
.\npm-deploy.ps1 -DryRun
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: NPM Deploy

on:
  release:
    types: [published]

jobs:
  deploy:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Deploy to npm
        run: .\npm-deploy.ps1 -BuildAll
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Security Considerations

1. **API Keys**: Never include API keys in the package
2. **Binary Verification**: Consider code signing for binaries
3. **npm 2FA**: Enable two-factor authentication on npm account
4. **Registry Security**: Use official npm registry or trusted private registry

## Maintenance

### Regular Tasks

- Monitor download statistics on npmjs.com
- Update dependencies regularly
- Test on different platforms
- Keep Rust toolchain updated
- Review and update documentation

### Deprecation Process

If you need to deprecate a version:

```bash
# Deprecate specific version
npm deprecate easyproject-mcp-server@1.0.0 "Please upgrade to 1.1.0"

# Deprecate all versions (use carefully)
npm deprecate easyproject-mcp-server "Package no longer maintained"
```

## Support

For deployment issues:

1. Check this documentation
2. Review GitHub issues
3. Test with `-DryRun` flag first
4. Contact the development team 