#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');

const platform = os.platform();
const arch = os.arch();

console.log(`Installing EasyProject MCP Server for ${platform}-${arch}...`);

// Check if we already have a binary
const binaryName = platform === 'win32' ? 'easyproject-mcp-server.exe' : 'easyproject-mcp-server';
const binPath = path.join(__dirname, '..', 'bin', binaryName);

if (fs.existsSync(binPath)) {
  console.log('Binary already exists, skipping installation.');
  process.exit(0);
}

// Try to find platform-specific package
const platformPackageName = `easyproject-mcp-server-${platform}-${arch}`;

try {
  // Check if platform package is available
  const platformPackagePath = require.resolve(`${platformPackageName}/package.json`);
  const platformBinPath = path.join(path.dirname(platformPackagePath), 'bin', binaryName);
  
  if (fs.existsSync(platformBinPath)) {
    console.log(`Found platform-specific binary at ${platformBinPath}`);
    
    // Ensure bin directory exists
    const targetBinDir = path.dirname(binPath);
    if (!fs.existsSync(targetBinDir)) {
      fs.mkdirSync(targetBinDir, { recursive: true });
    }
    
    // Copy binary to main package
    fs.copyFileSync(platformBinPath, binPath);
    
    // Make executable on Unix-like systems
    if (platform !== 'win32') {
      fs.chmodSync(binPath, 0o755);
    }
    
    console.log(`Successfully installed binary for ${platform}-${arch}`);
  } else {
    console.warn(`Platform-specific binary not found for ${platform}-${arch}`);
    console.warn('You may need to build the binary manually or install a platform-specific package.');
  }
  
} catch (error) {
  console.warn(`Platform-specific package not found: ${platformPackageName}`);
  console.warn('Falling back to manual installation.');
  console.warn('To install platform-specific package, run:');
  console.warn(`  npm install ${platformPackageName}`);
  
  // Check if we can use an existing binary from deployment
  const deploymentBinary = path.join(__dirname, '..', '..', 'deployment', 'easyproject-mcp-server.exe');
  if (platform === 'win32' && fs.existsSync(deploymentBinary)) {
    console.log('Found existing Windows binary in deployment folder, copying...');
    
    // Ensure bin directory exists
    const targetBinDir = path.dirname(binPath);
    if (!fs.existsSync(targetBinDir)) {
      fs.mkdirSync(targetBinDir, { recursive: true });
    }
    
    fs.copyFileSync(deploymentBinary, binPath);
    console.log('Successfully copied deployment binary.');
  }
}

console.log('Post-install completed.'); 