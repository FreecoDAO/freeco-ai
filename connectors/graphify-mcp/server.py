"""Graphify — MCP retrieval server for FreEco.ai (institutional memory).

Turns a folder of documents (grant PDFs, reports, policies, past applications)
into searchable memory that agents can query — the "Archivist" role. It wraps
Graphify (https://github.com/FreecoDAO/graphify) when available, and otherwise
falls back to a self-contained keyword search over the docs folder so the tool
is useful even before Graphify's graph is built.

Run:
    export DOCS_DIR="/path/to/association/documents"
    export GRAPHIFY_QUERY_CMD="graphify query"   # optional: your Graphify query CLI
    pip install -r requirements.txt && python server.py

Connect in FreEco.ai → Settings → Tools → "Add another tool server"
    name: memory   type: stdio   command: python /abs/path/to/server.py
"""

import os
import shlex
import subprocess
from pathlib import Path

from fastmcp import FastMCP

DOCS_DIR = Path(os.environ.get("DOCS_DIR", ".")).expanduser()
GRAPHIFY_QUERY_CMD = os.environ.get("GRAPHIFY_QUERY_CMD", "").strip()
TEXT_EXT = {".md", ".txt", ".rst", ".csv", ".json", ".yaml", ".yml", ".html", ".htm"}

mcp = FastMCP(
    "graphify-memory",
    instructions=(
        "Institutional memory over the Association's documents. Use search_docs "
        "to find relevant passages (grant history, policies, past applications) "
        "before drafting or answering. Cite the file each passage came from."
    ),
)


@mcp.tool()
def list_docs() -> list:
    """List the documents available as memory."""
    if not DOCS_DIR.exists():
        return [f"DOCS_DIR does not exist: {DOCS_DIR}"]
    return sorted(str(p.relative_to(DOCS_DIR)) for p in DOCS_DIR.rglob("*") if p.is_file())


@mcp.tool()
def search_docs(query: str, max_hits: int = 8) -> list:
    """Search institutional memory for a query.

    Uses Graphify's graph query when GRAPHIFY_QUERY_CMD is configured; otherwise
    keyword-searches the text documents in DOCS_DIR and returns matching snippets
    with their source file.
    """
    if GRAPHIFY_QUERY_CMD:
        try:
            out = subprocess.run(
                shlex.split(GRAPHIFY_QUERY_CMD) + [query],
                capture_output=True, text=True, timeout=60, cwd=str(DOCS_DIR),
            )
            if out.returncode == 0 and out.stdout.strip():
                return [out.stdout.strip()]
        except Exception as e:  # fall through to keyword search
            pass  # noqa

    if not DOCS_DIR.exists():
        return [f"DOCS_DIR does not exist: {DOCS_DIR}"]
    terms = [t.lower() for t in query.split() if t]
    hits = []
    for p in DOCS_DIR.rglob("*"):
        if not p.is_file() or p.suffix.lower() not in TEXT_EXT:
            continue
        try:
            text = p.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        low = text.lower()
        if terms and all(t in low for t in terms):
            idx = low.find(terms[0])
            start = max(0, idx - 160)
            snippet = text[start:idx + 240].replace("\n", " ").strip()
            hits.append({"file": str(p.relative_to(DOCS_DIR)), "snippet": snippet})
            if len(hits) >= max_hits:
                break
    return hits or [f"No matches for: {query}"]


if __name__ == "__main__":
    mcp.run()
