# FreEco.ai connectors (MCP)

Small MCP servers that put the Association's data where it belongs — **CRM,
accounting, and institutional memory** — so agents can read and write it.
They wrap open-source tools you already have in the FreecoDAO org.

| Connector | Wraps | Gives agents |
|-----------|-------|--------------|
| `twenty-crm-mcp`  | [Twenty](https://github.com/FreecoDAO/twenty) CRM | people, companies, notes, opportunities (donors / grant givers / VCs) |
| `akaunting-mcp`   | [Akaunting](https://github.com/FreecoDAO/akaunting) | income/expense transactions, accounts, invoices, financial summary |
| `graphify-mcp`    | [Graphify](https://github.com/FreecoDAO/graphify) + a docs folder | searchable institutional memory (grant history, policies, past applications) |

## How they fit together
CRM, accounting and memory are the **data layer**. An agent flow looks like:
Grant Scout → finds a funder → **CRM** `create_company` + `create_opportunity`;
Relationship Manager → logs a call → **CRM** `create_note`; App Writer → pulls
past applications from **memory** `search_docs`; Bookkeeper → records the grant →
**accounting** `record_transaction`.

## Run one
Each connector is a standalone [FastMCP](https://github.com/jlowin/fastmcp)
server. Set its env vars (see the top of each `server.py`), install deps, run it,
then connect it in **FreEco.ai → Settings → Tools → Add another tool server**
(`stdio`, `command: python /abs/path/to/server.py`). Restart FreEco.ai to
activate the tools.

> Security: these connectors hold real people's data. Run them locally, keep
> API keys in env vars (not in config), and complete the security review in
> `docs/security-substantiation.md` before onboarding real donor/grant records.
