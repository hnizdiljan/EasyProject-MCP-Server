const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

/**
 * Get the path to the platform-specific binary
 * @returns {string} Path to the binary executable
 */
function getBinaryPath() {
  const platform = os.platform();
  const arch = os.arch();
  
  let binaryName = 'easyproject-mcp-server';
  if (platform === 'win32') {
    binaryName += '.exe';
  }
  
  // Try platform-specific package first
  const platformPackageName = `easyproject-mcp-server-${platform}-${arch}`;
  try {
    const platformPackagePath = require.resolve(`${platformPackageName}/bin/${binaryName}`);
    if (fs.existsSync(platformPackagePath)) {
      return platformPackagePath;
    }
  } catch (e) {
    // Platform-specific package not found, fallback to main binary
  }
  
  // Fallback to bundled binary
  const bundledPath = path.join(__dirname, '..', 'bin', binaryName);
  if (fs.existsSync(bundledPath)) {
    return bundledPath;
  }
  
  throw new Error(
    `EasyProject MCP Server binary not found for platform ${platform}-${arch}. ` +
    `Please install the platform-specific package: npm install ${platformPackageName}`
  );
}

/**
 * Start the EasyProject MCP Server
 * @param {Object} options - Configuration options
 * @param {string} options.apiKey - EasyProject API key
 * @param {string} options.baseUrl - EasyProject base URL
 * @param {string} [options.logLevel] - Log level (trace, debug, info, warn, error)
 * @param {Object} [options.env] - Additional environment variables
 * @returns {ChildProcess} The spawned process
 */
function startServer(options = {}) {
  const binaryPath = getBinaryPath();
  
  const env = {
    ...process.env,
    ...options.env,
  };
  
  if (options.apiKey) {
    env.EASYPROJECT_API_KEY = options.apiKey;
  }
  
  if (options.baseUrl) {
    env.EASYPROJECT_BASE_URL = options.baseUrl;
  }
  
  if (options.logLevel) {
    env.MCP_LOG_LEVEL = options.logLevel;
  }
  
  const args = options.args || [];
  
  const child = spawn(binaryPath, args, {
    env,
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  
  return child;
}

/**
 * Get version information
 * @returns {Promise<string>} Version string from the binary
 */
function getVersion() {
  return new Promise((resolve, reject) => {
    const binaryPath = getBinaryPath();
    const child = spawn(binaryPath, ['--version'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    
    let output = '';
    child.stdout.on('data', (data) => {
      output += data.toString();
    });
    
    child.on('close', (code) => {
      if (code === 0) {
        resolve(output.trim());
      } else {
        reject(new Error(`Failed to get version, exit code: ${code}`));
      }
    });
    
    child.on('error', reject);
  });
}

/**
 * Check if the binary exists and is executable
 * @returns {boolean} True if binary is available
 */
function isBinaryAvailable() {
  try {
    const binaryPath = getBinaryPath();
    return fs.existsSync(binaryPath);
  } catch (e) {
    return false;
  }
}

/**
 * Get platform-specific information
 * @returns {Object} Platform info
 */
function getPlatformInfo() {
  return {
    platform: os.platform(),
    arch: os.arch(),
    binaryPath: getBinaryPath(),
    packageName: `easyproject-mcp-server-${os.platform()}-${os.arch()}`,
  };
}

module.exports = {
  startServer,
  getVersion,
  isBinaryAvailable,
  getBinaryPath,
  getPlatformInfo,
}; 