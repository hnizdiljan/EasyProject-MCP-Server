#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Supported targets for cross-compilation
const TARGETS = [
  { platform: 'win32', arch: 'x64', rustTarget: 'x86_64-pc-windows-msvc', ext: '.exe' },
  { platform: 'darwin', arch: 'x64', rustTarget: 'x86_64-apple-darwin', ext: '' },
  { platform: 'darwin', arch: 'arm64', rustTarget: 'aarch64-apple-darwin', ext: '' },
  { platform: 'linux', arch: 'x64', rustTarget: 'x86_64-unknown-linux-gnu', ext: '' },
  { platform: 'linux', arch: 'arm64', rustTarget: 'aarch64-unknown-linux-gnu', ext: '' },
];

function log(message) {
  console.log(`[BUILD] ${message}`);
}

function run(command, options = {}) {
  log(`Executing: ${command}`);
  try {
    return execSync(command, { 
      stdio: 'inherit', 
      encoding: 'utf8',
      ...options 
    });
  } catch (error) {
    console.error(`Command failed: ${command}`);
    throw error;
  }
}

function buildForTarget(target) {
  const { platform, arch, rustTarget, ext } = target;
  
  log(`Building for ${platform}-${arch} (${rustTarget})...`);
  
  try {
    // Add rust target if not already installed
    run(`rustup target add ${rustTarget}`);
    
    // Clean previous build
    run('cargo clean');
    
    // Build for target
    run(`cargo build --release --target ${rustTarget}`);
    
    // Copy binary to platform-specific location
    const sourceBinary = path.join('target', rustTarget, 'release', `easyproject-mcp-server${ext}`);
    const targetDir = path.join('npm-dist', `${platform}-${arch}`, 'bin');
    const targetBinary = path.join(targetDir, `easyproject-mcp-server${ext}`);
    
    if (!fs.existsSync(sourceBinary)) {
      throw new Error(`Binary not found: ${sourceBinary}`);
    }
    
    // Ensure target directory exists
    fs.mkdirSync(targetDir, { recursive: true });
    
    // Copy binary
    fs.copyFileSync(sourceBinary, targetBinary);
    
    // Make executable on Unix-like systems
    if (platform !== 'win32') {
      fs.chmodSync(targetBinary, 0o755);
    }
    
    // Get binary size
    const stats = fs.statSync(targetBinary);
    const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
    
    log(`✓ Built ${platform}-${arch} (${sizeMB} MB)`);
    
    return { platform, arch, size: sizeMB, path: targetBinary };
    
  } catch (error) {
    log(`✗ Failed to build ${platform}-${arch}: ${error.message}`);
    return null;
  }
}

function createPlatformPackages(buildResults) {
  log('Creating platform-specific package.json files...');
  
  buildResults.forEach(result => {
    if (!result) return;
    
    const { platform, arch } = result;
    const packageDir = path.join('npm-dist', `${platform}-${arch}`);
    
    const packageJson = {
      name: `easyproject-mcp-server-${platform}-${arch}`,
      version: require('../package.json').version,
      description: `EasyProject MCP Server binary for ${platform}-${arch}`,
      main: 'index.js',
      files: ['bin/', 'index.js'],
      os: [platform],
      cpu: [arch],
      license: 'MIT',
      repository: require('../package.json').repository,
    };
    
    fs.writeFileSync(
      path.join(packageDir, 'package.json'),
      JSON.stringify(packageJson, null, 2)
    );
    
    // Create simple index.js that just exports the binary path
    const indexJs = `
const path = require('path');
module.exports = {
  getBinaryPath: () => path.join(__dirname, 'bin', 'easyproject-mcp-server${platform === 'win32' ? '.exe' : ''}')
};
`.trim();
    
    fs.writeFileSync(path.join(packageDir, 'index.js'), indexJs);
    
    log(`✓ Created package for ${platform}-${arch}`);
  });
}

function main() {
  const args = process.argv.slice(2);
  const buildAll = args.includes('--all');
  const currentPlatformOnly = args.includes('--current') || (!buildAll && args.length === 0);
  
  log('EasyProject MCP Server - Build Script');
  log('====================================');
  
  // Clean previous builds
  if (fs.existsSync('npm-dist')) {
    fs.rmSync('npm-dist', { recursive: true });
  }
  fs.mkdirSync('npm-dist', { recursive: true });
  
  // Determine which targets to build
  let targetsToBuild = TARGETS;
  
  if (currentPlatformOnly) {
    const currentPlatform = os.platform();
    const currentArch = os.arch();
    
    targetsToBuild = TARGETS.filter(t => 
      t.platform === currentPlatform && t.arch === currentArch
    );
    
    if (targetsToBuild.length === 0) {
      log(`No matching target found for current platform: ${currentPlatform}-${currentArch}`);
      process.exit(1);
    }
    
    log(`Building for current platform only: ${currentPlatform}-${currentArch}`);
  } else if (buildAll) {
    log('Building for all supported platforms...');
  }
  
  // Build targets
  const buildResults = targetsToBuild.map(buildForTarget).filter(Boolean);
  
  if (buildResults.length === 0) {
    log('No successful builds!');
    process.exit(1);
  }
  
  // Create platform packages
  createPlatformPackages(buildResults);
  
  log('');
  log('Build Summary:');
  log('==============');
  buildResults.forEach(result => {
    log(`✓ ${result.platform}-${result.arch}: ${result.size} MB`);
  });
  
  log('');
  log('Next steps:');
  log('- Run `npm run test` to test the builds');
  log('- Run `npm run prepublish` to prepare for publishing');
  log('- Run `npm publish` to publish to npm');
}

if (require.main === module) {
  main();
} 