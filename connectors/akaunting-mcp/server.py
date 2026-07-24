"""Akaunting — MCP server for FreEco.ai.

Wraps Akaunting's REST API (https://akaunting.com) as MCP tools so the
Bookkeeper agent can record income/expenses, read accounts, and raise invoices —
the auditable books a Swiss nonprofit needs for grant reporting.

Run:
    export AKAUNTING_URL="http://localhost:8080"
    export AKAUNTING_EMAIL="admin@company.com"
    export AKAUNTING_PASSWORD="<password>"
    export AKAUNTING_COMPANY_ID="1"
    pip install -r requirements.txt && python server.py

Connect in FreEco.ai → Settings → Tools → "Add another tool server"
    name: accounting   type: stdio   command: python /abs/path/to/server.py

Akaunting uses HTTP Basic auth and a required `company_id` query param on every
call. Amounts are plain decimals in the company currency.
"""

import os
from typing import Any, Optional

import httpx
from fastmcp import FastMCP

BASE = os.environ.get("AKAUNTING_URL", "http://localhost:8080").rstrip("/")
EMAIL = os.environ.get("AKAUNTING_EMAIL", "")
PASSWORD = os.environ.get("AKAUNTING_PASSWORD", "")
COMPANY_ID = os.environ.get("AKAUNTING_COMPANY_ID", "1")
API = f"{BASE}/api"

mcp = FastMCP(
    "akaunting",
    instructions=(
        "Akaunting bookkeeping. Record income and expenses as transactions, list "
        "accounts and categories, and raise invoices. Every amount is in the "
        "company's base currency. Confirm the account and category before posting."
    ),
)


def _auth() -> httpx.BasicAuth:
    if not (EMAIL and PASSWORD):
        raise RuntimeError("AKAUNTING_EMAIL / AKAUNTING_PASSWORD not set")
    return httpx.BasicAuth(EMAIL, PASSWORD)


async def _req(method: str, path: str, params: Optional[dict] = None, json: Any = None) -> Any:
    params = dict(params or {})
    params.setdefault("company_id", COMPANY_ID)
    async with httpx.AsyncClient(timeout=30, auth=_auth()) as client:
        r = await client.request(
            method, f"{API}{path}", params=params, json=json,
            headers={"Accept": "application/json", "Content-Type": "application/json"},
        )
        if r.status_code >= 400:
            raise RuntimeError(f"Akaunting API {r.status_code}: {r.text[:500]}")
        return r.json() if r.text else {}


@mcp.tool()
async def list_accounts() -> Any:
    """List bank/cash accounts (needed to post a transaction)."""
    return await _req("GET", "/accounts")


@mcp.tool()
async def list_categories(category_type: str = "") -> Any:
    """List categories. `category_type` = 'income' | 'expense' | '' (all)."""
    params = {"search": f"type:{category_type}"} if category_type else {}
    return await _req("GET", "/categories", params=params)


@mcp.tool()
async def record_transaction(
    type: str,
    amount: float,
    account_id: int,
    category_id: int,
    paid_at: str,
    description: str = "",
    contact_id: Optional[int] = None,
    currency_code: str = "CHF",
) -> Any:
    """Record a transaction. `type` = 'income' or 'expense'. `paid_at` = 'YYYY-MM-DD'."""
    if type not in ("income", "expense"):
        raise RuntimeError("type must be 'income' or 'expense'")
    body: dict[str, Any] = {
        "type": type,
        "amount": amount,
        "account_id": account_id,
        "category_id": category_id,
        "paid_at": paid_at,
        "currency_code": currency_code,
        "currency_rate": 1,
        "description": description,
    }
    if contact_id is not None:
        body["contact_id"] = contact_id
    return await _req("POST", "/transactions", json=body)


@mcp.tool()
async def list_transactions(limit: int = 25) -> Any:
    """List recent transactions (income + expenses)."""
    return await _req("GET", "/transactions", params={"limit": max(1, min(limit, 100))})


@mcp.tool()
async def financial_summary() -> Any:
    """Rough income/expense totals from recent transactions, for a quick status."""
    data = await _req("GET", "/transactions", params={"limit": 100})
    rows = data.get("data", data) if isinstance(data, dict) else data
    income = expense = 0.0
    for t in rows or []:
        amt = float(t.get("amount", 0) or 0)
        if t.get("type") == "income":
            income += amt
        elif t.get("type") == "expense":
            expense += amt
    return {"income": income, "expense": expense, "net": income - expense, "counted": len(rows or [])}


if __name__ == "__main__":
    mcp.run()
