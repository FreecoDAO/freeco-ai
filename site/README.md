# freeco.ai — the public site

One file: `index.html`. No build step, no framework, no external requests —
every style is inline and every font falls back to one already on the
visitor's machine.

That last part is deliberate rather than lazy. A product that blocks cloud
voices on privacy grounds should not leak a pageview to a font CDN on every
visit, and a page with zero external requests also renders instantly on the
conference-room wifi where a demo actually happens.

## Deploying to Namecheap shared hosting

**The site runs fine on shared hosting.** It is static HTML — the restriction
that stops the *application* stack (no Docker, no root, no long-running
processes) does not apply to a single page.

1. cPanel → **File Manager** → `public_html`
2. Upload `index.html`
3. Done. `https://freeco.ai` serves it.

Enable **AutoSSL** in cPanel for HTTPS. It is free and renews itself.

### Keeping the app on a different server

DNS is independent of hosting, so the domain can stay at Namecheap while the
application runs anywhere:

```
A     @        <shared-hosting-ip>     # this page
A     app      <vps-ip>                # FreEco.ai dashboard
A     crm      <vps-ip>                # Twenty
A     books    <vps-ip>                # Akaunting
```

`app.freeco.ai` then reaches the stack from `deploy/demo`, while `freeco.ai`
serves this page from shared hosting. Full VPS setup, sizing and TLS are in
[`deploy/demo/README.md`](../deploy/demo/README.md).

## Editing

The comparison table is plain `<table>` markup in `index.html`. Cell classes
carry the meaning:

| Class | Renders as |
|---|---|
| `yes` | green, bold |
| `part` | amber — partial or DIY |
| `no` | grey, dimmed |
| `own` | highlights the FreEco.ai column |

Keep the "What FreEco.ai does not do" section honest when the table changes.
An investor who has seen a hundred comparison tables will trust the ones that
admit limits and discount the ones that do not, so the section earns more than
it costs.
