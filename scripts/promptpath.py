#!/usr/bin/python3
import os
from typing import Dict, Optional

# Configuration section
CODE_ROOT = "~/code"

# Define project mappings as (full_path: alias) pairs
PROJECT_MAPPINGS = {
    "~/code/github.com/ethereum-optimism/optimism": "optimism",
}

def normalize_path(path: str) -> str:
    """Normalize a path by expanding user directory and resolving symlinks."""
    return os.path.normpath(os.path.expanduser(path))

def find_project_alias(pwd: str) -> Optional[tuple[str, str]]:
    """
    Find the longest matching project path and its alias.
    Returns tuple of (full_path, alias) if found, None otherwise.
    """
    pwd = normalize_path(pwd)
    matching_paths = [
        (path, alias) 
        for path, alias in PROJECT_MAPPINGS.items()
        if pwd.startswith(normalize_path(path))
    ]
    
    return max(matching_paths, key=lambda x: len(x[0])) if matching_paths else None

def get_pathname() -> str:
    """
    Get the formatted pathname for shell prompt display.
    Handles project aliases and path trimming.
    """
    # Get current path with ~ for home directory
    pwd = os.getcwd().replace(os.path.expanduser('~'), '~', 1)
    
    # Check for project alias match
    project_match = find_project_alias(pwd)
    
    if project_match:
        full_path, alias = project_match
        # Instead of replacing the normalized path, replace the ~ version
        relative_path = pwd.replace(full_path, alias, 1)
        return strip_leading_slash(relative_path)
    
    # Fall back to original CODE_ROOT behavior
    if pwd == CODE_ROOT:
        return pwd[2:]
    pwd = pwd.replace(CODE_ROOT, "", 1)
    return strip_leading_slash(pwd)

def strip_leading_slash(path: str) -> str:
    """Remove leading slash from path if present."""
    return path[1:] if path and path[0] == "/" else path

if __name__ == "__main__":
    print(get_pathname())
