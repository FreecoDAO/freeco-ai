// FreEco.ai App — Alpine.js init, hash router, global store
'use strict';

// Marked.js configuration
if (typeof marked !== 'undefined') {
  marked.setOptions({
    breaks: true,
    gfm: true,
    highlight: function(code, lang) {
      if (typeof hljs !== 'undefined' && lang && hljs.getLanguage(lang)) {
        try { return hljs.highlight(code, { language: lang }).value; } catch(e) {}
      }
      return code;
    }
  });
}

function escapeHtml(text) {
  var div = document.createElement('div');
  div.textContent = text || '';
  return div.innerHTML.replace(/\n/g, '<br>');
}

function renderMarkdown(text) {
  if (!text) return '';
  if (typeof marked !== 'undefined') {
    // Protect LaTeX blocks from marked.js mangling (underscores, backslashes, etc.)
    var latexBlocks = [];
    var protected_ = text;
    // Protect display math $$...$$ first (greedy across lines)
    protected_ = protected_.replace(/\$\$([\s\S]+?)\$\$/g, function(match) {
      var idx = latexBlocks.length;
      latexBlocks.push(match);
      return '\x00LATEX' + idx + '\x00';
    });
    // Protect inline math $...$ (single line, not empty, not starting/ending with space)
    protected_ = protected_.replace(/\$([^\s$](?:[^$]*[^\s$])?)\$/g, function(match) {
      var idx = latexBlocks.length;
      latexBlocks.push(match);
      return '\x00LATEX' + idx + '\x00';
    });
    // Protect \[...\] display math
    protected_ = protected_.replace(/\\\[([\s\S]+?)\\\]/g, function(match) {
      var idx = latexBlocks.length;
      latexBlocks.push(match);
      return '\x00LATEX' + idx + '\x00';
    });
    // Protect \(...\) inline math
    protected_ = protected_.replace(/\\\(([\s\S]+?)\\\)/g, function(match) {
      var idx = latexBlocks.length;
      latexBlocks.push(match);
      return '\x00LATEX' + idx + '\x00';
    });

    var html = marked.parse(protected_);
    // Restore LaTeX blocks
    for (var i = 0; i < latexBlocks.length; i++) {
      html = html.replace('\x00LATEX' + i + '\x00', latexBlocks[i]);
    }
    // Add copy buttons to code blocks
    html = html.replace(/<pre><code/g, '<pre><button class="copy-btn" onclick="copyCode(this)">Copy</button><code');
    // Open external links in new tab
    html = html.replace(/<a\s+href="(https?:\/\/[^"]*)"(?![^>]*target=)([^>]*)>/gi, '<a href="$1" target="_blank" rel="noopener"$2>');
    return html;
  }
  return escapeHtml(text);
}

function copyCode(btn) {
  var code = btn.nextElementSibling;
  if (code) {
    navigator.clipboard.writeText(code.textContent).then(function() {
      btn.textContent = 'Copied!';
      btn.classList.add('copied');
      setTimeout(function() { btn.textContent = 'Copy'; btn.classList.remove('copied'); }, 1500);
    });
  }
}

