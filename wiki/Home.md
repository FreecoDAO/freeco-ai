# FreEco.ai

FreEco.ai is a local-first agent operating system written in Rust. It runs agents,
workflows, memory, tools, channels, and a browser dashboard from one installation.

## Start here

- [Installation](Installation)
- [Configuration](Configuration)
- [Architecture](Architecture)
- [Security](Security)
- [Operations](Operations)
- [Development and testing](Development-and-testing)
- [SDKs](SDKs)
- [Integrations](Integrations)
- [FAQ](FAQ)
- [Roadmap](Roadmap)

## What it provides

- A local daemon and dashboard API.
- Agent lifecycle management, memory, scheduled work, workflows, and approvals.
- Multiple model providers, including local-provider support.
- Channel adapters and MCP/A2A interoperability.
- Capability controls, audit records, rate limiting, and authenticated APIs.

FreEco.ai is intended to be operated locally by default. Exposing it beyond the
local machine requires deliberate network, origin, and credential configuration.
