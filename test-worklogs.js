#!/usr/bin/env node

/**
 * Quick test script to verify list_time_entries with issue_id parameter
 */

const { spawn } = require('child_process');
const path = require('path');

// Path to the built binary
const binaryPath = path.join(__dirname, 'target', 'release', 'easyproject-mcp-server.exe');

console.log('🧪 Testing MCP Server - list_time_entries with issue_id=5963');
console.log('='.repeat(60));

// Start the MCP server
const server = spawn(binaryPath, [], {
    stdio: ['pipe', 'pipe', 'pipe'],
    env: process.env
});

let responseBuffer = '';
let requestId = 1;

// Handle server output
server.stdout.on('data', (data) => {
    responseBuffer += data.toString();

    // Try to parse JSON-RPC responses
    const lines = responseBuffer.split('\n');
    responseBuffer = lines.pop(); // Keep incomplete line in buffer

    lines.forEach(line => {
        if (line.trim()) {
            try {
                const response = JSON.parse(line);
                console.log('\n📥 Response:', JSON.stringify(response, null, 2));

                // If this is the tools/list response, call list_time_entries
                if (response.result && response.result.tools) {
                    console.log('\n✅ Server initialized, found', response.result.tools.length, 'tools');

                    // Find list_time_entries tool
                    const tool = response.result.tools.find(t => t.name === 'list_time_entries');
                    if (tool) {
                        console.log('\n✅ Found list_time_entries tool');
                        console.log('   Description:', tool.description);
                        console.log('   Input schema has issue_id:', 'issue_id' in tool.inputSchema);

                        // Now call the tool
                        console.log('\n📤 Calling list_time_entries with issue_id=5963...');
                        sendRequest({
                            jsonrpc: '2.0',
                            id: requestId++,
                            method: 'tools/call',
                            params: {
                                name: 'list_time_entries',
                                arguments: {
                                    issue_id: 5963
                                }
                            }
                        });
                    } else {
                        console.error('❌ list_time_entries tool not found!');
                        server.kill();
                    }
                } else if (response.result && response.result.content) {
                    // This is the tool call result
                    console.log('\n✅ Tool call completed!');
                    console.log('\n📋 Result:');
                    response.result.content.forEach(content => {
                        if (content.type === 'text') {
                            console.log(content.text);
                        }
                    });

                    // Done, kill server
                    setTimeout(() => {
                        console.log('\n✅ Test completed successfully!');
                        server.kill();
                    }, 500);
                }
            } catch (e) {
                // Not JSON or incomplete, ignore
            }
        }
    });
});

server.stderr.on('data', (data) => {
    const msg = data.toString();
    if (msg.includes('ERROR') || msg.includes('error')) {
        console.error('⚠️  Server error:', msg);
    }
});

server.on('close', (code) => {
    console.log(`\n🏁 Server exited with code ${code}`);
    process.exit(code);
});

function sendRequest(request) {
    const json = JSON.stringify(request) + '\n';
    server.stdin.write(json);
}

// Initialize the server
console.log('\n📤 Initializing server...');
sendRequest({
    jsonrpc: '2.0',
    id: requestId++,
    method: 'initialize',
    params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: {
            name: 'test-client',
            version: '1.0.0'
        }
    }
});

// Wait for initialization response, then list tools
setTimeout(() => {
    console.log('\n📤 Listing available tools...');
    sendRequest({
        jsonrpc: '2.0',
        id: requestId++,
        method: 'tools/list',
        params: {}
    });
}, 1000);

// Timeout after 10 seconds
setTimeout(() => {
    console.error('\n❌ Test timeout!');
    server.kill();
    process.exit(1);
}, 10000);
