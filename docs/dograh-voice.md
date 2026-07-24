# Real-time voice with dograh (over MCP)

FreEco.ai speaks and listens on its own (Freeco's hold-to-talk mic + spoken
replies). For **real-time, phone-grade voice** — live calls, barge-in, premium/
cloned voices, and voice-agent *workflows* for sales, marketing, and nonprofit
outreach — connect **[dograh](https://github.com/FreecoDAO/dograh)**, an open
voice-AI platform (self-hosted Vapi/Retell alternative). dograh exposes an
**MCP server**, and FreEco.ai is an MCP client, so the two connect cleanly.

Once connected, Freeco and your agents gain dograh's tools: create/save voice
workflows, browse the node catalog, read the voice-prompting guide, and search
dograh docs — i.e., agents can *build and run voice campaigns themselves*.

## 1. Run dograh (Docker)

```bash
git clone https://github.com/FreecoDAO/dograh
cd dograh
docker compose up -d
```

This starts the dograh API (with its MCP server) on **port 8000**, plus its UI,
Postgres, Redis, MinIO and a TURN server for WebRTC audio. See dograh's own
README for BYOK model/voice configuration (STT/TTS/LLM or speech-to-speech).

## 2. Connect it in FreEco.ai

Open **Settings → Tools → Voice AI & tool servers (MCP)**, confirm the URL
(default `http://localhost:8000/mcp`), and click **Connect dograh voice**.
That writes an MCP server entry to `~/.openfang/config.toml`:

```toml
[[mcp_servers]]
name = "dograh"
[mcp_servers.transport]
type = "http"
url = "http://localhost:8000/mcp"
```

**Restart FreEco.ai** — MCP connections are established at boot. After restart,
dograh's tools appear in Settings → Tools and are callable by your agents.

## 3. Use it

Ask Freeco: *"Using dograh, build a voice campaign that calls this list of grant
givers, introduces our nonprofit, and books a follow-up."* Freeco delegates to
the dograh tools to create the workflow, and dograh runs the calls.

> Security: `mcp_servers` is a trusted connection — only connect a dograh
> instance you run. The dograh API on :8000 should be bound to localhost or
> firewalled; do not expose it publicly without auth.
