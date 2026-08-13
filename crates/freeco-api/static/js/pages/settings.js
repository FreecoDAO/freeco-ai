// FreEco.ai Settings Page — Provider Hub, Model Catalog, Config, Tools + Security, Network, Migration tabs
'use strict';

function settingsPage() {
  return {
    tab: 'providers',
    // People / accounts (RBAC)
    users: [],
    adminUsername: 'admin',
    newUser: { name: '', role: 'kid', password: '' },
    usersLoadError: '',
    usersMsg: '',
    usersBusy: false,
    sysInfo: {},
    usageData: [],
    tools: [],
    config: {},
    providers: [],
    models: [],
    settingAgents: [],
    settingWorkflows: [],
    toolSearch: '',
    modelSearch: '',
    modelProviderFilter: '',
    modelTierFilter: '',
    showCustomModelForm: false,
    customModelId: '',
    customModelProvider: 'openrouter',
    customModelContext: 128000,
    customModelMaxOutput: 8192,
    customModelStatus: '',
    providerKeyInputs: {},
    providerUrlInputs: {},
    providerUrlSaving: {},
    providerTesting: {},
    providerTestResults: {},
    providerSearch: '',
    providerStatusFilter: '',
    providerCategoryFilter: '',
    copilotOAuth: { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 },
    customProviderName: '',
    customProviderUrl: '',
    customProviderKey: '',
    customProviderStatus: '',
    addingCustomProvider: false,
    loading: true,
    loadError: '',

    // -- One-click services (dograh voice, CRM, accounting) --
    services: [],
    servicesLoaded: false,
    dockerReady: false,
    svc: { phase: 'idle', detail: '', percent: -1, service: '', running: false },
    svcPoll: null,

    async loadServices() {
      try {
        var data = await FreecoAPI.get('/api/services');
        this.services = data.services || [];
        this.dockerReady = !!data.docker_ready;
        this.servicesLoaded = true;
      } catch(e) {
        this.servicesLoaded = true;
        FreecoToast.error('Could not load services: ' + (e.message || 'error'));
      }
      // If an install is already running (e.g. after a page reload), resume polling.
      try {
        var st = await FreecoAPI.get('/api/services/status');
        this.svc = st;
        if (st.running) this.pollServices();
      } catch(e) { /* silent */ }
    },

    async installService(id) {
      if (this.svc.running) { FreecoToast.info('Another install is already running.'); return; }
      try {
        await FreecoAPI.post('/api/services/' + encodeURIComponent(id) + '/install', {});
        this.svc = { phase: 'checking', detail: 'Starting…', percent: -1, service: id, running: true };
        FreecoToast.success('Install started — this downloads a few GB, keep the app open.');
        this.pollServices();
      } catch(e) {
        FreecoToast.error('Install failed to start: ' + (e.message || 'error'));
      }
    },

    pollServices() {
      var self = this;
      if (this.svcPoll) clearInterval(this.svcPoll);
      this.svcPoll = setInterval(async function() {
        try { self.svc = await FreecoAPI.get('/api/services/status'); } catch(e) { return; }
        if (!self.svc.running) {
          clearInterval(self.svcPoll);
          self.svcPoll = null;
          if (self.svc.phase === 'done') {
            FreecoToast.success(self.svc.detail || 'Service is ready.');
          } else if (self.svc.phase === 'error') {
            FreecoToast.error(self.svc.detail || 'Service install failed.');
          } else if (self.svc.phase === 'needs-docker') {
            FreecoToast.warn ? FreecoToast.warn(self.svc.detail) : FreecoToast.error(self.svc.detail);
          }
          await self.loadServices(); // refresh running/connected badges
        }
      }, 2500);
    },

    // -- Integrations (extension/plugin subsystem) --
    installedIntegrations: [],
    availableIntegrations: [],
    integrationsLoaded: false,
    intBusy: false,

    async loadIntegrations() {
      try {
        var results = await Promise.all([
          FreecoAPI.get('/api/integrations'),
          FreecoAPI.get('/api/integrations/available')
        ]);
        this.installedIntegrations = (results[0] && results[0].installed) || [];
        this.availableIntegrations = (results[1] && results[1].integrations) || [];
        this.integrationsLoaded = true;
      } catch (e) {
        this.integrationsLoaded = true;
        FreecoToast.error('Could not load integrations: ' + (e.message || 'error'));
      }
    },

    async addIntegration(id) {
      this.intBusy = true;
      try {
        var r = await FreecoAPI.post('/api/integrations/add', { id: id });
        FreecoToast.success((r && r.message) || 'Integration installed.');
        await this.loadIntegrations();
      } catch (e) {
        FreecoToast.error('Install failed: ' + (e.message || 'error'));
      }
      this.intBusy = false;
    },

    removeIntegration(id) {
      var self = this;
      FreecoToast.confirm('Remove integration', 'Remove "' + id + '"? Its tools will no longer be available to agents.', async function() {
        self.intBusy = true;
        try {
          await FreecoAPI.del('/api/integrations/' + encodeURIComponent(id));
          FreecoToast.success('Removed.');
          await self.loadIntegrations();
        } catch (e) {
          FreecoToast.error('Remove failed: ' + (e.message || 'error'));
        }
        self.intBusy = false;
      });
    },

    async reconnectIntegration(id) {
      this.intBusy = true;
      try {
        await FreecoAPI.post('/api/integrations/' + encodeURIComponent(id) + '/reconnect', {});
        FreecoToast.success('Reconnected.');
        await this.loadIntegrations();
      } catch (e) {
        FreecoToast.error('Reconnect failed: ' + (e.message || 'error'));
      }
      this.intBusy = false;
    },

    // -- Devices (mobile pairing) --
    devices: [],
    devicesError: '',
    pairing: { token: '', qr_uri: '', qr_img: '', expires_at: '' },
    pairingBusy: false,

    async loadDevices() {
      this.devicesError = '';
      try {
        var data = await FreecoAPI.get('/api/pairing/devices');
        this.devices = (data && data.devices) || [];
      } catch (e) {
        this.devices = [];
        this.devicesError = (e.message && e.message.indexOf('not enabled') >= 0)
          ? 'Device pairing is not enabled on this server.'
          : ('Could not load devices: ' + (e.message || 'error'));
      }
    },

    async requestPairing() {
      if (this.pairingBusy) return;
      this.pairingBusy = true;
      try {
        var res = await FreecoAPI.post('/api/pairing/request', {});
        this.pairing = {
          token: res.token || '',
          qr_uri: res.qr_uri || '',
          // Only treat qr_uri as an image if the server already returned a data image.
          qr_img: (res.qr_uri && res.qr_uri.indexOf('data:image') === 0) ? res.qr_uri : '',
          expires_at: res.expires_at || ''
        };
      } catch (e) {
        FreecoToast.error((e.message && e.message.indexOf('not enabled') >= 0)
          ? 'Device pairing is not enabled on this server.'
          : ('Could not start pairing: ' + (e.message || 'error')));
      }
      this.pairingBusy = false;
    },

    unpairDevice(id) {
      var self = this;
      FreecoToast.confirm('Unpair device', 'Remove this device? It will lose access to your agents.', async function() {
        try {
          await FreecoAPI.del('/api/pairing/devices/' + encodeURIComponent(id));
          await self.loadDevices();
        } catch (e) { FreecoToast.error(e.message || 'Could not unpair'); }
      });
    },

    // -- Local AI (Ollama) setup --
    localAi: { phase: 'idle', detail: '', percent: -1, running: false, ollama_detected: false },
    localAiRecommendation: null,
    localAiPoll: null,

    async refreshLocalAi() {
      try { this.localAi = await FreecoAPI.get('/api/local-ai/status'); } catch(e) { /* silent */ }
    },

    // -- MCP / voice AI (dograh) connectors --
    mcpServers: [],
    dographUrl: 'http://localhost:8000/mcp',
    mcpBusy: false,
    mcpMsg: '',

    async loadMcpServers() {
      try {
        var data = await FreecoAPI.get('/api/mcp/servers');
        var configured = (data && data.configured) || [];
        var connected = (data && data.connected) || [];
        var byName = {};
        connected.forEach(function(c) { byName[c.name] = c; });
        this.mcpServers = configured.map(function(s) {
          var live = byName[s.name];
          return {
            name: s.name,
            transport: s.transport || {},
            connected: !!live,
            tool_count: live ? (live.tools_count || 0) : 0
          };
        });
      } catch (e) { this.mcpServers = []; }
    },

    newMcp: { name: '', type: 'stdio', value: '' },

    async addCustomMcp() {
      if (this.mcpBusy) return;
      var m = this.newMcp;
      var name = (m.name || '').trim();
      var value = (m.value || '').trim();
      if (!name || !value) { FreecoToast.error('Enter a name and a URL/command'); return; }
      var transport;
      if (m.type === 'http') {
        transport = { type: 'http', url: value };
      } else {
        var parts = value.split(/\s+/);
        transport = { type: 'stdio', command: parts[0], args: parts.slice(1) };
      }
      this.mcpBusy = true; this.mcpMsg = '';
      try {
        var res = await FreecoAPI.post('/api/mcp/servers', { name: name, transport: transport });
        this.mcpMsg = res.message || ('Connected "' + name + '". Restart to activate.');
        this.newMcp = { name: '', type: 'stdio', value: '' };
        await this.loadMcpServers();
      } catch (e) {
        FreecoToast.error('Could not connect: ' + (e.message || 'error'));
      }
      this.mcpBusy = false;
    },

    async connectDograh() {
      if (this.mcpBusy) return;
      var url = (this.dographUrl || '').trim();
      if (!url) { FreecoToast.error('Enter the dograh MCP URL'); return; }
      this.mcpBusy = true; this.mcpMsg = '';
      try {
        var res = await FreecoAPI.post('/api/mcp/servers', {
          name: 'dograh', transport: { type: 'http', url: url }
        });
        this.mcpMsg = res.message || 'Connected. Restart FreEco.ai to activate dograh voice tools.';
        await this.loadMcpServers();
      } catch (e) {
        FreecoToast.error('Could not connect dograh: ' + (e.message || 'error'));
      }
      this.mcpBusy = false;
    },

    removeMcpServer(name) {
      var self = this;
      FreecoToast.confirm('Remove server', 'Disconnect MCP server "' + name + '"? Takes effect after a restart.', async function() {
        try {
          await FreecoAPI.del('/api/mcp/servers/' + encodeURIComponent(name));
          self.mcpMsg = 'Removed "' + name + '". Restart to apply.';
          await self.loadMcpServers();
        } catch (e) { FreecoToast.error(e.message || 'Could not remove'); }
      });
    },

    // One-click model policy: everyday work on a free local Gemma, hard tasks
    // on the strongest cloud model whose key is configured.
    autoConfigBusy: false,
    autoConfigResult: null,

    async autoConfigureModels() {
      if (this.autoConfigBusy) return;
      this.autoConfigBusy = true;
      try {
        var res = await FreecoAPI.post('/api/models/autoconfig', {});
        this.autoConfigResult = res;
        FreecoToast.success(res.message || 'Models configured.');
      } catch(e) {
        FreecoToast.error('Auto-configure failed: ' + (e.message || 'unknown error'));
      }
      this.autoConfigBusy = false;
    },

    showModelPicker: false,

    // Load hardware + recommendation once when the card opens, so the one-click
    // path can show exactly which model it will install.
    async initLocalAi() {
      await this.refreshLocalAi();
      if (!this.localAiRecommendation) {
        try {
          this.localAiRecommendation = await FreecoAPI.get('/api/local-ai/recommendation?purpose=general');
        } catch(e) { /* card still works without it */ }
      }
    },

    async loadLocalAiRecommendation(purpose) {
      try {
        this.localAiRecommendation = await FreecoAPI.get('/api/local-ai/recommendation?purpose=' + (purpose || 'general'));
      } catch(e) { FreecoToast.error('Could not inspect local AI hardware: ' + e.message); }
    },

    // Gemma 4 GGUF catalog served by llama.cpp (no Ollama).
    llamaCatalog: null,

    // Point every existing agent at the model now configured as the default.
    // Agents store their own model, so without this they keep whatever they
    // were created with - which is why "I couldn't reach a language model"
    // survives a successful local-AI install.
    async switchAllAgentsToLocal() {
      var status = null;
      try { status = await FreecoAPI.get('/api/status'); } catch (e) { return 0; }
      var target = status && status.default_model;
      if (!target) return 0;
      var agents = [];
      try { agents = await FreecoAPI.get('/api/agents'); } catch (e) { return 0; }
      var switched = 0;
      for (var i = 0; i < agents.length; i++) {
        if (agents[i].model_name === target) continue;
        try {
          await FreecoAPI.put('/api/agents/' + agents[i].id + '/model', { model: target });
          switched++;
        } catch (e) { /* keep going; report the total */ }
      }
      if (switched) {
        FreecoToast.success(switched + ' agent(s) now use ' + target + '.');
        try { await Alpine.store('app').refreshAgents(); } catch (e) { /* optional */ }
      }
      return switched;
    },

    // -- Isolated Linux sandbox --
    sandbox: { loaded: false, ready: false, enabled: false, docker_running: false,
               image_present: false, next_step: '', protections: null, pulling: false },

    async loadSandbox(force) {
      if (this.sandbox.loaded && !force) return this.sandbox;
      try {
        var s = await FreecoAPI.get('/api/sandbox/status');
        s.loaded = true;
        s.pulling = false;
        this.sandbox = s;
      } catch (e) {
        // Report the failure rather than leaving the panel looking healthy.
        this.sandbox = { loaded: true, ready: false, enabled: false,
                         docker_running: false, image_present: false,
                         next_step: 'Could not read sandbox status: ' + e.message,
                         protections: null, pulling: false };
      }
      return this.sandbox;
    },

    // Explicit, never automatic: docker run would pull this silently on first
    // use, which on a metered connection is an unannounced download.
    async pullSandboxImage() {
      this.sandbox.pulling = true;
      try {
        var res = await FreecoAPI.post('/api/sandbox/pull', {});
        if (res && res.ok) {
          FreecoToast.success(res.message || 'Sandbox image downloaded.');
        } else {
          FreecoToast.error((res && res.error) || 'Could not download the sandbox image.');
        }
      } catch (e) {
        FreecoToast.error('Sandbox image: ' + e.message);
      }
      this.sandbox.pulling = false;
      await this.loadSandbox(true);
    },

    async loadLlamaCatalog() {
      try {
        this.llamaCatalog = await FreecoAPI.get('/api/local-ai/llama/catalog');
      } catch (e) { this.llamaCatalog = null; }
      return this.llamaCatalog;
    },

    // Path 1 - one click: download the RAM-sized Gemma 4, run it with
    // llama.cpp, and make it the model every agent uses.
    async oneClickLocalAi() {
      var cat = this.llamaCatalog || await this.loadLlamaCatalog();
      var pick = null;
      if (cat && cat.models) {
        // Best model this machine can actually run (largest that fits).
        var runnable = cat.models.filter(function (m) { return m.runnable; });
        pick = (runnable.length ? runnable : cat.models)[0];
      }
      await this.startLocalAiSetup(pick ? pick.id : 'gemma-4-e2b-qat-q4');
    },

    // Path 2 - pick a specific model from the compare list.
    async setupModel(id) {
      this.showModelPicker = false;
      await this.startLocalAiSetup(id);
    },

    // Warn BEFORE the download, not after. On a machine without a discrete
    // GPU a local model runs but takes about an hour per agent turn, so
    // pulling several GB first and discovering that afterwards wastes both
    // the user's bandwidth and their time. The check is honest about it
    // being a hardware limit rather than something a setting can fix.
    async confirmWeakHardware() {
      var cat = this.llamaCatalog || await this.loadLlamaCatalog();
      var cap = cat && cat.capability;
      if (!cap || cap.suitable) return true;
      var msg = cap.reason + '\n\n';
      if (cap.est_minutes_per_agent_turn) {
        msg += 'Estimated for THIS machine: about ' + cap.est_minutes_per_agent_turn +
               ' minutes for a single agent turn.\n\n';
      }
      // Answer "why not?" with "here is what would work". Without this the
      // warning is a dead end and the user has no way to judge what to buy
      // or which of their machines to run it on.
      if (cap.requirements && cap.requirements.length) {
        msg += 'What local AI needs to run well:\n';
        msg += cap.requirements.map(function (r) { return '  - ' + r; }).join('\n');
        msg += '\n\n';
      }
      msg += 'Set it up anyway? (Cancel keeps your current model.)';
      return window.confirm(msg);
    },

    async startLocalAiSetup(modelId) {
      if (!await this.confirmWeakHardware()) {
        FreecoToast.info('Local AI left off. Your current model stays the default.');
        return;
      }
      try {
        await FreecoAPI.post('/api/local-ai/llama/setup', { model_id: modelId });
        FreecoToast.success('Setting up ' + (modelId || 'local AI') + ' - this downloads a few GB and resumes if interrupted. Keep the app open.');
        this.pollLocalAi();
      } catch(e) {
        FreecoToast.error('Local AI setup: ' + e.message);
      }
    },

    pollLocalAi() {
      var self = this;
      if (this.localAiPoll) clearInterval(this.localAiPoll);
      this.localAiPoll = setInterval(async function() {
        await self.refreshLocalAi();
        if (!self.localAi.running) {
          clearInterval(self.localAiPoll);
          self.localAiPoll = null;
          if (self.localAi.phase === 'done') {
            // Existing agents keep their OWN model, so setting default_model
            // alone leaves them pointing at whatever they were created with
            // (often a model that was never installed). Switch them over too,
            // otherwise "set up local AI" appears to do nothing.
            try { await self.switchAllAgentsToLocal(); } catch(e) { /* non-fatal */ }
            try { await FreecoAPI.post('/api/config/reload', {}); } catch(e) { /* restart applies it */ }
            FreecoToast.success('Local AI is ready — restart FreEco.ai if agents still show the old model.');
          } else if (self.localAi.phase === 'error') {
            FreecoToast.error('Local AI setup failed: ' + self.localAi.detail);
          } else if (self.localAi.phase === 'needs-manual-install') {
            FreecoToast.info(self.localAi.detail || 'Follow the on-screen step to finish installing Ollama.');
          }
        }
      }, 2500);
    },

    // -- Software updates --
    updateChecking: false,
    updateBusy: false,
    updateStatus: '',
    // Shown as a real, clickable link when the in-app install is unavailable.
    // A URL announced only in a toast vanishes a few seconds later, which is
    // how "your browser blocked the popup" became a dead end.
    updateDownloadUrl: '',
    updateLatest: '',
    updateAvailable: false,
    updateUrl: 'https://github.com/FreecoDAO/freeco-ai/releases/latest',
    autoUpdateCheck: localStorage.getItem('freeco_auto_update_check') !== 'off',

    // -- Dynamic config state --
    configSchema: null,
    configValues: {},
    configDirty: {},
    configSaving: {},

    // -- Security state --
    securityData: null,
    secLoading: false,
    verifyingChain: false,
    chainResult: null,
    passwordInput: '',
    passwordConfirmation: '',
    currentPassword: '',
    passwordSaving: false,

    coreFeatures: [
      {
        name: 'Path Traversal Prevention', key: 'path_traversal',
        description: 'Blocks directory escape attacks (../) in all file operations. Two-phase validation: syntactic rejection of path components, then canonicalization to normalize symlinks.',
        threat: 'Directory escape, privilege escalation via symlinks',
        impl: 'host_functions.rs — safe_resolve_path() + safe_resolve_parent()'
      },
      {
        name: 'SSRF Protection', key: 'ssrf_protection',
        description: 'Blocks outbound requests to private IPs, localhost, and cloud metadata endpoints (AWS/GCP/Azure). Validates DNS resolution results to defeat rebinding attacks.',
        threat: 'Internal network reconnaissance, cloud credential theft',
        impl: 'host_functions.rs — is_ssrf_target() + is_private_ip()'
      },
      {
        name: 'Capability-Based Access Control', key: 'capability_system',
        description: 'Deny-by-default permission system. Every agent operation (file I/O, network, shell, memory, spawn) requires an explicit capability grant in the manifest.',
        threat: 'Unauthorized resource access, sandbox escape',
        impl: 'host_functions.rs — check_capability() on every host function'
      },
      {
        name: 'Privilege Escalation Prevention', key: 'privilege_escalation_prevention',
        description: 'When a parent agent spawns a child, the kernel enforces child capabilities are a subset of parent capabilities. No agent can grant rights it does not have.',
        threat: 'Capability escalation through agent spawning chains',
        impl: 'kernel_handle.rs — spawn_agent_checked()'
      },
      {
        name: 'Subprocess Environment Isolation', key: 'subprocess_isolation',
        description: 'Child processes (shell tools) inherit only a safe allow-list of environment variables. API keys, database passwords, and secrets are never leaked to subprocesses.',
        threat: 'Secret exfiltration via child process environment',
        impl: 'subprocess_sandbox.rs — env_clear() + SAFE_ENV_VARS'
      },
      {
        name: 'Security Headers', key: 'security_headers',
        description: 'Every HTTP response includes CSP, X-Frame-Options: DENY, X-Content-Type-Options: nosniff, Referrer-Policy, and X-XSS-Protection headers.',
        threat: 'XSS, clickjacking, MIME sniffing, content injection',
        impl: 'middleware.rs — security_headers()'
      },
      {
        name: 'Wire Protocol Authentication', key: 'wire_hmac_auth',
        description: 'Agent-to-agent OFP connections use HMAC-SHA256 mutual authentication with nonce-based handshake and constant-time signature comparison (subtle crate).',
        threat: 'Man-in-the-middle attacks on mesh network',
        impl: 'peer.rs — hmac_sign() + hmac_verify()'
      },
      {
        name: 'Request ID Tracking', key: 'request_id_tracking',
        description: 'Every API request receives a unique UUID (x-request-id header) and is logged with method, path, status code, and latency for full traceability.',
        threat: 'Untraceable actions, forensic blind spots',
        impl: 'middleware.rs — request_logging()'
      }
    ],

    configurableFeatures: [
      {
        name: 'API Rate Limiting', key: 'rate_limiter',
        description: 'GCRA (Generic Cell Rate Algorithm) with cost-aware tokens. Different endpoints cost different amounts — spawning an agent costs 50 tokens, health check costs 1.',
        configHint: 'Hard-coded: 500 tokens/minute per IP. Edit rate_limiter.rs to tune.',
        valueKey: 'rate_limiter'
      },
      {
        name: 'WebSocket Connection Limits', key: 'websocket_limits',
        description: 'Per-IP connection cap prevents connection exhaustion. Idle timeout closes abandoned connections. Message rate limiting prevents flooding.',
        configHint: 'Hard-coded: 5 connections/IP, 30min idle timeout, 64KB max message. Edit ws.rs to tune.',
        valueKey: 'websocket_limits'
      },
      {
        name: 'WASM Dual Metering', key: 'wasm_sandbox',
        description: 'WASM modules run with two independent resource limits: fuel metering (CPU instruction count) and epoch interruption (wall-clock timeout with watchdog thread).',
        configHint: 'Default: 1M fuel units, 30s timeout. Configurable per-agent via SandboxConfig.',
        valueKey: 'wasm_sandbox'
      },
      {
        name: 'Bearer Token Authentication', key: 'auth',
        description: 'All non-health endpoints require Authorization: Bearer header. When no API key is configured, all requests are restricted to localhost only.',
        configHint: 'Set api_key in ~/.freeco-ai/config.toml for remote access. Empty = localhost only.',
        valueKey: 'auth'
      }
    ],

    monitoringFeatures: [
      {
        name: 'Merkle Audit Trail', key: 'audit_trail',
        description: 'Every security-critical action is appended to an immutable, tamper-evident log. Each entry is cryptographically linked to the previous via SHA-256 hash chain.',
        configHint: 'Always active. Verify chain integrity from the Audit Log page.',
        valueKey: 'audit_trail'
      },
      {
        name: 'Information Flow Taint Tracking', key: 'taint_tracking',
        description: 'Labels data by provenance (ExternalNetwork, UserInput, PII, Secret, UntrustedAgent) and blocks unsafe flows: external data cannot reach shell_exec, secrets cannot reach network.',
        configHint: 'Always active. Prevents data flow attacks automatically.',
        valueKey: 'taint_tracking'
      },
      {
        name: 'Ed25519 Manifest Signing', key: 'manifest_signing',
        description: 'Agent manifests can be cryptographically signed with Ed25519. Verify manifest integrity before loading to prevent supply chain tampering.',
        configHint: 'Available for use. Sign manifests with ed25519-dalek for verification.',
        valueKey: 'manifest_signing'
      }
    ],

    // -- Peers state --
    peers: [],
    peersLoading: false,
    peersLoadError: '',
    _peerPollTimer: null,

    // -- Migration state --
    migStep: 'intro',
    detecting: false,
    scanning: false,
    migrating: false,
    sourcePath: '',
    targetPath: '',
    scanResult: null,
    migResult: null,

    // -- Settings load --
    async loadSettings() {
      this.loading = true;
      this.loadError = '';
      try {
        await Promise.all([
          this.loadSysInfo(),
          this.loadUsage(),
          this.loadTools(),
          this.loadConfig(),
          this.loadProviders(),
          this.loadModels(),
          this.loadAgents(),
          this.loadWorkflows()
        ]);
      } catch(e) {
        this.loadError = e.message || 'Could not load settings.';
      }
      this.loading = false;
      if (this.autoUpdateCheck) this.checkForUpdates();
    },

    async loadData() { return this.loadSettings(); },

    // -- Software updates --
    toggleAutoUpdateCheck() {
      this.autoUpdateCheck = !this.autoUpdateCheck;
      localStorage.setItem('freeco_auto_update_check', this.autoUpdateCheck ? 'on' : 'off');
      if (this.autoUpdateCheck) this.checkForUpdates();
    },

    // Returns true when `latest` is a newer semver than `current`.
    isNewerVersion(latest, current) {
      var l = String(latest).replace(/^v/, '').split('.').map(Number);
      var c = String(current).replace(/^v/, '').split('.').map(Number);
      for (var i = 0; i < Math.max(l.length, c.length); i++) {
        var a = l[i] || 0, b = c[i] || 0;
        if (a !== b) return a > b;
      }
      return false;
    },

    async checkForUpdates() {
      this.updateChecking = true;
      this.updateStatus = '';
      try {
        var res = await fetch('https://api.github.com/repos/FreecoDAO/freeco-ai/releases/latest', { headers: { Accept: 'application/vnd.github+json' } });
        if (!res.ok) throw new Error('GitHub API returned ' + res.status);
        var rel = await res.json();
        this.updateLatest = String(rel.tag_name || '').replace(/^v/, '');
        this.updateUrl = rel.html_url || this.updateUrl;
        var current = (this.sysInfo.version && this.sysInfo.version !== '-') ? this.sysInfo.version : (Alpine.store('app').version || '0.0.0');
        this.updateAvailable = this.isNewerVersion(this.updateLatest, current);
        Alpine.store('app').updateAvailable = this.updateAvailable;
        localStorage.setItem('freeco_update_last_check', String(Date.now()));
        this.updateStatus = this.updateAvailable
          ? 'Version ' + this.updateLatest + ' is available.'
          : 'You are on the latest version.';
      } catch (e) {
        // Distinguish "offline" from a real failure so the message isn't scary.
        var offline = (typeof navigator !== 'undefined' && navigator.onLine === false);
        this.updateStatus = offline
          ? "You're offline — can't check for updates right now. This is normal for a fully local install."
          : 'Update check failed: ' + (e.message || 'network error') + '. You can check manually on GitHub.';
      }
      this.updateChecking = false;
    },

    // Reach the desktop app's command bridge, or null in a plain browser.
    //
    // Tauri v2 exposes `invoke` at `window.__TAURI__.core.invoke`; v1 put it at
    // `window.__TAURI__.invoke`. Both are checked so this keeps working if the
    // desktop shell is upgraded, rather than silently reverting to "go download
    // it yourself" the way the previous check did.
    tauriInvoke() {
      if (typeof window === 'undefined' || !window.__TAURI__) return null;
      var t = window.__TAURI__;
      if (t.core && typeof t.core.invoke === 'function') return t.core.invoke;
      if (typeof t.invoke === 'function') return t.invoke;
      return null;
    },

    // Handle the "Get update" button. Never do nothing silently: on the desktop
    // app, run the built-in auto-updater (download + install + relaunch) with a
    // toast at every stage; in the browser/portable edition, open the download
    // page with clear feedback.
    async getUpdate() {
      if (this.updateBusy) return;
      this.updateBusy = true;
      // Call the desktop app's own commands rather than the updater plugin's
      // JavaScript API. This used to look for `window.__TAURI__.updater.check`,
      // which never existed: the plugin's JS bindings are not served with the
      // dashboard, so every desktop user silently fell through to the browser
      // branch and was told to go download an installer by hand. The Rust side
      // has always had `check_for_updates` and `install_update` registered --
      // nothing was calling them.
      var invoke = this.tauriInvoke();
      try {
        if (invoke) {
          // Desktop app: real download + install, no browser round trip.
          this.updateStatus = 'Checking for the update package...';
          var update = await invoke('check_for_updates');
          if (!update || !update.available) {
            this.updateStatus = 'You are already on the latest version.';
            FreecoToast.success('You are on the latest version.');
            return;
          }
          this.updateStatus = 'Downloading v' + (update.version || this.updateLatest) + '...';
          FreecoToast.info('Downloading the update — this can take a minute. Your data stays where it is.');
          await invoke('install_update');
          this.updateStatus = 'Update installed. Restarting FreEco.ai...';
          FreecoToast.success('Update installed — restarting.');
          return;
        }
        // Browser / portable / CLI: send them to the download page.
        var url = this.updateUrl || 'https://github.com/FreecoDAO/freeco-ai/releases/latest';
        // Show the link before trying to open it, not after failing to.
        // `window.open` is blocked by default in most browsers and in some
        // webviews, and the old code only revealed the URL inside a toast that
        // disappears -- leaving a dead end with nothing left to click.
        this.updateDownloadUrl = url;
        this.updateStatus = 'Download the installer, run it, and your data is kept where it is.';
        var win = window.open(url, '_blank', 'noopener');
        if (!win) {
          FreecoToast.warn('Your browser blocked the popup — use the download link shown above.');
        }
      } catch (e) {
        this.updateStatus = 'Update failed: ' + (e.message || 'unknown error');
        FreecoToast.error('Update failed: ' + (e.message || 'unknown error'));
      } finally {
        this.updateBusy = false;
      }
    },

    async loadSysInfo() {
      try {
        var ver = await FreecoAPI.get('/api/version');
        var status = await FreecoAPI.get('/api/status');
        this.sysInfo = {
          version: ver.version || '-',
          platform: ver.platform || '-',
          arch: ver.arch || '-',
          uptime_seconds: status.uptime_seconds || 0,
          agent_count: status.agent_count || 0,
          default_provider: status.default_provider || '-',
          default_model: status.default_model || '-'
        };
      } catch(e) { throw e; }
    },

    async loadUsage() {
      try {
        var data = await FreecoAPI.get('/api/usage');
        this.usageData = data.agents || [];
      } catch(e) { this.usageData = []; }
    },

    async loadTools() {
      try {
        var data = await FreecoAPI.get('/api/tools');
        this.tools = data.tools || [];
      } catch(e) { this.tools = []; }
    },

    async loadConfig() {
      try {
        this.config = await FreecoAPI.get('/api/config');
      } catch(e) { this.config = {}; }
    },

    async loadProviders() {
      try {
        var data = await FreecoAPI.get('/api/providers');
        this.providers = data.providers || [];
        for (var i = 0; i < this.providers.length; i++) {
          var p = this.providers[i];
          if (p.is_local) {
            if (!this.providerUrlInputs[p.id]) {
              this.providerUrlInputs[p.id] = p.base_url || '';
            }
            if (this.providerUrlSaving[p.id] === undefined) {
              this.providerUrlSaving[p.id] = false;
            }
          }
        }
      } catch(e) { this.providers = []; }
    },

    async loadModels() {
      try {
        var data = await FreecoAPI.get('/api/models');
        this.models = data.models || [];
      } catch(e) { this.models = []; }
    },

    async loadAgents() {
      try {
        this.settingAgents = await FreecoAPI.get('/api/agents');
      } catch(e) { this.settingAgents = []; }
    },

    async loadWorkflows() {
      try {
        this.settingWorkflows = await FreecoAPI.get('/api/workflows');
      } catch(e) { this.settingWorkflows = []; }
    },

    async addCustomModel() {
      var id = this.customModelId.trim();
      if (!id) return;
      this.customModelStatus = 'Adding...';
      try {
        await FreecoAPI.post('/api/models/custom', {
          id: id,
          provider: this.customModelProvider || 'openrouter',
          context_window: this.customModelContext || 128000,
          max_output_tokens: this.customModelMaxOutput || 8192,
        });
        this.customModelStatus = 'Added!';
        this.customModelId = '';
        this.showCustomModelForm = false;
        await this.loadModels();
      } catch(e) {
        this.customModelStatus = 'Error: ' + (e.message || 'Failed');
      }
    },

    async deleteCustomModel(modelId) {
      if (!confirm('Delete custom model "' + modelId + '"?')) return;
      try {
        await FreecoAPI.del('/api/models/custom/' + encodeURIComponent(modelId));
        FreecoToast.success('Model deleted');
        await this.loadModels();
      } catch(e) {
        FreecoToast.error('Failed to delete: ' + (e.message || 'Unknown error'));
      }
    },

    async loadConfigSchema() {
      try {
        var results = await Promise.all([
          FreecoAPI.get('/api/config/schema').catch(function() { return {}; }),
          FreecoAPI.get('/api/config')
        ]);
        this.configSchema = results[0].sections || null;
        this.configValues = results[1] || {};
      } catch(e) { /* silent */ }
    },

    isConfigDirty(section, field) {
      return this.configDirty[section + '.' + field] === true;
    },

    markConfigDirty(section, field) {
      this.configDirty[section + '.' + field] = true;
    },

    async saveConfigField(section, field, value) {
      var key = section + '.' + field;
      // Root-level fields (api_key, api_listen, log_level) use just the field name
      var sectionMeta = this.configSchema && this.configSchema[section];
      var path = (sectionMeta && sectionMeta.root_level) ? field : key;
      this.configSaving[key] = true;
      try {
        await FreecoAPI.post('/api/config/set', { path: path, value: value });
        this.configDirty[key] = false;
        FreecoToast.success('Saved ' + field);
      } catch(e) {
        FreecoToast.error('Failed to save: ' + e.message);
      }
      this.configSaving[key] = false;
    },

    get filteredTools() {
      var q = this.toolSearch.toLowerCase().trim();
      if (!q) return this.tools;
      return this.tools.filter(function(t) {
        return t.name.toLowerCase().indexOf(q) !== -1 ||
               (t.description || '').toLowerCase().indexOf(q) !== -1;
      });
    },

    get filteredModels() {
      var self = this;
      return this.models.filter(function(m) {
        if (self.modelProviderFilter && m.provider !== self.modelProviderFilter) return false;
        if (self.modelTierFilter && m.tier !== self.modelTierFilter) return false;
        if (self.modelSearch) {
          var q = self.modelSearch.toLowerCase();
          if (m.id.toLowerCase().indexOf(q) === -1 &&
              (m.display_name || '').toLowerCase().indexOf(q) === -1) return false;
        }
        return true;
      });
    },

    get uniqueProviderNames() {
      var seen = {};
      this.models.forEach(function(m) { seen[m.provider] = true; });
      return Object.keys(seen).sort();
    },

    /// Coarse category for a provider used to group the Providers tab.
    /// Returns: 'frontier' | 'oss' | 'local' | 'aggregator' | 'regional' | 'other'.
    providerCategory(p) {
      if (!p) return 'other';
      if (p.is_local || p.key_required === false) return 'local';
      var id = (p.id || '').toLowerCase();
      var FRONTIER = ['anthropic','openai','gemini','google','xai','bedrock','azure','vertex'];
      var OSS = ['groq','together','fireworks','cerebras','sambanova','deepseek','mistral','perplexity','cohere','ai21','huggingface','replicate','nvidia','venice','novita','chutes'];
      var AGG = ['openrouter','litellm','github-copilot','claude-code'];
      var REGIONAL = ['qwen','minimax','zhipu','zai','moonshot','qianfan','volcengine','kimi'];
      if (FRONTIER.indexOf(id) !== -1) return 'frontier';
      if (REGIONAL.indexOf(id) !== -1) return 'regional';
      if (AGG.indexOf(id) !== -1) return 'aggregator';
      if (OSS.indexOf(id) !== -1) return 'oss';
      return 'other';
    },

    providerCategoryLabel(cat) {
      switch (cat) {
        case 'frontier':   return 'Frontier (Anthropic, OpenAI, Google, xAI, Bedrock)';
        case 'oss':        return 'Open-Weight Hosts (Groq, Together, Fireworks, DeepSeek, etc.)';
        case 'aggregator': return 'Aggregators & Gateways (OpenRouter, GitHub Copilot)';
        case 'regional':   return 'Regional / China (Qwen, Zhipu, Moonshot, MiniMax)';
        case 'local':      return 'Local / Self-Hosted (Ollama, vLLM, LM Studio, Lemonade)';
        default:           return 'Other Providers';
      }
    },

    /// Stable category order for grouped rendering.
    get providerCategoriesOrdered() {
      return ['frontier', 'oss', 'aggregator', 'regional', 'local', 'other'];
    },

    /// Returns filter-matched providers grouped by category, preserving order.
    /// Each entry: { category, label, items: [...] }. Empty groups are omitted.
    get providersGrouped() {
      var self = this;
      var filtered = this.filteredProviders;
      var by = {};
      filtered.forEach(function(p) {
        var c = self.providerCategory(p);
        if (!by[c]) by[c] = [];
        by[c].push(p);
      });
      // Sort each group: configured first, then alphabetical
      Object.keys(by).forEach(function(c) {
        by[c].sort(function(a, b) {
          var ac = a.auth_status === 'configured' ? 0 : 1;
          var bc = b.auth_status === 'configured' ? 0 : 1;
          if (ac !== bc) return ac - bc;
          return (a.display_name || a.id).localeCompare(b.display_name || b.id);
        });
      });
      var out = [];
      this.providerCategoriesOrdered.forEach(function(c) {
        if (by[c] && by[c].length) {
          out.push({ category: c, label: self.providerCategoryLabel(c), items: by[c] });
        }
      });
      return out;
    },

    get filteredProviders() {
      var self = this;
      return this.providers.filter(function(p) {
        if (self.providerStatusFilter === 'configured' && p.auth_status !== 'configured') return false;
        if (self.providerStatusFilter === 'unconfigured' && p.auth_status === 'configured') return false;
        if (self.providerCategoryFilter && self.providerCategory(p) !== self.providerCategoryFilter) return false;
        if (self.providerSearch) {
          var q = self.providerSearch.toLowerCase();
          if ((p.display_name || '').toLowerCase().indexOf(q) === -1 &&
              (p.id || '').toLowerCase().indexOf(q) === -1 &&
              (p.api_key_env || '').toLowerCase().indexOf(q) === -1) return false;
        }
        return true;
      });
    },

    get configuredProviderCount() {
      return this.providers.filter(function(p) { return p.auth_status === 'configured'; }).length;
    },

    clearProviderFilters() {
      this.providerSearch = '';
      this.providerStatusFilter = '';
      this.providerCategoryFilter = '';
    },

    get uniqueTiers() {
      var seen = {};
      this.models.forEach(function(m) { if (m.tier) seen[m.tier] = true; });
      return Object.keys(seen).sort();
    },

    providerAuthClass(p) {
      if (p.auth_status === 'configured') return 'auth-configured';
      if (p.auth_status === 'not_set' || p.auth_status === 'missing') return 'auth-not-set';
      return 'auth-no-key';
    },

    providerAuthText(p) {
      if (p.auth_status === 'configured') return 'Configured';
      if (p.auth_status === 'not_set' || p.auth_status === 'missing') {
        if (p.id === 'claude-code') return 'Not Installed';
        return 'Not Set';
      }
      return 'No Key Needed';
    },

    providerCardClass(p) {
      if (p.auth_status === 'configured') return 'configured';
      if (p.auth_status === 'not_set' || p.auth_status === 'missing') return 'not-configured';
      return 'no-key';
    },

    tierBadgeClass(tier) {
      if (!tier) return '';
      var t = tier.toLowerCase();
      if (t === 'frontier') return 'tier-frontier';
      if (t === 'smart') return 'tier-smart';
      if (t === 'balanced') return 'tier-balanced';
      if (t === 'fast') return 'tier-fast';
      return '';
    },

    formatCost(cost) {
      if (!cost && cost !== 0) return '-';
      return '$' + cost.toFixed(4);
    },

    formatContext(ctx) {
      if (!ctx) return '-';
      if (ctx >= 1000000) return (ctx / 1000000).toFixed(1) + 'M';
      if (ctx >= 1000) return Math.round(ctx / 1000) + 'K';
      return String(ctx);
    },

    formatUptime(secs) {
      if (!secs) return '-';
      var h = Math.floor(secs / 3600);
      var m = Math.floor((secs % 3600) / 60);
      var s = secs % 60;
      if (h > 0) return h + 'h ' + m + 'm';
      if (m > 0) return m + 'm ' + s + 's';
      return s + 's';
    },

    async saveProviderKey(provider) {
      var key = this.providerKeyInputs[provider.id];
      if (!key || !key.trim()) { FreecoToast.error('Please enter an API key'); return; }
      // PRIVACY RED-FLAG: connecting a cloud provider means every message the
      // agents send is transmitted to that company. Non-technical users must
      // understand this before it happens. Local providers never leave the
      // device, so no warning is needed for them.
      if (!provider.is_local) {
        var name = provider.display_name || provider.id;
        var confirmed = window.confirm(
          '⚠️ Privacy warning\n\n' +
          'You are connecting ' + name + ', an online (cloud) AI provider.\n\n' +
          'Everything your agents send — your messages, files they read, and\n' +
          'your company’s data — will be transmitted to ' + name + '’s servers\n' +
          'and is subject to their privacy policy. This can expose sensitive data.\n\n' +
          'For full privacy, use a local model instead (Settings → Providers →\n' +
          '“Free local AI”) — it runs on this device and never sends your data out.\n\n' +
          'Connect ' + name + ' anyway?'
        );
        if (!confirmed) { FreecoToast.info('Cloud provider not connected — your data stays local.'); return; }
      }
      try {
        var resp = await FreecoAPI.post('/api/providers/' + encodeURIComponent(provider.id) + '/key', { key: key.trim() });
        if (resp && resp.switched_default) {
          FreecoToast.warning(resp.message || 'Default provider was switched to ' + provider.display_name);
        } else {
          FreecoToast.success('API key saved for ' + provider.display_name);
        }
        this.providerKeyInputs[provider.id] = '';
        await this.loadProviders();
        await this.loadModels();
      } catch(e) {
        FreecoToast.error('Failed to save key: ' + e.message);
      }
    },

    async removeProviderKey(provider) {
      try {
        await FreecoAPI.del('/api/providers/' + encodeURIComponent(provider.id) + '/key');
        FreecoToast.success('API key removed for ' + provider.display_name);
        await this.loadProviders();
        await this.loadModels();
      } catch(e) {
        FreecoToast.error('Failed to remove key: ' + e.message);
      }
    },

    async startCopilotOAuth() {
      this.copilotOAuth.polling = true;
      this.copilotOAuth.userCode = '';
      try {
        var resp = await FreecoAPI.post('/api/providers/github-copilot/oauth/start', {});
        this.copilotOAuth.userCode = resp.user_code;
        this.copilotOAuth.verificationUri = resp.verification_uri;
        this.copilotOAuth.pollId = resp.poll_id;
        this.copilotOAuth.interval = resp.interval || 5;
        window.open(resp.verification_uri, '_blank');
        this.pollCopilotOAuth();
      } catch(e) {
        FreecoToast.error('Failed to start Copilot login: ' + e.message);
        this.copilotOAuth.polling = false;
      }
    },

    pollCopilotOAuth() {
      var self = this;
      setTimeout(async function() {
        if (!self.copilotOAuth.pollId) return;
        try {
          var resp = await FreecoAPI.get('/api/providers/github-copilot/oauth/poll/' + self.copilotOAuth.pollId);
          if (resp.status === 'complete') {
            FreecoToast.success('GitHub Copilot authenticated successfully!');
            self.copilotOAuth = { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 };
            await self.loadProviders();
            await self.loadModels();
          } else if (resp.status === 'pending') {
            if (resp.interval) self.copilotOAuth.interval = resp.interval;
            self.pollCopilotOAuth();
          } else if (resp.status === 'expired') {
            FreecoToast.error('Device code expired. Please try again.');
            self.copilotOAuth = { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 };
          } else if (resp.status === 'denied') {
            FreecoToast.error('Access denied by user.');
            self.copilotOAuth = { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 };
          } else {
            FreecoToast.error('OAuth error: ' + (resp.error || resp.status));
            self.copilotOAuth = { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 };
          }
        } catch(e) {
          FreecoToast.error('Poll error: ' + e.message);
          self.copilotOAuth = { polling: false, userCode: '', verificationUri: '', pollId: '', interval: 5 };
        }
      }, self.copilotOAuth.interval * 1000);
    },

    async testProvider(provider) {
      this.providerTesting[provider.id] = true;
      this.providerTestResults[provider.id] = null;
      try {
        var result = await FreecoAPI.post('/api/providers/' + encodeURIComponent(provider.id) + '/test', {});
        this.providerTestResults[provider.id] = result;
        if (result.status === 'ok') {
          FreecoToast.success(provider.display_name + ' connected (' + (result.latency_ms || '?') + 'ms)');
        } else {
          FreecoToast.error(provider.display_name + ': ' + (result.error || 'Connection failed'));
        }
      } catch(e) {
        this.providerTestResults[provider.id] = { status: 'error', error: e.message };
        FreecoToast.error('Test failed: ' + e.message);
      }
      this.providerTesting[provider.id] = false;
    },

    async saveProviderUrl(provider) {
      var url = this.providerUrlInputs[provider.id];
      if (!url || !url.trim()) { FreecoToast.error('Please enter a base URL'); return; }
      url = url.trim();
      if (url.indexOf('http://') !== 0 && url.indexOf('https://') !== 0) {
        FreecoToast.error('URL must start with http:// or https://'); return;
      }
      this.providerUrlSaving[provider.id] = true;
      try {
        var result = await FreecoAPI.put('/api/providers/' + encodeURIComponent(provider.id) + '/url', { base_url: url });
        if (result.reachable) {
          FreecoToast.success(provider.display_name + ' URL saved &mdash; reachable (' + (result.latency_ms || '?') + 'ms)');
        } else {
          FreecoToast.warning(provider.display_name + ' URL saved but not reachable');
        }
        await this.loadProviders();
      } catch(e) {
        FreecoToast.error('Failed to save URL: ' + e.message);
      }
      this.providerUrlSaving[provider.id] = false;
    },

    async addCustomProvider() {
      var name = this.customProviderName.trim().toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-');
      if (!name) { FreecoToast.error('Please enter a provider name'); return; }
      var url = this.customProviderUrl.trim();
      if (!url) { FreecoToast.error('Please enter a base URL'); return; }
      if (url.indexOf('http://') !== 0 && url.indexOf('https://') !== 0) {
        FreecoToast.error('URL must start with http:// or https://'); return;
      }
      this.addingCustomProvider = true;
      this.customProviderStatus = '';
      try {
        var result = await FreecoAPI.put('/api/providers/' + encodeURIComponent(name) + '/url', { base_url: url });
        if (this.customProviderKey.trim()) {
          await FreecoAPI.post('/api/providers/' + encodeURIComponent(name) + '/key', { key: this.customProviderKey.trim() });
        }
        this.customProviderName = '';
        this.customProviderUrl = '';
        this.customProviderKey = '';
        this.customProviderStatus = '';
        FreecoToast.success('Provider "' + name + '" added' + (result.reachable ? ' (reachable)' : ' (not reachable yet)'));
        await this.loadProviders();
      } catch(e) {
        this.customProviderStatus = 'Error: ' + (e.message || 'Failed');
        FreecoToast.error('Failed to add provider: ' + e.message);
      }
      this.addingCustomProvider = false;
    },

    // -- People / accounts (RBAC) --
    async loadUsers() {
      this.usersLoadError = '';
      try {
        var data = await FreecoAPI.get('/api/users');
        this.users = data.users || [];
        if (data.admin_username) this.adminUsername = data.admin_username;
      } catch(e) {
        this.usersLoadError = (e.message && e.message.indexOf('Owner') >= 0)
          ? 'Only the owner can manage accounts.'
          : ('Could not load accounts: ' + (e.message || 'error'));
      }
    },

    async addUser() {
      if (this.usersBusy) return;
      var u = this.newUser;
      if (!u.name.trim()) { FreecoToast.error('Enter a name'); return; }
      if (!u.password || u.password.length < 8) { FreecoToast.error('Password must be at least 8 characters'); return; }
      this.usersBusy = true;
      this.usersMsg = '';
      try {
        var res = await FreecoAPI.post('/api/users', { name: u.name.trim(), role: u.role, password: u.password, enabled: true });
        this.usersMsg = res.message || 'Account saved. Restart FreEco.ai for it to take effect.';
        this.newUser = { name: '', role: 'kid', password: '' };
        await this.loadUsers();
      } catch(e) {
        FreecoToast.error(e.message || 'Could not add account');
      }
      this.usersBusy = false;
    },

    deleteUser(name) {
      var self = this;
      FreecoToast.confirm('Remove account', 'Remove the account "' + name + '"? This takes effect after a restart.', async function() {
        try {
          await FreecoAPI.del('/api/users/' + encodeURIComponent(name));
          self.usersMsg = 'Account removed. Restart FreEco.ai for it to take effect.';
          await self.loadUsers();
        } catch(e) {
          FreecoToast.error(e.message || 'Could not remove account');
        }
      });
    },

    // -- Security methods --
    async loadSecurity() {
      this.secLoading = true;
      try {
        this.securityData = await FreecoAPI.get('/api/security');
      } catch(e) {
        this.securityData = null;
      }
      this.secLoading = false;
    },

    async changePassword() {
      if (this.passwordSaving) return;
      if (this.passwordInput !== this.passwordConfirmation) {
        FreecoToast.error('New passwords do not match');
        return;
      }
      this.passwordSaving = true;
      try {
        var result = await FreecoAPI.post('/api/auth/set-password', {
          password: this.passwordInput,
          current_password: this.currentPassword
        });
        if (result.status === 'ok') {
          this.passwordInput = '';
          this.passwordConfirmation = '';
          this.currentPassword = '';
          FreecoToast.success('Password changed. Restart FreEco.ai to apply it.');
        }
      } catch(e) {
        FreecoToast.error(e.message || 'Could not change password');
      }
      this.passwordSaving = false;
    },

    isActive(key) {
      if (!this.securityData) return true;
      var core = this.securityData.core_protections || {};
      if (core[key] !== undefined) return core[key];
      return true;
    },

    getConfigValue(key) {
      if (!this.securityData) return null;
      var cfg = this.securityData.configurable || {};
      return cfg[key] || null;
    },

    getMonitoringValue(key) {
      if (!this.securityData) return null;
      var mon = this.securityData.monitoring || {};
      return mon[key] || null;
    },

    formatConfigValue(feature) {
      var val = this.getConfigValue(feature.valueKey);
      if (!val) return feature.configHint;
      switch (feature.valueKey) {
        case 'rate_limiter':
          return 'Algorithm: ' + (val.algorithm || 'GCRA') + ' | ' + (val.tokens_per_minute || 500) + ' tokens/min per IP';
        case 'websocket_limits':
          return 'Max ' + (val.max_per_ip || 5) + ' conn/IP | ' + Math.round((val.idle_timeout_secs || 1800) / 60) + 'min idle timeout | ' + Math.round((val.max_message_size || 65536) / 1024) + 'KB max msg';
        case 'wasm_sandbox':
          return 'Fuel: ' + (val.fuel_metering ? 'ON' : 'OFF') + ' | Epoch: ' + (val.epoch_interruption ? 'ON' : 'OFF') + ' | Timeout: ' + (val.default_timeout_secs || 30) + 's';
        case 'auth':
          return 'Mode: ' + (val.mode || 'unknown') + (val.api_key_set ? ' (key configured)' : ' (no key set)');
        default:
          return feature.configHint;
      }
    },

    formatMonitoringValue(feature) {
      var val = this.getMonitoringValue(feature.valueKey);
      if (!val) return feature.configHint;
      switch (feature.valueKey) {
        case 'audit_trail':
          return (val.enabled ? 'Active' : 'Disabled') + ' | ' + (val.algorithm || 'SHA-256') + ' | ' + (val.entry_count || 0) + ' entries logged';
        case 'taint_tracking':
          var labels = val.tracked_labels || [];
          return (val.enabled ? 'Active' : 'Disabled') + ' | Tracking: ' + labels.join(', ');
        case 'manifest_signing':
          return 'Algorithm: ' + (val.algorithm || 'Ed25519') + ' | ' + (val.available ? 'Available' : 'Not available');
        default:
          return feature.configHint;
      }
    },

    async verifyAuditChain() {
      this.verifyingChain = true;
      this.chainResult = null;
      try {
        var res = await FreecoAPI.get('/api/audit/verify');
        this.chainResult = res;
      } catch(e) {
        this.chainResult = { valid: false, error: e.message };
      }
      this.verifyingChain = false;
    },

    // -- Peers methods --
    async loadPeers() {
      this.peersLoading = true;
      this.peersLoadError = '';
      try {
        var data = await FreecoAPI.get('/api/peers');
        this.peers = (data.peers || []).map(function(p) {
          return {
            node_id: p.node_id,
            node_name: p.node_name,
            address: p.address,
            state: p.state,
            agent_count: (p.agents || []).length,
            protocol_version: p.protocol_version || 1
          };
        });
      } catch(e) {
        this.peers = [];
        this.peersLoadError = e.message || 'Could not load peers.';
      }
      this.peersLoading = false;
    },

    startPeerPolling() {
      var self = this;
      this.stopPeerPolling();
      this._peerPollTimer = setInterval(async function() {
        if (self.tab !== 'network') { self.stopPeerPolling(); return; }
        try {
          var data = await FreecoAPI.get('/api/peers');
          self.peers = (data.peers || []).map(function(p) {
            return {
              node_id: p.node_id,
              node_name: p.node_name,
              address: p.address,
              state: p.state,
              agent_count: (p.agents || []).length,
              protocol_version: p.protocol_version || 1
            };
          });
        } catch(e) { /* silent */ }
      }, 15000);
    },

    stopPeerPolling() {
      if (this._peerPollTimer) { clearInterval(this._peerPollTimer); this._peerPollTimer = null; }
    },

    // -- Migration methods --
    async autoDetect() {
      this.detecting = true;
      try {
        var data = await FreecoAPI.get('/api/migrate/detect');
        if (data.detected && data.scan) {
          this.sourcePath = data.path;
          this.scanResult = data.scan;
          this.migStep = 'preview';
        } else {
          this.migStep = 'not_found';
        }
      } catch(e) {
        this.migStep = 'not_found';
      }
      this.detecting = false;
    },

    async scanPath() {
      if (!this.sourcePath) return;
      this.scanning = true;
      try {
        var data = await FreecoAPI.post('/api/migrate/scan', { path: this.sourcePath });
        if (data.error) {
          FreecoToast.error('Scan error: ' + data.error);
          this.scanning = false;
          return;
        }
        this.scanResult = data;
        this.migStep = 'preview';
      } catch(e) {
        FreecoToast.error('Scan failed: ' + e.message);
      }
      this.scanning = false;
    },

    async runMigration(dryRun) {
      this.migrating = true;
      try {
        var target = this.targetPath;
        if (!target) target = '';
        var data = await FreecoAPI.post('/api/migrate', {
          source: 'openclaw',
          source_dir: this.sourcePath || (this.scanResult ? this.scanResult.path : ''),
          target_dir: target,
          dry_run: dryRun
        });
        this.migResult = data;
        this.migStep = 'result';
      } catch(e) {
        this.migResult = { status: 'failed', error: e.message };
        this.migStep = 'result';
      }
      this.migrating = false;
    },

    destroy() {
      this.stopPeerPolling();
    }
  };
}
