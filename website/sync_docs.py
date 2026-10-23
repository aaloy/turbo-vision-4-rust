#!/usr/bin/env python3
"""Copy documentation out of the repository into website/docs.

Run from anywhere:  python3 website/sync_docs.py
Hand-written pages (index, whats-new, comparison, examples, tutorials index)
live in website/docs and are never touched by this script.

The Extensions section is the companion tv-extensions crate's docs/ folder,
copied from a checkout beside this one (or from $TV_EXTENSIONS_DIR). Without
the checkout the pages already under website/docs/extensions are kept.
"""

from __future__ import annotations

import os
import re
import shutil
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SITE_DOCS = REPO / "website" / "docs"
EXTENSIONS = Path(os.environ.get("TV_EXTENSIONS_DIR", REPO.parent / "tv-extensions"))

# Repository file -> page under website/docs
COPIES: dict[str, str] = {
    "CHANGELOG.md": "reference/changelog.md",
    "UPGRADING-TO-3.0.md": "reference/upgrading.md",
    "UPGRADING-TO-4.0.md": "reference/upgrading-4.0.md",
    "docs/TURBO-VISION-DESIGN.md": "reference/design.md",
    "docs/OWNER-COORDINATES.md": "reference/owner-coordinates.md",
    "docs/MISSING-INHERITANCE.md": "reference/inheritance.md",
    "docs/CLASS-DIAGRAM.md": "reference/class-diagram.md",
    "docs/RUST-API-CATALOG.md": "reference/api-catalog.md",
    "docs/RUST-API-CATALOGUE-INDEX.md": "reference/api-index.md",
    "docs/RUST-IMPLEMENTATION-REFERENCE.md": "reference/implementation.md",
    "docs/RUST-CODING-GUIDELINES.md": "reference/coding-guidelines.md",
    "docs/PALETTE-SYSTEM.md": "reference/palette-system.md",
    "docs/PALETTE-SYSTEM-DESIGN.md": "reference/palette-design.md",
    "docs/BORLAND-PALETTE-CHART.md": "reference/palette-chart.md",
    "docs/SERIALIZATION-PERSISTENCE.md": "reference/serialization.md",
    "docs/SERIALIZATION-QUICK-REFERENCE.md": "reference/serialization-quick.md",
    "docs/MORE-CONTROLS.md": "reference/more-controls.md",
    "AGENTS.md": "reference/agents.md",
    "docs/BIORHYTHM-CALCULATOR-TUTORIAL.md": "tutorials/biorhythm.md",
    "docs/CUSTOM-APPLICATION-RUST-EXAMPLE.md": "compare/custom-application-rust.md",
    "docs/CUSTOM-PROGRAM-CPP-EXAMPLE.md": "compare/custom-program-cpp.md",
}

IMAGES = ["logo.png"]
IMAGE_DIRS: list[str] = []  # captures live in docs/assets/captures

