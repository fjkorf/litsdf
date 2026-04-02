#!/usr/bin/env python3
"""Extract full API reference from Rust source files for Claude.

Parses .rs files to extract module docs, struct/enum definitions with fields,
function signatures with parameter types and return types, const declarations,
and module import relationships. Outputs knowledge/api/API.md.

Usage:
    python3 scripts/generate-api-docs.py
"""

import re
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
OUTPUT = WORKSPACE / "knowledge" / "api" / "API.md"

# Files to document (ordered by crate, then importance)
SOURCE_FILES = [
    # ── litsdf_core (no Bevy dependency) ──
    "crates/litsdf_core/src/lib.rs",
    "crates/litsdf_core/src/models.rs",
    "crates/litsdf_core/src/scene.rs",
    "crates/litsdf_core/src/sdf.rs",
    "crates/litsdf_core/src/persistence.rs",
    # ── litsdf_render (Bevy rendering plugin) ──
    "crates/litsdf_render/src/lib.rs",
    "crates/litsdf_render/src/shader.rs",
    "crates/litsdf_render/src/scene_sync.rs",
    "crates/litsdf_render/src/camera.rs",
    "crates/litsdf_render/src/gizmos.rs",
    "crates/litsdf_render/src/picking.rs",
    # ── litsdf_cli (command-line tool) ──
    "crates/litsdf_cli/src/main.rs",
    "crates/litsdf_cli/src/commands/mod.rs",
    "crates/litsdf_cli/src/commands/scene.rs",
    "crates/litsdf_cli/src/commands/bone.rs",
    "crates/litsdf_cli/src/commands/shape.rs",
    "crates/litsdf_cli/src/commands/modifier.rs",
    # ── litsdf_editor (editor UI plugin) ──
    "crates/litsdf_editor/src/lib.rs",
    "crates/litsdf_editor/src/ui/mod.rs",
    "crates/litsdf_editor/src/ui/populate.rs",
    "crates/litsdf_editor/src/ui/sync.rs",
    "crates/litsdf_editor/src/ui/handlers.rs",
    "crates/litsdf_editor/src/ui/helpers.rs",
    "crates/litsdf_editor/src/ui/tree.rs",
    "crates/litsdf_editor/src/undo.rs",
    "crates/litsdf_editor/src/testing.rs",
]


def extract_module_docs(lines):
    """Extract leading //! module doc comments."""
    docs = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("//!"):
            docs.append(stripped[3:].lstrip(" ") if len(stripped) > 3 else "")
        elif stripped == "" and docs:
            docs.append("")
        else:
            break
    while docs and docs[-1] == "":
        docs.pop()
    return docs


def extract_doc_comment(lines, start):
    """Extract consecutive /// lines. Returns (docs, next_line_idx)."""
    docs = []
    i = start
    while i < len(lines) and lines[i].strip().startswith("///"):
        content = lines[i].strip()[3:]
        docs.append(content.lstrip(" ") if content else "")
        i += 1
    while docs and docs[-1] == "":
        docs.pop()
    return docs, i


def find_closing_brace(lines, start_line):
    """Find line of closing } matching first { on or after start_line."""
    depth = 0
    for i in range(start_line, len(lines)):
        for ch in lines[i]:
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return i
    return len(lines) - 1


def extract_block(lines, start_line):
    """Extract brace-delimited block from start through matching }."""
    end = find_closing_brace(lines, start_line)
    return "\n".join(lines[start_line : end + 1])


def extract_fn_signature(lines, start_line):
    """Extract function signature up to (but not including) the opening {."""
    sig_parts = []
    for i in range(start_line, min(start_line + 20, len(lines))):
        line = lines[i].rstrip()
        if "{" in line:
            sig_parts.append(line[: line.index("{")].rstrip())
            break
        sig_parts.append(line)
    return "\n".join(sig_parts)


def extract_const_value(lines, start_line):
    """Extract const declaration including multi-line array values."""
    result = []
    for i in range(start_line, min(start_line + 30, len(lines))):
        result.append(lines[i].rstrip())
        if lines[i].rstrip().endswith(";") or lines[i].rstrip().endswith("];"):
            break
    return "\n".join(result)


