# Integrations

## Model providers

Configure the default model with a provider name, model ID, and environment
variable containing its credential. Local providers can be used when privacy or
offline operation is required.

## Channels

FreEco.ai includes adapters for common chat, collaboration, social, and
self-hosted services. Configure only the channels needed for your deployment and
apply user allowlists and channel-specific policies.

## WhatsApp

WhatsApp supports Cloud API credentials and an optional Web/QR gateway. The
Web gateway requires Node.js 18+, remains loopback-only, and is authenticated by
a daemon-generated local token. Do not expose its port through a proxy or
container port mapping.

## MCP and A2A

Use MCP for tool interoperability and A2A for external-agent discovery and task
exchange. Treat external endpoints as untrusted and scope agent capabilities
before enabling them.
