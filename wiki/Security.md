# Security

FreEco.ai uses defense in depth: capability checks, sandboxing, rate limiting,
security headers, session protection, audit records, path validation, and
authenticated peer communication.

## API access

Operational API endpoints require credentials when an API key is configured.
Use the standard HTTP authorization or `X-API-Key` header. SSE clients may
supply the same credential through their supported token mechanism. Keep API
keys out of URLs except where an EventSource client cannot set headers.

## WhatsApp gateway

The optional WhatsApp Web gateway listens only on loopback. The daemon generates
a local token for it, and the gateway accepts requests only with that token. Its
browser CORS response is restricted to the daemon's configured local origin.

## Operator guidance

- Bind the daemon to loopback by default.
- Use a strong, unique API key for exposed deployments.
- Store secrets in environment variables or a secret manager.
- Review audit records and keep dependencies current.
- Report vulnerabilities using the repository's `SECURITY.md` policy.
