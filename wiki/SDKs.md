# SDKs

FreEco.ai exposes HTTP, Server-Sent Events, WebSocket, MCP, and A2A interfaces.
There is no separate language-specific SDK requirement: any HTTP client can call
the REST API.

## HTTP

Send the API key with the standard HTTP authorization header or `X-API-Key` on
every operational request. Use JSON request bodies and inspect non-2xx
responses before retrying.

## Streaming

Use SSE for audit/log streams and WebSocket endpoints for interactive agent
streaming. Browser EventSource cannot attach custom headers, so configure it to
use the supported token flow only over a trusted local or HTTPS origin.

## Protocol integrations

MCP exposes tools to compatible model clients. A2A supports discovering and
communicating with external agents. Consult the repository API and protocol
documentation for endpoint schemas.
