# engine/python/uvr_utils.py
"""Utilities to locate the vendored `uvr` directory (which contains `lib_v5`).

This module provides a robust `find_uvr_dir()` implementation that tries several
likely locations (environment overrides, ancestors, cwd, sys.executable location,
pyinstaller `_MEIPASS`, etc.) and a convenience helper
`ensure_uvr_in_sys_path()` which inserts the directory into `sys.path`.

The engine and backends should call `ensure_uvr_in_sys_path()` early during
startup so that imports like `import lib_v5` resolve correctly whether running
from source, from a development build, or from an application bundle.
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Optional


def _has_lib_v5(path: Path) -> bool:
    try:
        return (path / "lib_v5").is_dir()
    except Exception:
        return False


def find_uvr_dir() -> Optional[Path]:
    """Return a Path to the `uvr` directory if found, otherwise a fallback Path.

    The search order is:
    1. Environment variables: PRISMSPLIT_UVR_DIR, PRISMSPLIT_UVR, UVR_DIR
    2. Ancestors of this file (checks for <ancestor>/uvr, <ancestor>/resources/uvr,
       <ancestor>/app/uvr)
    3. Current working directory: cwd/uvr
    4. Locations relative to the Python executable
    5. PyInstaller's _MEIPASS if present
    6. Fallback: guess repo root relative to this file (parents[2]/uvr)

    The function prefers paths that actually contain a `lib_v5` directory.
    """
    # 1) Environment overrides
    env_vars = ("PRISMSPLIT_UVR_DIR", "PRISMSPLIT_UVR", "UVR_DIR")
    for ev in env_vars:
        val = os.environ.get(ev)
        if not val:
            continue
        p = Path(val).expanduser().resolve()
        if _has_lib_v5(p):
            return p

    here = Path(__file__).resolve()

    # 2) Ancestor-based candidates
    candidates = []
    for anc in here.parents:
        candidates.append(anc / "uvr")
        candidates.append(anc / "resources" / "uvr")
        candidates.append(anc / "app" / "uvr")

    # 3) cwd
    candidates.append(Path.cwd() / "uvr")

    # 4) sys.executable neighbors
    try:
        exe = Path(sys.executable).resolve()
        candidates.append(exe.parent / "uvr")
        candidates.append(exe.parent.parent / "uvr")
    except Exception:
        pass

    # 5) PyInstaller bundle dir
    meipass = getattr(sys, "_MEIPASS", None)
    if meipass:
        candidates.append(Path(meipass) / "uvr")

    # De-duplicate and test
    seen = set()
    for c in candidates:
        try:
            rp = c.resolve()
        except Exception:
            rp = c
        if str(rp) in seen:
            continue
        seen.add(str(rp))
        if _has_lib_v5(rp):
            return rp

    # 6) Fallback guess (matches old behavior)
    try:
        repo_root = here.parents[2]
        fallback = repo_root / "uvr"
        return fallback
    except Exception:
        return None


def ensure_uvr_in_sys_path() -> Optional[Path]:
    """Find the uvr directory and insert it at the front of sys.path.

    Returns the Path that was inserted (or the fallback Path) or None if nothing
    could be guessed.
    """
    p = find_uvr_dir()
    if p is None:
        return None
    p_str = str(p)
    if p_str not in sys.path:
        sys.path.insert(0, p_str)
    return p


__all__ = ["find_uvr_dir", "ensure_uvr_in_sys_path"]