// Tool category icon SVGs — returns inline SVG for each tool category
function toolIcon(toolName) {
  if (!toolName) return '';
  var n = toolName.toLowerCase();
  var s = 'width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"';
  // File/directory operations
  if (n.indexOf('file_') === 0 || n.indexOf('directory_') === 0)
    return '<svg ' + s + '><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M16 13H8"/><path d="M16 17H8"/></svg>';
  // Web/fetch
  if (n.indexOf('web_') === 0 || n.indexOf('link_') === 0)
    return '<svg ' + s + '><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15 15 0 0 1 4 10 15 15 0 0 1-4 10 15 15 0 0 1-4-10 15 15 0 0 1 4-10z"/></svg>';
  // Shell/exec
  if (n.indexOf('shell') === 0 || n.indexOf('exec_') === 0)
    return '<svg ' + s + '><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>';
  // Agent operations
  if (n.indexOf('agent_') === 0)
    return '<svg ' + s + '><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>';
  // Memory/knowledge
  if (n.indexOf('memory_') === 0 || n.indexOf('knowledge_') === 0)
    return '<svg ' + s + '><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg>';
  // Cron/schedule
  if (n.indexOf('cron_') === 0 || n.indexOf('schedule_') === 0)
    return '<svg ' + s + '><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>';
  // Browser/playwright
  if (n.indexOf('browser_') === 0 || n.indexOf('playwright_') === 0)
    return '<svg ' + s + '><rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/></svg>';
  // Container/docker
  if (n.indexOf('container_') === 0 || n.indexOf('docker_') === 0)
    return '<svg ' + s + '><path d="M22 12H2"/><path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/></svg>';
  // Image/media
  if (n.indexOf('image_') === 0 || n.indexOf('tts_') === 0)
    return '<svg ' + s + '><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>';
  // Hand tools
  if (n.indexOf('hand_') === 0)
    return '<svg ' + s + '><path d="M18 11V6a2 2 0 0 0-2-2 2 2 0 0 0-2 2"/><path d="M14 10V4a2 2 0 0 0-2-2 2 2 0 0 0-2 2v6"/><path d="M10 10.5V6a2 2 0 0 0-2-2 2 2 0 0 0-2 2v8"/><path d="M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.9-5.7-2.4L3.4 16a2 2 0 0 1 3.2-2.4L8 15"/></svg>';
  // Task/collab
  if (n.indexOf('task_') === 0)
    return '<svg ' + s + '><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg>';
  // Default — wrench
  return '<svg ' + s + '><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>';
}

