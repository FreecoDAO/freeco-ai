# FAQ

## Where is configuration stored?

In `~/.openfang/config.toml` by default, or beneath `OPENFANG_HOME`.

## Is the dashboard public?

No. Keep the daemon on loopback by default. Configure an API key before exposing
the dashboard or operational API to another machine.

## Why does an SSE request return 401?

The log stream is operational data and requires authentication. Configure the
client with the API credential; browser EventSource clients use the supported
token flow.

## Can I use a local model?

Yes. Configure a local provider such as Ollama and its local base URL. See
[Configuration](Configuration).

## Is the WhatsApp gateway internet-facing?

No. It binds to loopback and requires a daemon-generated local token. Do not
publish port 3009.

## Where can I report a vulnerability?

Follow the contact and disclosure process in the repository's `SECURITY.md`.
