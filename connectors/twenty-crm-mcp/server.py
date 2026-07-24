"""Twenty CRM — MCP server for FreEco.ai.

Wraps Twenty's REST API (https://twenty.com) as MCP tools so FreEco.ai agents
(Grant Scout, Relationship Manager, …) can put contacts, companies, notes and
opportunities into the CRM — the system of record for donors, grant givers and
VCs.

Run:
    export TWENTY_API_URL="http://localhost:3000"     # your Twenty base URL
    export TWENTY_API_KEY="<api key from Twenty → Settings → API & Webhooks>"
    uvx --from fastmcp --with httpx python server.py   # or: pip install -r requirements.txt && python server.py

Then in FreEco.ai → Settings → Tools → "Add another tool server":
    name: crm   type: stdio   command: python /abs/path/to/server.py
(or run it over HTTP and connect by URL).

Notes:
- Twenty uses composite fields: a person's name is {firstName, lastName} and
  emails is {primaryEmail}. This wrapper maps friendly arguments onto that shape.
- Field names are stable across recent Twenty versions but may drift; every tool
  returns Twenty's raw error text so mismatches are visible, not silent.
"""

import os
from typing import Any, Optional

import httpx
from fastmcp import FastMCP

API_URL = os.environ.get("TWENTY_API_URL", "http://localhost:3000").rstrip("/")
API_KEY = os.environ.get("TWENTY_API_KEY", "")
REST = f"{API_URL}/rest"

mcp = FastMCP(
    "twenty-crm",
    instructions=(
        "Twenty CRM. Use these tools to record and retrieve People (contacts), "
        "Companies, Notes and Opportunities. Always search before creating to "
        "avoid duplicates. Emails are the natural key for a person."
    ),
)


def _headers() -> dict:
    if not API_KEY:
        raise RuntimeError("TWENTY_API_KEY is not set")
    return {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}


async def _req(method: str, path: str, **kw) -> Any:
    async with httpx.AsyncClient(timeout=30) as client:
        r = await client.request(method, f"{REST}{path}", headers=_headers(), **kw)
        if r.status_code >= 400:
            # Surface Twenty's own message rather than a generic failure.
            raise RuntimeError(f"Twenty API {r.status_code}: {r.text[:500]}")
        return r.json() if r.text else {}


@mcp.tool()
async def find_people(query: str = "", limit: int = 10) -> Any:
    """Search People (contacts) by name/email substring. Empty query lists recent."""
    params = {"limit": max(1, min(limit, 60))}
    if query:
        params["filter"] = f"or(name.firstName[ilike]:%{query}%,name.lastName[ilike]:%{query}%,emails.primaryEmail[ilike]:%{query}%)"
    return await _req("GET", "/people", params=params)


@mcp.tool()
async def create_person(
    first_name: str,
    last_name: str = "",
    email: str = "",
    phone: str = "",
    job_title: str = "",
    company_id: str = "",
) -> Any:
    """Create a contact. `email` is the natural key — pass it when known."""
    body: dict[str, Any] = {"name": {"firstName": first_name, "lastName": last_name}}
    if email:
        body["emails"] = {"primaryEmail": email}
    if phone:
        body["phones"] = {"primaryPhoneNumber": phone}
    if job_title:
        body["jobTitle"] = job_title
    if company_id:
        body["companyId"] = company_id
    return await _req("POST", "/people", json=body)


@mcp.tool()
async def find_companies(query: str = "", limit: int = 10) -> Any:
    """Search Companies by name. Empty query lists recent."""
    params = {"limit": max(1, min(limit, 60))}
    if query:
        params["filter"] = f"name[ilike]:%{query}%"
    return await _req("GET", "/companies", params=params)


@mcp.tool()
async def create_company(name: str, domain: str = "", employees: Optional[int] = None) -> Any:
    """Create a Company (e.g. a grant-giving foundation or VC fund)."""
    body: dict[str, Any] = {"name": name}
    if domain:
        body["domainName"] = {"primaryLinkUrl": domain}
    if employees is not None:
        body["employees"] = employees
    return await _req("POST", "/companies", json=body)


@mcp.tool()
async def create_note(title: str, body: str, person_id: str = "", company_id: str = "") -> Any:
    """Log a Note (e.g. a call summary or meeting). Optionally attach to a person/company."""
    note = await _req("POST", "/notes", json={"title": title, "body": body})
    note_id = (note.get("data", {}).get("createNote", {}) or note.get("data", {}) or {}).get("id") \
        or note.get("id")
    # Best-effort relate the note to a target via a note target row.
    if note_id and (person_id or company_id):
        target: dict[str, Any] = {"noteId": note_id}
        if person_id:
            target["personId"] = person_id
        if company_id:
            target["companyId"] = company_id
        try:
            await _req("POST", "/noteTargets", json=target)
        except Exception as e:  # relating is a convenience; don't lose the note
            return {"note": note, "warning": f"note created but not linked: {e}"}
    return note


@mcp.tool()
async def create_opportunity(
    name: str,
    amount: Optional[float] = None,
    stage: str = "NEW",
    company_id: str = "",
) -> Any:
    """Create an Opportunity (e.g. a grant application or funding round in the pipeline)."""
    body: dict[str, Any] = {"name": name, "stage": stage}
    if amount is not None:
        body["amount"] = {"amountMicros": int(amount * 1_000_000), "currencyCode": "CHF"}
    if company_id:
        body["companyId"] = company_id
    return await _req("POST", "/opportunities", json=body)


@mcp.tool()
async def list_opportunities(limit: int = 20) -> Any:
    """List Opportunities (the funding/grant pipeline)."""
    return await _req("GET", "/opportunities", params={"limit": max(1, min(limit, 60))})


if __name__ == "__main__":
    mcp.run()