// Alpine.js global store
document.addEventListener('alpine:init', function() {
  // Restore saved API key on load
  var savedKey = localStorage.getItem('openfang-api-key');
  if (savedKey) OpenFangAPI.setAuthToken(savedKey);

  Alpine.store('app', {
    agents: [],
    connected: false,
    booting: true,
    wsConnected: false,
    connectionState: 'connected',
    lastError: '',
    version: '0.1.0',
    agentCount: 0,
    pendingApprovalCount: 0,
    lastPendingApprovalSignature: '',
    pendingAgent: null,
    focusMode: localStorage.getItem('openfang-focus') === 'true',
    showOnboarding: false,
    showAuthPrompt: false,
    showPasswordSetup: false,
    authMode: 'apikey',
    sessionUser: null,
    sessionRole: null,
    capabilities: {},
    authAccounts: [],
    canSkipSignIn: false,
    updateAvailable: false,
    updateLatest: '',
    frozen: false,
    freezeBusy: false,

    async refreshFreeze() {
      try {
        var result = await OpenFangAPI.get('/api/system/freeze');
        this.frozen = !!result.frozen;
      } catch(e) { /* do not obscure the dashboard when the optional control is unavailable */ }
    },

    async toggleEmergencyFreeze() {
      if (this.freezeBusy) return;
      this.freezeBusy = true;
      try {
        if (this.frozen) {
          var released = await OpenFangAPI.del('/api/system/freeze');
          this.frozen = !!released.frozen;
          OpenFangToast.success('Emergency stop released. ' + (released.resumed || 0) + ' agent(s) resumed.');
        } else {
          var engaged = await OpenFangAPI.post('/api/system/freeze', {});
          this.frozen = !!engaged.frozen;
          OpenFangToast.warn('Emergency stop engaged. ' + (engaged.suspended || 0) + ' agent(s) paused.');
        }
      } catch(e) {
        OpenFangToast.error('Could not change emergency stop: ' + e.message);
      }
      this.freezeBusy = false;
    },

    // Quiet daily update check (Settings → System has the manual button
    // and the on/off toggle; both share the freeco_* localStorage keys).
    async quietUpdateCheck() {
      if (localStorage.getItem('freeco_auto_update_check') === 'off') return;
      var last = Number(localStorage.getItem('freeco_update_last_check') || 0);
      if (Date.now() - last < 24 * 60 * 60 * 1000) return;
      try {
        var res = await fetch('https://api.github.com/repos/FreecoDAO/freeco-ai/releases/latest', { headers: { Accept: 'application/vnd.github+json' } });
        if (!res.ok) return;
        var rel = await res.json();
        var latest = String(rel.tag_name || '').replace(/^v/, '').split('.').map(Number);
        var current = String(this.version || '0.0.0').replace(/^v/, '').split('.').map(Number);
        for (var i = 0; i < Math.max(latest.length, current.length); i++) {
          var a = latest[i] || 0, b = current[i] || 0;
          if (a !== b) { this.updateAvailable = a > b; break; }
        }
        this.updateLatest = String(rel.tag_name || '').replace(/^v/, '');
        localStorage.setItem('freeco_update_last_check', String(Date.now()));
      } catch (e) { /* silent — never bother the user over a failed check */ }
    },

    toggleFocusMode() {
      this.focusMode = !this.focusMode;
      localStorage.setItem('openfang-focus', this.focusMode);
    },

    async refreshAgents() {
      try {
        var agents = await OpenFangAPI.get('/api/agents');
        this.agents = Array.isArray(agents) ? agents : [];
        this.agentCount = this.agents.length;
      } catch(e) { /* silent */ }
    },

    async refreshApprovals() {
      try {
        var data = await OpenFangAPI.get('/api/approvals');
        var approvals = Array.isArray(data) ? data : (data.approvals || []);
        var pending = approvals.filter(function(a) { return a.status === 'pending'; });
        var signature = pending
          .map(function(a) { return a.id; })
          .sort()
          .join(',');
        if (pending.length > 0 && signature !== this.lastPendingApprovalSignature && typeof OpenFangToast !== 'undefined') {
          OpenFangToast.warn('An agent is waiting for approval. Open Approvals to review.');
        }
        this.pendingApprovalCount = pending.length;
        this.lastPendingApprovalSignature = signature;
      } catch(e) { /* silent */ }
    },

    async checkStatus() {
      try {
        var s = await OpenFangAPI.get('/api/status');
        this.connected = true;
        this.booting = false;
        this.lastError = '';
        this.version = s.version || '0.1.0';
        this.agentCount = s.agent_count || 0;
        this.quietUpdateCheck();
      } catch(e) {
        this.connected = false;
        this.lastError = e.message || 'Unknown error';
        console.warn('[FreEco.ai] Status check failed:', e.message);
      }
    },

    async checkOnboarding() {
      if (localStorage.getItem('openfang-onboarded')) return;
      try {
        var config = await OpenFangAPI.get('/api/config');
        var apiKey = config && config.api_key;
        var noKey = !apiKey || apiKey === 'not set' || apiKey === '';
        if (noKey && this.agentCount === 0) {
          this.showOnboarding = true;
        }
      } catch(e) {
        // If config endpoint fails, still show onboarding if no agents
        if (this.agentCount === 0) this.showOnboarding = true;
      }
    },

    dismissOnboarding() {
      this.showOnboarding = false;
      localStorage.setItem('openfang-onboarded', 'true');
    },

    async checkAuth() {
      try {
        // First check if session-based auth is configured
        var authInfo = await OpenFangAPI.get('/api/auth/check');
        this.authAccounts = Array.isArray(authInfo.accounts) ? authInfo.accounts : [];
        if (!authInfo.password_configured && !localStorage.getItem('openfang-password-setup-skipped')) {
          this.showPasswordSetup = true;
        }
        if (authInfo.mode === 'none') {
          // No session auth — fall back to API key detection
          this.authMode = 'apikey';
          this.sessionUser = null;
          this.sessionRole = null;
          this.capabilities = {};
          this.canSkipSignIn = true;
        } else if (authInfo.mode === 'session') {
          this.authMode = 'session';
          this.canSkipSignIn = false;
          if (authInfo.authenticated) {
            this.sessionUser = authInfo.username;
            this.sessionRole = authInfo.role || null;
            this.capabilities = authInfo.capabilities || {};
            this.showAuthPrompt = false;
            return;
          }
          // Session auth enabled but not authenticated — show login prompt
          this.showAuthPrompt = true;
          return;
        }
      } catch(e) { /* ignore — fall through to API key check */ }

      // API key mode detection
      try {
        await OpenFangAPI.get('/api/tools');
        this.showAuthPrompt = false;
      } catch(e) {
        if (e.message && (e.message.indexOf('Not authorized') >= 0 || e.message.indexOf('401') >= 0 || e.message.indexOf('Missing Authorization') >= 0 || e.message.indexOf('Unauthorized') >= 0)) {
          var saved = localStorage.getItem('openfang-api-key');
          if (saved) {
            OpenFangAPI.setAuthToken('');
            localStorage.removeItem('openfang-api-key');
          }
          this.showAuthPrompt = true;
        }
      }
    },

    submitApiKey(key) {
      if (!key || !key.trim()) return;
      OpenFangAPI.setAuthToken(key.trim());
      localStorage.setItem('openfang-api-key', key.trim());
      this.showAuthPrompt = false;
      this.refreshAgents();
    },

    async sessionLogin(username, password) {
      try {
        var result = await OpenFangAPI.post('/api/auth/login', { username: username, password: password });
        if (result.status === 'ok') {
          this.sessionUser = result.username;
          this.sessionRole = result.role || null;
          this.capabilities = result.capabilities || {};
          this.showAuthPrompt = false;
          this.refreshAgents();
        } else {
          OpenFangToast.error(result.error || 'Login failed');
        }
      } catch(e) {
        OpenFangToast.error(e.message || 'Login failed');
      }
    },

    async setInitialPassword(password, confirmation) {
      if (password !== confirmation) {
        OpenFangToast.error('Passwords do not match');
        return;
      }
      try {
        var result = await OpenFangAPI.post('/api/auth/set-password', { password: password, role: 'owner' });
        if (result.status === 'ok') {
          this.showPasswordSetup = false;
          localStorage.setItem('openfang-password-setup-skipped', 'true');
          OpenFangToast.success('Password saved. Restart FreEco.ai to enable sign-in.');
        }
      } catch(e) {
        OpenFangToast.error(e.message || 'Could not save password');
      }
    },

    skipPasswordSetup() {
      this.showPasswordSetup = false;
      localStorage.setItem('openfang-password-setup-skipped', 'true');
    },

    async bootstrapSkip() {
      try {
        await OpenFangAPI.post('/api/auth/bootstrap', { action: 'skip' });
      } catch(e) { /* ignore */ }
      this.showPasswordSetup = false;
      this.showAuthPrompt = false;
      this.canSkipSignIn = true;
      localStorage.setItem('openfang-password-setup-skipped', 'true');
    },

    async bootstrapUseCurrentUser() {
      var pwd = window.prompt('Set password for your current OS account (12+ chars):') || '';
      if (pwd.length < 12) {
        OpenFangToast.error('Password must be at least 12 characters');
        return;
      }
      try {
        var result = await OpenFangAPI.post('/api/auth/bootstrap', {
          action: 'use_current_user',
          password: pwd,
          role: 'owner'
        });
        if (result.status === 'ok') {
          this.showPasswordSetup = false;
          OpenFangToast.success('Account created. Restart FreEco.ai to load multi-user auth.');
        }
      } catch(e) {
        OpenFangToast.error(e.message || 'Bootstrap failed');
      }
    },

    async bootstrapCreateAccount(username, role, password, confirmation) {
      if (password !== confirmation) {
        OpenFangToast.error('Passwords do not match');
        return;
      }
      try {
        var result = await OpenFangAPI.post('/api/auth/bootstrap', {
          action: 'create_account',
          username: username,
          role: role,
          password: password
        });
        if (result.status === 'ok') {
          this.showPasswordSetup = false;
          OpenFangToast.success('Account created. Restart FreEco.ai to load multi-user auth.');
        }
      } catch(e) {
        OpenFangToast.error(e.message || 'Bootstrap failed');
      }
    },

    async ensureStepUp(password, method) {
      try {
        await OpenFangAPI.post('/api/auth/step-up', {
          method: method || 'password',
          password: password || '',
          biometric_asserted: method === 'biometric'
        });
        return true;
      } catch(e) {
        OpenFangToast.error(e.message || 'Verification failed');
        return false;
      }
    },

    async sessionLogout() {
      try {
        await OpenFangAPI.post('/api/auth/logout');
      } catch(e) { /* ignore */ }
      this.sessionUser = null;
      this.sessionRole = null;
      this.capabilities = {};
      this.showAuthPrompt = true;
    },

    clearApiKey() {
      OpenFangAPI.setAuthToken('');
      localStorage.removeItem('openfang-api-key');
    }
  });
});

