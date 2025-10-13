#!/usr/bin/env python3
"""
# Deploy Cell Data Utility Script

## Description

Python utility for deploying cell data to CKB blockchain via MCP server. Reads a file, hex-encodes contents, and submits to DeployCellData endpoint at specified MCP server URL. Handles JSON-RPC formatting, error responses, and returns deployment results (transaction hash, output index, data size). Simplifies file deployment by eliminating manual hex encoding and request construction.

## Usage

Deploy a file to CKB blockchain:
```bash
python3 deploy_cell_data.py <file_path> <mcp_server_url>
```

Example:
```bash
python3 deploy_cell_data.py /path/to/contract.bin http://localhost:8003
```

The script will:
1. Read the file contents
2. Hex-encode the data
3. Send to MCP server's DeployCellData tool at specified URL
4. Display deployment results (tx_hash, output_index, data_size, capacity)

## Requirements

- Python 3.6+
- MCP ckb-tools-server accessible at the specified URL
- Valid CKB testnet/devnet with funded account configured in server

## Exit Codes

- 0: Successful deployment
- 1: Error (file not found, server error, deployment failure)
"""

import sys
import json
import requests
from pathlib import Path


def deploy_cell_data(file_path, mcp_server_url):
	"""Deploy a file to CKB blockchain via MCP server."""

	# Validate file exists
	path = Path(file_path)
	if not path.exists():
		print(f"Error: File not found: {file_path}", file=sys.stderr)
		return 1

	if not path.is_file():
		print(f"Error: Path is not a file: {file_path}", file=sys.stderr)
		return 1

	# Read file contents
	try:
		with open(path, 'rb') as f:
			data = f.read()
		print(f"Read {len(data)} bytes from {file_path}")
	except Exception as e:
		print(f"Error reading file: {e}", file=sys.stderr)
		return 1

	# Hex encode data (without 0x prefix as per tool spec)
	hex_data = data.hex()
	print(f"Hex-encoded data: {len(hex_data)} characters ({len(data)} bytes)")

	# Prepare JSON-RPC request for MCP server
	mcp_request = {
		"jsonrpc": "2.0",
		"id": 1,
		"method": "tools/call",
		"params": {
			"name": "DeployCellData",
			"arguments": {
				"data": hex_data
			}
		}
	}

	# Build MCP endpoint URL
	mcp_url = f"{mcp_server_url.rstrip('/')}/mcp"
	print(f"Sending request to {mcp_url}...")

	try:
		response = requests.post(
			mcp_url,
			json=mcp_request,
			headers={"Content-Type": "application/json"},
			timeout=120  # 2 minute timeout for deployment
		)
		response.raise_for_status()
	except requests.exceptions.ConnectionError:
		print(f"Error: Cannot connect to MCP server at {mcp_server_url}", file=sys.stderr)
		print("Is the ckb-tools-server running and accessible?", file=sys.stderr)
		return 1
	except requests.exceptions.Timeout:
		print("Error: Request timed out after 120 seconds", file=sys.stderr)
		return 1
	except requests.exceptions.RequestException as e:
		print(f"Error sending request: {e}", file=sys.stderr)
		return 1

	# Parse response
	try:
		result = response.json()
	except json.JSONDecodeError as e:
		print(f"Error parsing JSON response: {e}", file=sys.stderr)
		print(f"Response text: {response.text}", file=sys.stderr)
		return 1

	# Check for JSON-RPC error
	if "error" in result:
		error = result["error"]
		print(f"MCP Error: {error.get('message', 'Unknown error')}", file=sys.stderr)
		if "data" in error:
			print(f"Details: {error['data']}", file=sys.stderr)
		return 1

	# Extract deployment result from MCP response
	if "result" not in result:
		print("Error: No result in response", file=sys.stderr)
		print(f"Response: {json.dumps(result, indent=2)}", file=sys.stderr)
		return 1

	mcp_result = result["result"]

	# MCP wraps tool result in content array
	if "content" in mcp_result and len(mcp_result["content"]) > 0:
		content_text = mcp_result["content"][0].get("text", "")
		try:
			deployment_result = json.loads(content_text)
		except json.JSONDecodeError:
			print("Error: Could not parse deployment result", file=sys.stderr)
			print(f"Content: {content_text}", file=sys.stderr)
			return 1
	else:
		print("Error: No content in MCP response", file=sys.stderr)
		return 1

	# Display deployment results
	print("\n=== Deployment Successful ===")
	print(f"Transaction Hash: {deployment_result.get('tx_hash', 'N/A')}")
	print(f"Output Index: {deployment_result.get('output_index', 'N/A')}")
	print(f"Data Size: {deployment_result.get('data_size', 'N/A')} bytes")
	print(f"Capacity: {deployment_result.get('capacity', 'N/A')} shannons")

	return 0


def main():
	"""Main entry point."""
	if len(sys.argv) != 3:
		print("Usage: python3 deploy_cell_data.py <file_path> <mcp_server_url>", file=sys.stderr)
		print("\nExample:", file=sys.stderr)
		print("  python3 deploy_cell_data.py /path/to/contract.bin http://localhost:8003", file=sys.stderr)
		return 1

	file_path = sys.argv[1]
	mcp_server_url = sys.argv[2]
	return deploy_cell_data(file_path, mcp_server_url)


if __name__ == "__main__":
	sys.exit(main())
