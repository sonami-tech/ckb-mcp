# CKB MCP Utilities

This directory contains utility scripts for maintaining and validating the CKB MCP project.

## Scripts

### `verify_descriptions.py`

Verifies that all markdown documentation files have proper Description sections suitable for both document introductions and MCP resource descriptions.

#### Purpose

The CKB documentation is served through the ckb-docs-server MCP server, which uses the Description sections to provide meaningful summaries to AI assistants. This script ensures all documentation maintains consistent quality and format.

#### Requirements

All markdown files in the `docs/` directory must:
- Have a `## Description` section immediately after the main title (`# Title`)
- Have non-empty description content
- Keep descriptions under 1024 bytes
- Use complete sentences with proper punctuation
- Be action-oriented and highlight practical value

#### Usage

```bash
# Basic verification
python3 utils/verify_descriptions.py

# Verbose output showing each file checked
python3 utils/verify_descriptions.py --verbose

# Custom docs path
python3 utils/verify_descriptions.py --docs-path /path/to/docs

# Show help
python3 utils/verify_descriptions.py --help
```

#### Output

The script provides:
- Summary statistics (total files, valid descriptions, errors)
- Average description length
- Detailed error messages for problematic files
- Warnings for descriptions that may need improvement
- Exit code 0 for success, 1 for failures

#### Example Output

```
Verifying 66 markdown files in /home/username/ckb-mcp/docs
============================================================

SUMMARY
============================================================
Total files: 66
Files with valid descriptions: 66
Files with errors: 0
Average description length: 460 bytes

✅ All 66 files have valid descriptions!
```

#### Integration

This script should be run:
- Before committing documentation changes
- As part of CI/CD validation pipelines
- When adding new documentation files
- Periodically to ensure documentation quality

#### Error Types

Common errors detected:
- Missing `## Description` section
- Empty description content
- Description exceeds 1024 bytes
- No main title in document
- File read errors

#### Warnings

Quality warnings include:
- Very short descriptions (may not be comprehensive)
- Descriptions not ending with proper punctuation

## Development

### Adding New Utilities

When adding new utility scripts:

1. Follow Python naming conventions (`snake_case.py`)
2. Include comprehensive docstrings and help text
3. Add argument parsing with `argparse`
4. Provide both verbose and quiet modes
5. Use appropriate exit codes (0 for success, 1+ for errors)
6. Update this README with script documentation
7. Reference the script in the main project documentation

### Code Style

- Use Python 3.6+ features
- Follow PEP 8 style guidelines
- Include type hints where appropriate
- Add comprehensive error handling
- Provide meaningful error messages

## Maintenance

These utilities are part of the project maintenance workflow and should be kept up-to-date with any changes to project structure or requirements.