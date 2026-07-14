# Configuration

FreEco.ai reads `~/.openfang/config.toml` by default. `OPENFANG_HOME` changes
the base directory. Configuration structs use defaults, so omitted fields retain
safe project defaults.

## Essential settings

```toml
api_listen = "127.0.0.1:4200"
api_key = "choose-a-long-random-secret"

[default_model]
provider = "groq"
model = "llama-3.3-70b-versatile"
api_key_env = "GROQ_API_KEY"
```

Keep provider secrets in the environment named by `api_key_env`; do not commit
them to configuration files.

## Network exposure

Keep `api_listen` on loopback unless remote access is required. When binding to
a LAN or public interface, configure `api_key` and restrict the reverse proxy,
firewall, and allowed browser origins.

## Channels

Channel sections live under `[channels]`. Add only channels you use, set their
secret environment-variable names, and restrict `allowed_users` where supported.
See [Integrations](Integrations).
