#!/usr/bin/env python3
"""
CKB Documentation Description Verification Script

This script verifies that all markdown files in the docs/ directory have proper
Description sections that are suitable for both document introductions and MCP
resource descriptions.

Usage:
    python3 utils/verify_descriptions.py [--verbose] [--docs-path PATH]

Requirements:
    - All markdown files must have a "## Description" section after the main title
    - Descriptions must be under 1024 bytes
    - Descriptions should be action-oriented and highlight practical value
"""

import os
import sys
import glob
import re
import argparse
from pathlib import Path
from typing import List, Tuple, Optional


class DescriptionVerifier:
    def __init__(self, docs_path: str, verbose: bool = False):
        self.docs_path = Path(docs_path)
        self.verbose = verbose
        self.errors = []
        self.warnings = []
    
    def log_verbose(self, message: str):
        """Log verbose output if enabled."""
        if self.verbose:
            print(f"[VERBOSE] {message}")
    
    def verify_file(self, filepath: Path) -> Tuple[bool, Optional[int]]:
        """
        Verify a single markdown file has a proper Description section.
        
        Returns:
            Tuple of (has_description, byte_length)
        """
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            self.errors.append(f"Could not read {filepath}: {e}")
            return False, None
        
        self.log_verbose(f"Checking {filepath.relative_to(self.docs_path.parent)}")
        
        # Check for main title
        if not content.strip().startswith('#'):
            self.errors.append(f"{filepath.relative_to(self.docs_path.parent)}: No main title found")
            return False, None
        
        # Find Description section using regex
        description_match = re.search(
            r'## Description\s*\n\n(.*?)(?=\n## |\n# |\Z)', 
            content, 
            re.DOTALL
        )
        
        if not description_match:
            self.errors.append(f"{filepath.relative_to(self.docs_path.parent)}: Missing '## Description' section")
            return False, None
        
        description_text = description_match.group(1).strip()
        
        if not description_text:
            self.errors.append(f"{filepath.relative_to(self.docs_path.parent)}: Description section is empty")
            return False, None
        
        # Check byte length
        byte_length = len(description_text.encode('utf-8'))
        
        if byte_length > 1024:
            self.errors.append(f"{filepath.relative_to(self.docs_path.parent)}: Description too long ({byte_length} bytes > 1024)")
            return True, byte_length
        
        # Check for basic quality indicators
        if len(description_text.split()) < 10:
            self.warnings.append(f"{filepath.relative_to(self.docs_path.parent)}: Description seems very short (may not be comprehensive)")
        
        if not description_text.endswith('.'):
            self.warnings.append(f"{filepath.relative_to(self.docs_path.parent)}: Description doesn't end with period")
        
        self.log_verbose(f"  ✓ Valid description ({byte_length} bytes)")
        return True, byte_length
    
    def verify_all_files(self) -> bool:
        """
        Verify all markdown files in the docs directory.
        
        Returns:
            True if all files pass verification, False otherwise
        """
        if not self.docs_path.exists():
            self.errors.append(f"Documentation directory not found: {self.docs_path}")
            return False
        
        # Find all markdown files
        md_files = list(self.docs_path.glob("**/*.md"))
        
        if not md_files:
            self.errors.append(f"No markdown files found in {self.docs_path}")
            return False
        
        print(f"Verifying {len(md_files)} markdown files in {self.docs_path}")
        print("=" * 60)
        
        valid_files = 0
        total_bytes = 0
        
        for filepath in sorted(md_files):
            has_description, byte_length = self.verify_file(filepath)
            if has_description:
                valid_files += 1
                if byte_length:
                    total_bytes += byte_length
        
        # Print summary
        print("\nSUMMARY")
        print("=" * 60)
        print(f"Total files: {len(md_files)}")
        print(f"Files with valid descriptions: {valid_files}")
        print(f"Files with errors: {len(md_files) - valid_files}")
        print(f"Average description length: {total_bytes // valid_files if valid_files > 0 else 0} bytes")
        
        if self.warnings:
            print(f"\nWARNINGS ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"  ⚠️  {warning}")
        
        if self.errors:
            print(f"\nERRORS ({len(self.errors)}):")
            for error in self.errors:
                print(f"  ❌ {error}")
            return False
        
        print(f"\n✅ All {len(md_files)} files have valid descriptions!")
        return True


def main():
    parser = argparse.ArgumentParser(
        description="Verify CKB documentation descriptions",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    python3 utils/verify_descriptions.py
    python3 utils/verify_descriptions.py --verbose
    python3 utils/verify_descriptions.py --docs-path ./docs
        """
    )
    
    parser.add_argument(
        '--docs-path',
        default='docs',
        help='Path to documentation directory (default: docs)'
    )
    
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose output'
    )
    
    args = parser.parse_args()
    
    # Resolve docs path relative to script location or current directory
    script_dir = Path(__file__).parent.parent
    docs_path = script_dir / args.docs_path
    
    if not docs_path.exists():
        # Try relative to current directory
        docs_path = Path(args.docs_path)
    
    verifier = DescriptionVerifier(docs_path, args.verbose)
    success = verifier.verify_all_files()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()