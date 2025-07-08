#!/usr/bin/env node

const { startServer, getVersion, isBinaryAvailable, getPlatformInfo } = require('../lib/index.js');
const fs = require('fs');
const path = require('path');

function log(message) {
  console.log(`[TEST] ${message}`);
}

function logError(message) {
  console.error(`[TEST ERROR] ${message}`);
}

function logSuccess(message) {
  console.log(`[TEST ✓] ${message}`);
}

async function testBinaryAvailability() {
  log('Testing binary availability...');
  
  try {
    const available = isBinaryAvailable();
    if (available) {
      logSuccess('Binary is available');
      return true;
    } else {
      logError('Binary is not available');
      return false;
    }
  } catch (error) {
    logError(`Binary availability check failed: ${error.message}`);
    return false;
  }
}

async function testPlatformInfo() {
  log('Testing platform info...');
  
  try {
    const info = getPlatformInfo();
    log(`Platform: ${info.platform}`);
    log(`Architecture: ${info.arch}`);
    log(`Binary path: ${info.binaryPath}`);
    log(`Package name: ${info.packageName}`);
    
    // Check if binary exists at the reported path
    if (fs.existsSync(info.binaryPath)) {
      logSuccess('Platform info is correct');
      return true;
    } else {
      logError(`Binary not found at reported path: ${info.binaryPath}`);
      return false;
    }
  } catch (error) {
    logError(`Platform info test failed: ${error.message}`);
    return false;
  }
}

async function testVersion() {
  log('Testing version retrieval...');
  
  try {
    const version = await getVersion();
    log(`Version: ${version}`);
    
    if (version && version.length > 0) {
      logSuccess('Version retrieval successful');
      return true;
    } else {
      logError('Version is empty or invalid');
      return false;
    }
  } catch (error) {
    logError(`Version test failed: ${error.message}`);
    return false;
  }
}

async function testServerStart() {
  log('Testing server start...');
  
  return new Promise((resolve) => {
    try {
      // Start server with minimal config
      const server = startServer({
        apiKey: 'test-key',
        baseUrl: 'https://test.example.com',
        logLevel: 'info'
      });
      
      let hasOutput = false;
      let timeoutId;
      
      // Set timeout for test
      timeoutId = setTimeout(() => {
        server.kill();
        if (hasOutput) {
          logSuccess('Server started successfully (killed after timeout)');
          resolve(true);
        } else {
          logError('Server started but produced no output within timeout');
          resolve(false);
        }
      }, 5000);
      
      server.stdout.on('data', (data) => {
        hasOutput = true;
        log(`Server output: ${data.toString().trim()}`);
      });
      
      server.stderr.on('data', (data) => {
        hasOutput = true;
        log(`Server stderr: ${data.toString().trim()}`);
      });
      
      server.on('error', (error) => {
        clearTimeout(timeoutId);
        logError(`Server start failed: ${error.message}`);
        resolve(false);
      });
      
      server.on('close', (code) => {
        clearTimeout(timeoutId);
        if (code === 0 || hasOutput) {
          logSuccess('Server started and closed successfully');
          resolve(true);
        } else {
          logError(`Server exited with code: ${code}`);
          resolve(false);
        }
      });
      
    } catch (error) {
      logError(`Server start test failed: ${error.message}`);
      resolve(false);
    }
  });
}

async function testCLIWrapper() {
  log('Testing CLI wrapper...');
  
  try {
    const { spawn } = require('child_process');
    const cliPath = path.join(__dirname, '..', 'bin', 'easyproject-mcp-server');
    
    if (!fs.existsSync(cliPath)) {
      logError('CLI wrapper not found');
      return false;
    }
    
    return new Promise((resolve) => {
      const child = spawn('node', [cliPath, '--version'], {
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let output = '';
      child.stdout.on('data', (data) => {
        output += data.toString();
      });
      
      child.stderr.on('data', (data) => {
        output += data.toString();
      });
      
      child.on('close', (code) => {
        if (output.length > 0) {
          log(`CLI output: ${output.trim()}`);
          logSuccess('CLI wrapper works');
          resolve(true);
        } else {
          logError('CLI wrapper produced no output');
          resolve(false);
        }
      });
      
      child.on('error', (error) => {
        logError(`CLI wrapper test failed: ${error.message}`);
        resolve(false);
      });
      
      // Timeout after 5 seconds
      setTimeout(() => {
        child.kill();
        if (output.length > 0) {
          logSuccess('CLI wrapper works (killed after timeout)');
          resolve(true);
        } else {
          logError('CLI wrapper timeout with no output');
          resolve(false);
        }
      }, 5000);
    });
    
  } catch (error) {
    logError(`CLI wrapper test failed: ${error.message}`);
    return false;
  }
}

async function testPackageIntegrity() {
  log('Testing package integrity...');
  
  try {
    // Check package.json
    const packagePath = path.join(__dirname, '..', 'package.json');
    if (!fs.existsSync(packagePath)) {
      logError('package.json not found');
      return false;
    }
    
    const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
    
    // Check required fields
    const requiredFields = ['name', 'version', 'description', 'main', 'bin'];
    for (const field of requiredFields) {
      if (!packageJson[field]) {
        logError(`package.json missing required field: ${field}`);
        return false;
      }
    }
    
    // Check main file exists
    const mainPath = path.join(__dirname, '..', packageJson.main);
    if (!fs.existsSync(mainPath)) {
      logError(`Main file not found: ${packageJson.main}`);
      return false;
    }
    
    // Check bin file exists
    const binPath = path.join(__dirname, '..', Object.values(packageJson.bin)[0]);
    if (!fs.existsSync(binPath)) {
      logError(`Bin file not found: ${Object.values(packageJson.bin)[0]}`);
      return false;
    }
    
    logSuccess('Package integrity check passed');
    return true;
    
  } catch (error) {
    logError(`Package integrity test failed: ${error.message}`);
    return false;
  }
}

async function main() {
  log('EasyProject MCP Server - Test Suite');
  log('===================================');
  
  const tests = [
    { name: 'Package Integrity', fn: testPackageIntegrity },
    { name: 'Binary Availability', fn: testBinaryAvailability },
    { name: 'Platform Info', fn: testPlatformInfo },
    { name: 'Version Retrieval', fn: testVersion },
    { name: 'CLI Wrapper', fn: testCLIWrapper },
    { name: 'Server Start', fn: testServerStart },
  ];
  
  let passed = 0;
  let failed = 0;
  
  for (const test of tests) {
    log('');
    log(`Running test: ${test.name}`);
    log('-'.repeat(40));
    
    try {
      const result = await test.fn();
      if (result) {
        passed++;
        logSuccess(`${test.name} PASSED`);
      } else {
        failed++;
        logError(`${test.name} FAILED`);
      }
    } catch (error) {
      failed++;
      logError(`${test.name} FAILED: ${error.message}`);
    }
  }
  
  log('');
  log('Test Results');
  log('=============');
  log(`Passed: ${passed}`);
  log(`Failed: ${failed}`);
  log(`Total: ${tests.length}`);
  
  if (failed === 0) {
    logSuccess('All tests passed! 🎉');
    process.exit(0);
  } else {
    logError(`${failed} test(s) failed! ❌`);
    process.exit(1);
  }
}

if (require.main === module) {
  main().catch(error => {
    logError(`Test suite failed: ${error.message}`);
    process.exit(1);
  });
} 