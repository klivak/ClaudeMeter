"""Append missing translations from scripts/i18n_fill.json into locale files.

Each locale module ends with `<var>\n}`; new `<var>.insert("key", "value");`
lines are inserted just before that. Run `cargo fmt` afterwards.
"""
import io
import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
D = os.path.join(ROOT, "src", "i18n")
KEY_RE = re.compile(r'insert\(\s*"((?:[^"\\]|\\.)*)"')
UNI_RE = re.compile(r'\\u\{([0-9a-fA-F]+)\}')


def norm(k):
    return UNI_RE.sub(lambda m: chr(int(m.group(1), 16)), k)


def rust_lit(s):
    """Escape a string for a Rust source literal, escaping only the chars that need it."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def main():
    table = json.load(io.open(os.path.join(ROOT, "scripts", "i18n_fill.json"),
                              encoding="utf-8"))
    for locale, entries in table.items():
        path = os.path.join(D, locale + ".rs")
        src = io.open(path, encoding="utf-8").read()
        have = {norm(k) for k in KEY_RE.findall(src)}
        var = "strings" if "strings.insert(" in src else "m"
        tail = "\n    %s\n}" % var
        assert src.rstrip().endswith(tail.strip()), locale
        added = []
        for key, value in entries.items():
            if key in have:
                continue
            added.append('    %s.insert("%s", "%s");' % (var, rust_lit(key), rust_lit(value)))
        if not added:
            print(locale, "up to date")
            continue
        idx = src.rindex(tail)
        src = src[:idx] + "\n" + "\n".join(added) + src[idx:]
        io.open(path, "w", encoding="utf-8", newline="\n").write(src)
        print(locale, "+%d" % len(added))


if __name__ == "__main__":
    main()
