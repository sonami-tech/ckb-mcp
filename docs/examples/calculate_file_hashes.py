#!/usr/bin/env python3
"""
# Calculate File Hashes Utility

## Description

Python utility for calculating cryptographic hashes of local files. Computes SHA-256, Blake2b-256 (plain), and CKB hash (Blake2b-256 with ckb-default-hash personalization). Returns JSON output optimized for AI parsing with file metadata, hex-encoded data preview, and all three hash values for verification and comparison.

## Usage

Calculate hashes for a file:
```bash
python3 calculate_file_hashes.py <file_path>
```

Example:
```bash
python3 calculate_file_hashes.py /path/to/contract.bin
```

Output (JSON):
```json
{
  "file_path": "/path/to/contract.bin",
  "file_size": 5120,
  "data_hex_preview": "0x8db724857fd9686dc2dd766adae2fa460ceff033...",
  "sha256": "0xeee4b0f8bfe05d29e4b00becb1b1d4b0d5f0847e58bb387998fbef50773cd99b",
  "blake2b_plain": "0x710319eea90a60d91b9743e129578dede208ea0c88725968b02db38f7d01b19d",
  "ckb_hash": "0x0ca8275ae16a2eb01aee8693461ec7aa3f6d7cb4cb85f0528b3242b4416127a8"
}
```

## Hash Algorithms

- **SHA-256**: Standard SHA-256 hash
- **Blake2b-256 (plain)**: Blake2b with 32-byte output, no personalization
- **CKB hash**: Blake2b-256 with 'ckb-default-hash' personalization (used by CKB blockchain)

## Requirements

- Python 3.6+
- Standard library only (no external dependencies)

## Exit Codes

- 0: Successful hash calculation
- 1: Error (file not found, read error)
"""

import sys
import json
import hashlib
from pathlib import Path


def calculate_hashes(file_path):
	"""Calculate SHA-256, Blake2b-256, and CKB hash for a file."""

	# Validate file exists
	path = Path(file_path)
	if not path.exists():
		return {
			"error": "File not found",
			"file_path": file_path
		}, 1

	if not path.is_file():
		return {
			"error": "Path is not a file",
			"file_path": file_path
		}, 1

	# Read file contents
	try:
		with open(path, 'rb') as f:
			data = f.read()
	except Exception as e:
		return {
			"error": f"Failed to read file: {e}",
			"file_path": file_path
		}, 1

	# Calculate SHA-256
	sha256_hash = hashlib.sha256(data).hexdigest()

	# Calculate Blake2b-256 (no personalization)
	blake2b_plain_hash = hashlib.blake2b(data, digest_size=32).hexdigest()

	# Calculate CKB hash (Blake2b-256 with "ckb-default-hash" personalization)
	ckb_hash = hashlib.blake2b(data, digest_size=32, person=b'ckb-default-hash').hexdigest()

	# Create data preview (first 32 bytes)
	preview_bytes = min(32, len(data))
	data_hex_preview = f"0x{data[:preview_bytes].hex()}"
	if len(data) > preview_bytes:
		data_hex_preview += "..."

	# Return structured result
	result = {
		"file_path": str(path.absolute()),
		"file_size": len(data),
		"data_hex_preview": data_hex_preview,
		"sha256": f"0x{sha256_hash}",
		"blake2b_plain": f"0x{blake2b_plain_hash}",
		"ckb_hash": f"0x{ckb_hash}"
	}

	return result, 0


def main():
	"""Main entry point."""
	if len(sys.argv) != 2:
		print(json.dumps({
			"error": "Invalid usage",
			"usage": "python3 calculate_file_hashes.py <file_path>",
			"example": "python3 calculate_file_hashes.py /path/to/file.bin"
		}, indent=2), file=sys.stderr)
		return 1

	file_path = sys.argv[1]
	result, exit_code = calculate_hashes(file_path)

	# Output JSON (stdout for success, stderr for errors)
	if exit_code == 0:
		print(json.dumps(result, indent=2))
	else:
		print(json.dumps(result, indent=2), file=sys.stderr)

	return exit_code


if __name__ == "__main__":
	sys.exit(main())
