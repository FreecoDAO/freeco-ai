# FreEco.ai demo stack

FreEco.ai plus the three business services it integrates — Twenty (CRM),
Akaunting (accounting) and dograh (real-time voice, MCP-native) — in one
command. Intended as the investor demo and as the base image for FreEco.ai
as a SaaS.

```bash
cd deploy/demo
cp .env.example .env      # then fill in the passwords
docker compose up -d
```

| Service | URL | What it is |
|---|---|---|
| FreEco.ai | `:4200` | agent OS, dashboard, assistant |
| Twenty | `:3200` | CRM |
| Akaunting | `:8080` | accounting |
| dograh | `:8000` | real-time voice |

dograh ships its own compose in its repo, so it is brought up alongside
rather than duplicated here:

```bash
git clone https://github.com/FreecoDAO/dograh
cd dograh && docker compose up -d
```

Its `ui` service publishes **3000**, which is why Twenty is mapped to 3200
even though it listens on 3000 internally. Installing voice and CRM on one
host otherwise collides and the second one silently fails to bind.

---

## Namecheap: shared hosting will not run this

**This stack cannot run on Namecheap Shared Hosting or EasyWP, and no amount
of configuration will change that.** It is worth being blunt about, because
buying the wrong plan for an investor demo is an expensive way to find out:

- Shared hosting is **cPanel with no root and no Docker daemon**. Every
  service here is a container.
- FreEco.ai is a **long-running Rust binary holding a port**. Shared hosting
  runs short-lived CGI/PHP requests and kills background processes.
- The agent sandbox **starts containers of its own**. That needs a Docker
  socket, which shared hosting does not expose to customers.
- The stack wants several GB of RAM. Shared plans do not allocate it.

### What does work

A **VPS or dedicated server with root access**, from Namecheap or anywhere
else. Namecheap sells both; check their current tiers and prices rather than
trusting a number written here, and size against:

| Resource | Minimum | Comfortable |
|---|---|---|
| RAM | 8 GB | 16 GB |
| vCPU | 4 | 8 |
| Disk | 60 GB SSD | 120 GB SSD |
| OS | Ubuntu 22.04 / 24.04 LTS | same |

Two Postgres/MariaDB instances, Twenty, Akaunting, FreEco.ai and dograh add
up; 4 GB will thrash and the demo will be judged on the lag.

**You can keep the domain at Namecheap regardless.** Registration and DNS are
separate from hosting — point an `A` record at whatever VPS you use, and
`freeco.ai` works normally even if the server is elsewhere.

### Setup on a fresh Ubuntu VPS

```bash
# 1. Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER && newgrp docker

# 2. The stack
git clone https://github.com/FreecoDAO/freeco-ai
cd freeco-ai/deploy/demo
cp .env.example .env
openssl rand -base64 24        # run per secret, paste into .env
docker compose up -d
```

Then in Namecheap's DNS panel, point the records at the server:

```
A     @      <server-ip>
A     www    <server-ip>
A     crm    <server-ip>
A     books  <server-ip>
```

### Put TLS in front before anyone sees it

The apps above serve plain HTTP on their own ports. Do not demo that: logins
would cross the internet in the clear, and the browser will say so in the
address bar while an investor is watching.

```bash
sudo apt install -y caddy
```

`/etc/caddy/Caddyfile`:

```
freeco.ai, www.freeco.ai {
    reverse_proxy localhost:4200
}
crm.freeco.ai {
    reverse_proxy localhost:3200
}
books.freeco.ai {
    reverse_proxy localhost:8080
}
```

```bash
sudo systemctl reload caddy
```

Caddy obtains and renews Let's Encrypt certificates automatically, so this is
the shortest path to HTTPS on all three.

Then set the public URLs in `.env` and restart — Twenty and Akaunting build
links from them, and left at `localhost` every login redirect breaks for
anyone not sitting at the server:

```
TWENTY_URL=https://crm.freeco.ai
AKAUNTING_URL=https://books.freeco.ai
```

### Before the demo

- **Close the database ports.** `ufw allow 22,80,443/tcp` and nothing else;
  the compose file publishes only the app ports, but check with `ss -tlnp`.
- **Change every default in the apps themselves.** Twenty and Akaunting each
  create an admin account on first run.
- **Take a snapshot** once the demo data is loaded, so a bad click during the
  meeting is a rollback rather than an incident.
