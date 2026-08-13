# Docker sandbox boundaries

`docker_exec` has two intentionally different operating modes:

| Mode | Configuration | Workspace | Container lifetime |
| --- | --- | --- | --- |
| Ephemeral (default) | `scope = "session"` | Read-only | Destroyed after every command |
| Per-agent development | `scope = "agent"` plus explicit authorization | Read/write, only for its authorized agent | Reused until idle or maximum age cleanup |

The default is an ephemeral, read-only sandbox. It has no network unless
explicitly configured otherwise, drops all Linux capabilities, prevents
privilege escalation, limits resources, and is removed after its command.

## Persistent development workspaces

Persistent development workspaces are for trusted development agents, not
untrusted execution. They require all of the following:

```toml
[docker]
enabled = true
mode = "all"
scope = "agent"
image = "registry.example/freeco-sandbox@sha256:<64 hexadecimal digest>"
persistent_workspace = true
persistent_workspace_agents = ["<agent-id>"]
idle_timeout_secs = 86400
max_age_secs = 604800
```

The image must be pinned by SHA-256 digest in any reusable mode. Agent scope
keys the container pool by agent ID and canonical workspace path, so one agent
cannot reuse another agent's container. `scope = "shared"` is rejected for
agent execution because a shared container is not a valid tenant isolation
boundary.

Only an agent listed in `persistent_workspace_agents` receives a writable
mount. All other agents remain read-only even when persistence is enabled.
The underlying workspace is the durable storage; container lifetime is bounded
by `idle_timeout_secs` and `max_age_secs`.

On Unix, the sandbox process runs as the workspace owner. This keeps private
(`0700`) workspaces usable while all Linux capabilities remain dropped.

## SaaS deployment boundary

This local Docker integration is **not** a multi-tenant SaaS execution
architecture. A hosted deployment must use a separate authenticated control
plane and tenant-bound worker fleet. Each worker needs its own non-host volume,
identity-bound authorization, CPU/memory/PID/storage quotas, TTL cleanup,
audited egress policy, and network controls. Never mount the host filesystem
or Docker socket into tenant workers, and never reuse a worker or volume across
tenants.

Production sandbox images must be digest-pinned, signed, and accompanied by
SBOM/provenance and vulnerability scanning in the image-release pipeline. The
release workflow checks the Dockerfile, emits BuildKit SBOM and provenance
attestations, and blocks a release image with fixable high or critical Trivy
findings.