# Links that point at repository paths and must be rewritten for the site.
LINK_REWRITES = [
    (r"\]\(\.\./CHANGELOG\.md", "](../reference/changelog.md"),
    # MISSING-INHERITANCE.md and TURBO-VISION-DESIGN.md point at the 4.0 upgrade
    # guide, which lives under reference/ on the site, same as ../CHANGELOG.md above.
    (r"\]\(\.\./UPGRADING-TO-4\.0\.md", "](../reference/upgrading-4.0.md"),
    # Chapter 7 points at the class diagram, which lives under reference/ on the site.
    (r"\]\(\.\./CLASS-DIAGRAM\.md", "](../reference/class-diagram.md"),
    # AGENTS.md (reference/agents.md on the site) points at repository paths.
    (r"\]\(docs/user-guide/\)", "](../guide/index.md)"),
    (r"\]\(examples/\)", "](https://github.com/aovestdipaperino/turbo-vision-4-rust/tree/main/examples)"),
    (r"\]\(examples/gallery/\)", "](https://github.com/aovestdipaperino/turbo-vision-4-rust/tree/main/examples/gallery)"),
    (r"\]\(docs/DESIGN-SYSTEM-PLAN\.md\)", "](https://github.com/aovestdipaperino/turbo-vision-4-rust/blob/main/docs/DESIGN-SYSTEM-PLAN.md)"),
    # The API catalog points back at AGENTS.md, its neighbour under reference/.
    (r"\]\(\.\./AGENTS\.md", "](agents.md"),
    # Doctest fence attributes mean nothing to the site's highlighter.
    (r"```rust,(?:no_run|ignore)", "```rust"),
    # The class diagram points back at chapter 7 from reference/.
    (r"\]\(user-guide/Chapter-(\d\d)[^)]*\.md\)", r"](../guide/chapter-\1.md)"),
    (r"\]\(docs/user-guide/Chapter-(\d\d)[^)]*\.md\)", r"](guide/chapter-\1.md)"),
    (r"\]\(screenshots/", "](assets/screenshots/"),
    # Images referenced by the coding guidelines were never in the repository.
    (r"!\[[^\]]*\]\(M-[A-Z0-9_-]+\.png\)\n?", ""),
]

# A bare filename link between two repository documents becomes the site slug of
# the page that document was copied to.
def basename_rewrites() -> list[tuple[str, str]]:
    rules = []
    for rel_src, rel_dst in {**COPIES, **chapter_pages()}.items():
        name = Path(rel_src).name
        slug = Path(rel_dst).name
        rules.append((rf"\]\((?:\./)?(?:docs/)?(?:user-guide/)?{re.escape(name)}", f"]({slug}"))
    return rules


def chapter_pages() -> dict[str, str]:
    pages = {}
    for src in sorted((REPO / "docs" / "user-guide").glob("Chapter-*.md")):
        number = src.name.split("-", 2)[1]
        pages[f"docs/user-guide/{src.name}"] = f"guide/chapter-{number}.md"
    return pages


def rewrite(text: str) -> str:
    for pattern, replacement in LINK_REWRITES + basename_rewrites():
        text = re.sub(pattern, replacement, text)
    return text


def copy_page(rel_src: str, rel_dst: str) -> None:
    src = REPO / rel_src
    dst = SITE_DOCS / rel_dst
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(rewrite(src.read_text(encoding="utf-8")), encoding="utf-8")


def main() -> None:
    pages = {**COPIES, **chapter_pages()}
    for rel_src, rel_dst in pages.items():
        copy_page(rel_src, rel_dst)

    assets = SITE_DOCS / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    for name in IMAGES:
        shutil.copy2(REPO / name, assets / name)
    for name in IMAGE_DIRS:
        target = assets / name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(REPO / name, target)

    print(f"synced {len(pages)} pages and {len(IMAGES) + len(IMAGE_DIRS)} asset entries")
    sync_extensions()


def sync_extensions() -> None:
    """Copy tv-extensions' docs/*.md and docs/assets to website/docs/extensions."""
    src_dir = EXTENSIONS / "docs"
    if not src_dir.is_dir():
        print(f"no tv-extensions checkout at {EXTENSIONS}; kept the extensions pages as they are")
        return
    dst_dir = SITE_DOCS / "extensions"
    if dst_dir.exists():
        shutil.rmtree(dst_dir)
    dst_dir.mkdir(parents=True)
    pages = sorted(src_dir.glob("*.md"))
    for src in pages:
        # Its pages link only to each other, so only the doctest fences change.
        text = re.sub(r"```rust,(?:no_run|ignore)", "```rust", src.read_text(encoding="utf-8"))
        (dst_dir / src.name).write_text(text, encoding="utf-8")
    # The pages' pictures, at the same relative path.
    if (src_dir / "assets").is_dir():
        shutil.copytree(src_dir / "assets", dst_dir / "assets")
    print(f"synced {len(pages)} tv-extensions pages from {src_dir}")


if __name__ == "__main__":
    main()
