#!/usr/bin/env python3
"""Resolve a ScummVM game folder/file to its full game title.

Usage:
    scummvm_name.py <code-or-.scummvm-file> [...]

Examples:
    scummvm_name.py dw
    scummvm_name.py /path/to/dw.scummvm
    scummvm_name.py dw tinsel:dw
"""
import argparse
import os
import re
import sys
import urllib.request

URL = "https://www.scummvm.org/compatibility/"
CACHE = os.path.expanduser("~/.cache/retroverse/scummvm_compat.html")


def fetch_html(force: bool = False) -> str:
    if not force and os.path.exists(CACHE):
        # cache for 7 days
        import time
        if time.time() - os.path.getmtime(CACHE) < 7 * 86400:
            with open(CACHE, encoding="utf-8") as f:
                return f.read()
    with urllib.request.urlopen(URL) as resp:
        html = resp.read().decode("utf-8", "ignore")
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    with open(CACHE, "w", encoding="utf-8") as f:
        f.write(html)
    return html


def load_mapping(force: bool = False):
    html = fetch_html(force)
    pairs = re.findall(
        r'<td class="gameFullName"><a[^>]*>(.*?)</a></td>\s*'
        r'<td class="gameScummVmId">([^<]+)</td>',
        html,
        re.S,
    )
    mapping = {}
    for name, sid in pairs:
        name = re.sub(r"<[^>]+>", "", name).strip()
        sid = sid.strip()
        code = sid.split(":")[-1].strip()
        mapping.setdefault(code, []).append((name, sid))
    return mapping


def code_from_arg(arg: str) -> str:
    if os.path.isfile(arg):
        with open(arg, encoding="utf-8", errors="ignore") as f:
            content = f.read().strip()
        # .scummvm files contain the game id (e.g. "dw") or a full id
        if content:
            return content.split(":")[-1].strip() if ":" in content else content.strip()
    # allow passing full id like tinsel:dw
    return arg.split(":")[-1].strip() if ":" in arg else arg.strip()


def main():
    parser = argparse.ArgumentParser(description="Resolve ScummVM game name")
    parser.add_argument("items", nargs="+", help="game code or .scummvm file")
    parser.add_argument("--refresh", action="store_true", help="refresh cached compatibility page")
    args = parser.parse_args()

    mapping = load_mapping(force=args.refresh)

    for item in args.items:
        code = code_from_arg(item)
        matches = mapping.get(code)
        if not matches:
            print(f"{code}: NOT FOUND")
            continue
        for name, sid in matches:
            print(f"{code}: {name} ({sid})")


if __name__ == "__main__":
    main()