// Main app component
function app() {
  return {
    page: 'agents',
    themeMode: localStorage.getItem('openfang-theme-mode') || 'system',
    theme: (() => {
      var mode = localStorage.getItem('openfang-theme-mode') || 'system';
      if (mode === 'system') return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      return mode;
    })(),
    sidebarCollapsed: localStorage.getItem('openfang-sidebar') === 'collapsed',
    mobileMenuOpen: false,
    connected: false,
    wsConnected: false,
    version: '0.1.0',
    agentCount: 0,

    get agents() { return Alpine.store('app').agents; },

    init() {
      var self = this;

      // Listen for OS theme changes (only matters when mode is 'system')
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
        if (self.themeMode === 'system') {
          self.theme = e.matches ? 'dark' : 'light';
        }
      });

      // Hash routing
      var validPages = ['overview','agents','sessions','approvals','comms','workflows','scheduler','channels','skills','hands','analytics','logs','runtime','settings','wizard'];
      var pageRedirects = {
        'chat': 'agents',
        'templates': 'agents',
        'triggers': 'workflows',
        'cron': 'scheduler',
        'schedules': 'scheduler',
        'memory': 'sessions',
        'audit': 'logs',
        'security': 'settings',
        'peers': 'settings',
        'migration': 'settings',
        'usage': 'analytics',
        'approval': 'approvals'
      };
      function handleHash() {
        var hash = window.location.hash.replace('#', '') || 'agents';
        if (pageRedirects[hash]) {
          hash = pageRedirects[hash];
          window.location.hash = hash;
        }
        if (validPages.indexOf(hash) >= 0) self.page = hash;
      }
      window.addEventListener('hashchange', handleHash);
      handleHash();

      // Keyboard shortcuts
      document.addEventListener('keydown', function(e) {
        // Ctrl+K — focus agent switch / go to agents
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
          e.preventDefault();
          self.navigate('agents');
        }
        // Ctrl+N — new agent
        if ((e.ctrlKey || e.metaKey) && e.key === 'n' && !e.shiftKey) {
          e.preventDefault();
          self.navigate('agents');
        }
        // Ctrl+Shift+F — toggle focus mode
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'F') {
          e.preventDefault();
          Alpine.store('app').toggleFocusMode();
        }
        // Escape — close mobile menu
        if (e.key === 'Escape') {
          self.mobileMenuOpen = false;
        }
      });

      // Connection state listener
      OpenFangAPI.onConnectionChange(function(state) {
        Alpine.store('app').connectionState = state;
      });

      // Initial data load
      this.pollStatus();
      Alpine.store('app').refreshApprovals();
      Alpine.store('app').refreshFreeze();
      Alpine.store('app').checkOnboarding();
      Alpine.store('app').checkAuth();
      setInterval(function() {
        self.pollStatus();
        Alpine.store('app').refreshApprovals();
        Alpine.store('app').refreshFreeze();
      }, 5000);
    },

    navigate(p) {
      this.page = p;
      window.location.hash = p;
      this.mobileMenuOpen = false;
    },

    setTheme(mode) {
      this.themeMode = mode;
      localStorage.setItem('openfang-theme-mode', mode);
      if (mode === 'system') {
        this.theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
      } else {
        this.theme = mode;
      }
    },

    toggleTheme() {
      var modes = ['light', 'system', 'dark'];
      var next = modes[(modes.indexOf(this.themeMode) + 1) % modes.length];
      this.setTheme(next);
    },

    toggleSidebar() {
      this.sidebarCollapsed = !this.sidebarCollapsed;
      localStorage.setItem('openfang-sidebar', this.sidebarCollapsed ? 'collapsed' : 'expanded');
    },

    async pollStatus() {
      var store = Alpine.store('app');
      await store.checkStatus();
      await store.refreshAgents();
      this.connected = store.connected;
      this.version = store.version;
      this.agentCount = store.agentCount;
      this.wsConnected = OpenFangAPI.isWsConnected();
    }
  };
}
