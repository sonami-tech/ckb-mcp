#!/usr/bin/env python3
"""
Extract and display descriptions from all CKB documentation files.
"""

import os
import re
from pathlib import Path

def extract_description(file_path):
    """Extract the description section from a markdown file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Look for ## Description section
        description_match = re.search(
            r'^## Description\s*\n\s*\n(.+?)(?=\n## |\n# |\Z)',
            content,
            re.MULTILINE | re.DOTALL
        )

        if description_match:
            description = description_match.group(1).strip()
            # Clean up any extra whitespace
            description = re.sub(r'\n\s*\n', '\n\n', description)
            return description
        else:
            return "No description found"

    except Exception as e:
        return f"Error reading file: {e}"

def main():
    """Find all markdown files and extract their descriptions."""
    docs_dir = Path("/home/username/ckb-mcp/docs")

    if not docs_dir.exists():
        print(f"Documentation directory not found: {docs_dir}")
        return

    # Find all markdown files
    md_files = sorted(docs_dir.rglob("*.md"))

    print(f"Found {len(md_files)} markdown files in {docs_dir}")
    print("=" * 80)
    print()

    for file_path in md_files:
        # Get relative path from docs directory
        relative_path = file_path.relative_to(docs_dir)

        print(f"FILE: {relative_path}")
        print("-" * 40)

        description = extract_description(file_path)
        print(description)
        print()
        print("=" * 80)
        print()

if __name__ == "__main__":
    main()