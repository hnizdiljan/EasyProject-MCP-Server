#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

function log(message) {
  console.log(`[PREPUBLISH] ${message}`);
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

function copyCurrentPlatformBinary() {
  const os = require('os');
  const platform = os.platform();
  const arch = os.arch();
  
  log(`Copying binary for current platform: ${platform}-${arch}`);
  
  // Try different sources for the binary
  const possibleSources = [
    path.join('npm-dist', `${platform}-${arch}`, 'bin', `easyproject-mcp-server${platform === 'win32' ? '.exe' : ''}`),
    path.join('deployment', 'easyproject-mcp-server.exe'), // Windows deployment
    path.join('target', 'release', `easyproject-mcp-server${platform === 'win32' ? '.exe' : ''}`),
    path.join('target', `x86_64-pc-windows-msvc`, 'release', 'easyproject-mcp-server.exe'), // Windows MSVC
  ];
  
  let sourceBinary = null;
  for (const source of possibleSources) {
    if (fs.existsSync(source)) {
      sourceBinary = source;
      break;
    }
  }
  
  if (!sourceBinary) {
    log('No binary found for current platform. Building...');
    run('npm run build -- --current');
    
    // Try again after build
    sourceBinary = path.join('npm-dist', `${platform}-${arch}`, 'bin', `easyproject-mcp-server${platform === 'win32' ? '.exe' : ''}`);
    if (!fs.existsSync(sourceBinary)) {
      throw new Error('Failed to build binary for current platform');
    }
  }
  
  // Copy to bin directory
  const targetBinary = path.join('bin', `easyproject-mcp-server${platform === 'win32' ? '.exe' : ''}`);
  
  log(`Copying ${sourceBinary} to ${targetBinary}`);
  fs.copyFileSync(sourceBinary, targetBinary);
  
  // Make executable on Unix-like systems
  if (platform !== 'win32') {
    fs.chmodSync(targetBinary, 0o755);
  }
  
  const stats = fs.statSync(targetBinary);
  const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
  log(`✓ Binary copied (${sizeMB} MB)`);
}

function createLicenseFile() {
  log('Creating LICENSE file...');
  
  const licenseContent = `MIT License

Copyright (c) 2025 EasyProject MCP Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;

  fs.writeFileSync('LICENSE', licenseContent);
  log('✓ LICENSE file created');
}

function createNpmReadme() {
  log('Creating npm-specific README...');
  
  const readmeContent = `# EasyProject MCP Server

Model Context Protocol server for EasyProject API integration.

## Installation

\`\`\`bash
npm install -g easyproject-mcp-server
\`\`\`

## Quick Start

### CLI Usage

\`\`\`bash
# Set environment variables
export EASYPROJECT_API_KEY="your-api-key"
export EASYPROJECT_BASE_URL="https://your-instance.easyproject.com"

# Start the server
easyproject-mcp-server
\`\`\`

### Programmatic Usage

\`\`\`javascript
const { startServer } = require('easyproject-mcp-server');

const server = startServer({
  apiKey: 'your-api-key',
  baseUrl: 'https://your-instance.easyproject.com',
  logLevel: 'info'
});

server.on('data', (data) => {
  console.log('Server output:', data.toString());
});
\`\`\`

### Cursor Integration

Add to your Cursor MCP configuration:

\`\`\`json
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
\`\`\`

## Available Tools

- **Project Management**: \`list_projects\`, \`get_project\`, \`create_project\`, etc.
- **Issue Management**: \`list_issues\`, \`get_issue\`, \`create_issue\`, etc.
- **User Management**: \`list_users\`, \`get_user\`, \`get_user_workload\`
- **Time Tracking**: \`list_time_entries\`, \`log_time\`, etc.
- **Reporting**: \`generate_project_report\`, \`get_dashboard_data\`

## Configuration

Environment Variables:

- \`EASYPROJECT_API_KEY\` - Your EasyProject API key (required)
- \`EASYPROJECT_BASE_URL\` - Your EasyProject instance URL (required)
- \`MCP_LOG_LEVEL\` - Log level: trace, debug, info, warn, error

## Platform Support

This package includes pre-built binaries for:
- Windows x64
- macOS x64 & ARM64
- Linux x64 & ARM64

## License

MIT

## Repository

https://github.com/your-org/easyproject-mcp-server
`;

  fs.writeFileSync('README-NPM.md', readmeContent);
  log('✓ npm README created');
}

function validatePackage() {
  log('Validating package...');
  
  // Check required files
  const requiredFiles = [
    'package.json',
    'lib/index.js',
    'bin/easyproject-mcp-server',
    'scripts/postinstall.js',
    'LICENSE'
  ];
  
  for (const file of requiredFiles) {
    if (!fs.existsSync(file)) {
      throw new Error(`Required file missing: ${file}`);
    }
  }
  
  // Check package.json validity
  const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  if (!packageJson.name || !packageJson.version) {
    throw new Error('package.json is missing name or version');
  }
  
  log('✓ Package validation passed');
}

function main() {
  log('EasyProject MCP Server - Prepublish');
  log('===================================');
  
  try {
    // Copy binary for current platform
    copyCurrentPlatformBinary();
    
    // Create LICENSE file
    createLicenseFile();
    
    // Create npm-specific README
    createNpmReadme();
    
    // Validate package
    validatePackage();
    
    log('');
    log('✓ Package ready for publishing!');
    log('');
    log('Next steps:');
    log('1. Review the package contents');
    log('2. Test installation: npm pack && npm install -g easyproject-mcp-server-*.tgz');
    log('3. Publish: npm publish');
    log('4. Or publish with tag: npm publish --tag beta');
    
  } catch (error) {
    console.error('Prepublish failed:', error.message);
    process.exit(1);
  }
}

if (require.main === module) {
  main();
} 