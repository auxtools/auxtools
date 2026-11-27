#!/usr/bin/env python3
"""Index the design notes by source file, and check the code they name.

    ci/design-notes.py            Rewrite docs/design/by-file.md
    ci/design-notes.py --check    Check the notes and that by-file.md is current

Each note in docs/design/ ends with an "Applies to" list: one bullet for
each file, a path in backticks, followed by the items the note concerns.
The index lists, for each of those files, the notes that name it and their
items. The check fails when a bullet names a path that does not exist, when
an item is not found in its file, when a note has no "Applies to" list, or
when by-file.md differs from what the notes give, or when README.md does not
list every note once, by its title.

An item is a name in backticks after the path. Text in parentheses is an
explanation, and a name with a space in it describes a form: neither is
checked. `*` in a name stands for any run of identifier characters.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
NOTES = ROOT / "docs" / "design"
INDEX = NOTES / "by-file.md"
LISTED = re.compile(r"^- \[(.*)\]\(([a-z-]+/[^)]+\.md)\)$", re.M)
BULLET = re.compile(r"- `([^`]+)`(?::\s*(.*))?$", re.S)
NAME = re.compile(r"[A-Za-z_*][A-Za-z0-9_*]*$")


def bullets(text):
    section = text.split("\n## Applies to\n", 1)
    if len(section) != 2:
        return None
    found = []
    for line in section[1].split("\n"):
        if line.startswith("- "):
            found.append(line)
        elif line.startswith("  ") and found:
            found[-1] += " " + line.strip()
    return found


def items(rest):
    outside, inside = rest or "", None
    while inside != outside:
        inside, outside = outside, re.sub(r"\([^()`]*(`[^`]*`[^()`]*)*\)", "", outside)
    return [item for item in re.findall(r"`([^`]+)`", outside) if " " not in item]


def text_of(path):
    files = [path] if path.is_file() else sorted(file for file in path.rglob("*") if file.is_file())
    return "\n".join(file.read_text(errors="replace") for file in files)


def named(item, text):
    last = re.split(r"::|\.|#", item.split("(", 1)[0].removesuffix("!"))[-1]
    if not NAME.match(last):
        return item in text
    pattern = r"\w*".join(re.escape(part) for part in last.split("*"))
    return last in text or re.search(r"(?<![A-Za-z0-9_])" + pattern + r"(?![A-Za-z0-9_])", text) is not None


def read():
    by_file, problems, titles = {}, [], {}
    for note in sorted(NOTES.glob("*/*.md")):
        text = note.read_text()
        name = note.relative_to(NOTES).as_posix()
        title = text.split("\n", 1)[0].removeprefix("# ")
        titles[name] = title
        listed = bullets(text)
        if not listed:
            problems.append(f"{name}: no \"Applies to\" list")
            continue
        for bullet in listed:
            match = BULLET.match(bullet)
            if not match:
                problems.append(f"{name}: a bullet does not start with a path in backticks: {bullet}")
                continue
            path, rest = match.group(1), match.group(2)
            if not (ROOT / path).exists():
                problems.append(f"{name}: {path} does not exist")
                continue
            text_at = text_of(ROOT / path)
            for item in items(rest):
                if not named(item, text_at):
                    problems.append(f"{name}: `{item}` is not in {path}")
            by_file.setdefault(path.rstrip("/"), []).append((title, name, rest))
    return by_file, problems + unlisted(titles)


def unlisted(titles):
    listed = LISTED.findall((NOTES / "README.md").read_text())
    names = [name for _, name in listed]
    problems = [f"README.md lists {name} {names.count(name)} times" for name in sorted(set(names)) if names.count(name) > 1]
    for title, name in listed:
        if name not in titles:
            problems.append(f"README.md lists {name}, which does not exist")
        elif titles[name] != title:
            problems.append(f"README.md calls {name} \"{title}\", and its title is \"{titles[name]}\"")
    return problems + [f"README.md does not list {name}" for name in sorted(set(titles) - set(names))]


def index(by_file):
    head = INDEX.read_text().split("\n## ", 1)[0].rstrip("\n") if INDEX.exists() else "# Design notes by source file"
    lines = [head, ""]
    group = None
    for path in sorted(by_file):
        top = path.split("/", 1)[0]
        if top != group:
            group = top
            lines += [f"## `{top}/`" if "/" in path else f"## `{top}`", ""]
        if "/" in path:
            lines += [f"### `{path}`", ""]
        for title, name, rest in by_file[path]:
            lines.append(f"- [{title}]({name})" + (f": {rest}" if rest else ""))
        lines.append("")
    return "\n".join(lines)


def main():
    by_file, problems = read()
    text = index(by_file)
    if sys.argv[1:] == ["--check"]:
        if not INDEX.exists() or INDEX.read_text() != text:
            problems.append("docs/design/by-file.md is out of date: run ci/design-notes.py")
        for problem in problems:
            print(problem, file=sys.stderr)
        sys.exit(1 if problems else 0)
    if sys.argv[1:]:
        sys.exit(__doc__)
    INDEX.write_text(text)
    for problem in problems:
        print(problem, file=sys.stderr)


if __name__ == "__main__":
    main()
