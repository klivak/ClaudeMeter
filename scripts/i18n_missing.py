"""Report which en.rs string keys each locale is missing (fallback usage)."""
import io
import json
import os
import re
import sys

D = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "src", "i18n")
KEY_RE = re.compile(r'insert\(\s*"((?:[^"\\]|\\.)*)"')


UNI_RE = re.compile(r'\\u\{([0-9a-fA-F]+)\}')


def norm(k):
    """Rust source escapes and raw characters denote the same key."""
    return UNI_RE.sub(lambda m: chr(int(m.group(1), 16)), k)


def keys(fname):
    s = io.open(os.path.join(D, fname), encoding="utf-8").read()
    return [norm(k) for k in KEY_RE.findall(s)]


def main():
    en = keys("en.rs")
    result = {}
    for f in sorted(os.listdir(D)):
        if f in ("mod.rs", "en.rs") or not f.endswith(".rs"):
            continue
        have = set(keys(f))
        missing = [k for k in en if k not in have]
        result[f[:-3]] = missing
    out = json.dumps(result, ensure_ascii=False, indent=1)
    io.open(sys.argv[1] if len(sys.argv) > 1 else "missing.json", "w",
            encoding="utf-8").write(out)
    for loc, miss in result.items():
        print(loc, len(miss))
    print("en keys:", len(en))


if __name__ == "__main__":
    main()