def extract_use_crate(lines):
    """Extract use crate::... import statements."""
    imports = []
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("use crate::"):
            parts = [line]
            while not line.endswith(";"):
                i += 1
                if i >= len(lines):
                    break
                line = lines[i].strip()
                parts.append(line)
            imports.append(" ".join(parts))
        i += 1
    return imports


def process_file(filepath):
    """Process a .rs file and extract all documented and public items."""
    lines = filepath.read_text().splitlines()
    rel_path = filepath.relative_to(WORKSPACE)

    result = {
        "path": str(rel_path),
        "module_docs": extract_module_docs(lines),
        "structs": [],
        "enums": [],
        "functions": [],
        "consts": [],
        "imports": extract_use_crate(lines),
    }

    i = 0
    while i < len(lines):
        stripped = lines[i].strip()

        # /// doc comment block → look for the item it documents
        if stripped.startswith("///"):
            docs, next_i = extract_doc_comment(lines, i)
            i = next_i

            # Skip #[...] attributes and blank lines
            while i < len(lines) and (
                lines[i].strip().startswith("#[") or lines[i].strip() == ""
            ):
                i += 1
            if i >= len(lines):
                break

            item_line = lines[i].strip()

            if re.match(r"(?:pub(?:\(crate\))?\s+)?struct\s+\w+", item_line):
                name = re.search(r"struct\s+(\w+)", item_line).group(1)
                if "{" in item_line or (i + 1 < len(lines) and "{" in lines[i + 1]):
                    code = extract_block(lines, i)
                else:
                    code = item_line
                result["structs"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            elif re.match(r"(?:pub(?:\(crate\))?\s+)?enum\s+\w+", item_line):
                name = re.search(r"enum\s+(\w+)", item_line).group(1)
                code = extract_block(lines, i)
                result["enums"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            elif re.match(r"(?:pub(?:\(crate\))?\s+)?fn\s+\w+", item_line):
                name = re.search(r"fn\s+(\w+)", item_line).group(1)
                sig = extract_fn_signature(lines, i)
                result["functions"].append(
                    {"name": name, "docs": docs, "signature": sig, "line": i + 1}
                )

            elif re.match(r"(?:pub(?:\(crate\))?\s+)?const\s+\w+", item_line):
                name = re.search(r"const\s+(\w+)", item_line).group(1)
                code = extract_const_value(lines, i)
                result["consts"].append(
                    {"name": name, "docs": docs, "code": code, "line": i + 1}
                )

            continue

        # Also extract pub structs/enums/fns without doc comments (common in litsdf)
        if re.match(r"pub\s+struct\s+\w+", stripped):
            name = re.search(r"struct\s+(\w+)", stripped).group(1)
            if "{" in stripped or (i + 1 < len(lines) and "{" in lines[i + 1]):
                code = extract_block(lines, i)
            else:
                code = stripped
            result["structs"].append(
                {"name": name, "docs": [], "code": code, "line": i + 1}
            )

        elif re.match(r"pub\s+enum\s+\w+", stripped):
            name = re.search(r"enum\s+(\w+)", stripped).group(1)
            code = extract_block(lines, i)
            result["enums"].append(
                {"name": name, "docs": [], "code": code, "line": i + 1}
            )

        elif re.match(r"pub\s+fn\s+\w+", stripped):
            name = re.search(r"fn\s+(\w+)", stripped).group(1)
            sig = extract_fn_signature(lines, i)
            result["functions"].append(
                {"name": name, "docs": [], "signature": sig, "line": i + 1}
            )

        elif re.match(r"pub\s+const\s+\w+", stripped):
            name = re.search(r"const\s+(\w+)", stripped).group(1)
            code = extract_const_value(lines, i)
            result["consts"].append(
                {"name": name, "docs": [], "code": code, "line": i + 1}
            )

        i += 1

    return result


def format_section(data):
    """Format one file's data as markdown."""
    parts = []
    parts.append(f"## `{data['path']}`\n")

    if data["module_docs"]:
        parts.append("\n".join(data["module_docs"]))
        parts.append("")

    for kind, key, code_field in [
        ("Structs", "structs", "code"),
        ("Enums", "enums", "code"),
        ("Functions", "functions", "signature"),
        ("Constants", "consts", "code"),
    ]:
        items = data[key]
        if not items:
            continue
        parts.append(f"### {kind}\n")
        for item in items:
            parts.append(f"#### `{item['name']}` (line {item['line']})\n")
            parts.append(f"```rust\n{item[code_field]}\n```\n")
            if item["docs"]:
                parts.append("\n".join(item["docs"]))
            parts.append("")

    if data["imports"]:
        parts.append("### Module Dependencies\n")
        parts.append("```rust")
        for imp in data["imports"]:
            parts.append(imp)
        parts.append("```\n")

    return "\n".join(parts)


def compute_stratification():
    """Compute module dependency stratification across all crates."""
    CRATE_DIRS = {
        "crates/litsdf_core/src": "core",
        "crates/litsdf_render/src": "render",
        "crates/litsdf_editor/src": "editor",
        "crates/litsdf_cli/src": "cli",
    }

    modules = {}
    for crate_dir, prefix in CRATE_DIRS.items():
        src_path = WORKSPACE / crate_dir
        if not src_path.exists():
            continue
        for rs_file in sorted(src_path.rglob("*.rs")):
            rel = rs_file.relative_to(src_path)
            mod_name = str(rel).replace("/", "::").replace(".rs", "")
            if mod_name in ("main", "bin"):
                continue
            mod_id = f"{prefix}::{mod_name}"
            modules[mod_id] = rs_file

    deps = {mod_id: set() for mod_id in modules}
    intra_re = re.compile(r"use crate::(\w+)")
    cross_re = re.compile(r"use (litsdf_core|litsdf_render|litsdf_editor)::(\w+)")

    CRATE_PREFIX = {
        "litsdf_core": "core",
        "litsdf_render": "render",
        "litsdf_editor": "editor",
    }

    for mod_id, filepath in modules.items():
        text = filepath.read_text()
        prefix = mod_id.split("::")[0]

        for m in intra_re.finditer(text):
            dep = m.group(1)
            dep_id = f"{prefix}::{dep}"
            if dep_id in modules and dep_id != mod_id:
                deps[mod_id].add(dep_id)

        for m in cross_re.finditer(text):
            crate_name = m.group(1)
            dep_mod = m.group(2)
            dep_prefix = CRATE_PREFIX.get(crate_name, crate_name)
            dep_id = f"{dep_prefix}::{dep_mod}"
            if dep_id in modules and dep_id != mod_id:
                deps[mod_id].add(dep_id)

    incoming = {mod_id: 0 for mod_id in modules}
    for mod_id, dep_set in deps.items():
        for dep in dep_set:
            if dep in incoming:
                incoming[dep] += 1

    rows = []
    for mod_id in modules:
        out_c = len(deps[mod_id])
        in_c = incoming[mod_id]
        strat = (out_c + 1) / (in_c + 1)
        role = (
            "foundation" if strat < 0.5
            else "core" if strat <= 1.0
            else "connector" if strat <= 2.0
            else "leaf"
        )
        rows.append((mod_id, out_c, in_c, strat, role))

    rows.sort(key=lambda r: r[3])

    lines = [
        "---\n",
        "## Module Stratification\n",
        "Stratification = (outgoing + 1) / (incoming + 1). "
        "Low = foundational, high = leaf.\n",
        "| Module | Out | In | Strat | Role |",
        "|--------|-----|-----|-------|------|",
    ]
    for mod_id, out_c, in_c, strat, role in rows:
        lines.append(f"| `{mod_id}` | {out_c} | {in_c} | {strat:.2f} | {role} |")
    lines.append("")
    return "\n".join(lines)


def main():
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)

    sections = [
        "# litsdf API Reference\n",
        "Generated from source by `python3 scripts/generate-api-docs.py`.",
        "Contains full type definitions, function signatures, and module dependencies.",
        "Run after any code change that adds, removes, or modifies types or functions.\n",
        "---\n",
    ]

    total = 0
    for rel_path in SOURCE_FILES:
        fp = WORKSPACE / rel_path
        if not fp.exists():
            print(f"  SKIP: {rel_path} (not found)")
            continue
        data = process_file(fp)
        n = (
            len(data["structs"])
            + len(data["enums"])
            + len(data["functions"])
            + len(data["consts"])
        )
        total += n
        if data["module_docs"] or n > 0:
            sections.append(format_section(data))

    sections.append(compute_stratification())

    OUTPUT.write_text("\n".join(sections) + "\n")
    print(f"  Generated {OUTPUT.relative_to(WORKSPACE)} ({total} items)")


if __name__ == "__main__":
    main()
