# Operations

## Start and stop

Use `openfang start` to run the daemon. The health endpoint is:

```text
GET /api/health
```

Use the CLI or your service manager to stop it cleanly so agent state is
persisted.

## Monitoring

Check health, status, configured agents, budgets, and audit records through the
authenticated API. The dashboard log page uses an authenticated SSE stream and
falls back to authenticated polling.

## Backups

Stop the daemon or use a consistent filesystem snapshot before backing up
`~/.openfang/`, especially its SQLite data and agent state. Test restoration on
a non-production profile before relying on a backup.

## Incident response

Rotate the API key and affected provider/channel credentials, stop exposed
services, preserve audit data, and inspect firewall, reverse-proxy, and channel
configuration before restarting.
