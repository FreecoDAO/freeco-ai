// FreEco.ai — Freeco Assistant: a global concierge widget present on every page.
// It listens (chat + voice), guides setup, and routes the user to the right
// builder (agents, teams, workflows, tools/MCP, channels, local AI). It talks to
// a concierge agent over the normal message API; if none exists yet it offers to
// create one. This is the v1 shell for the self-running company/nonprofit
// concierge — deeper autonomous orchestration builds on top of it.
'use strict';

function freecoAssistant() {
  var mId = 0;
  return {
    open: false,
    booted: false,
    agent: null,          // resolved concierge agent
    noAgents: false,      // true when the workspace has no agents yet
    input: '',
    sending: false,
    messages: [],         // { id, role: 'user'|'freeco'|'system', html, ts }
    // voice
    recording: false,
    recordingTime: 0,
    _rec: null,
    _chunks: [],
    _timer: null,
    // spoken replies (browser text-to-speech, offline, in a warm male voice)
    voiceOut: (function() { try { return localStorage.getItem('freeco-voice-out') !== 'off'; } catch (e) { return true; } })(),
    speaking: false,
    paused: false,
    _voice: null,
    // window state — resizable, movable, full-screen
    fullscreen: false,
    moved: false,          // true once the user drags it (switches to free positioning)
    pos: { x: 0, y: 0 },   // top-left when moved
    size: { w: 380, h: 560 },
    _drag: null,
    _resize: null,
    // attachments queued for the next message
    showAttach: false,
    attachments: [],       // { name, kind } uploaded and ready to send
    attaching: false,
    // live progress — what Freeco is doing right now, so a misheard request can
    // be caught and stopped instead of running to completion in silence
    steps: [],             // { text, kind: 'phase'|'tool'|'note' }
    showSteps: true,
    _wsAgentId: null,
    awaitingConfirm: false,  // voice transcript is in the box, waiting for review
    queued: [],              // messages typed while a reply was still streaming
    approvals: [],           // permission requests, answered inline in this chat

    // Quick-setup topics — each routes to the relevant builder and seeds a
    // guiding prompt so Freeco can walk the user through it.
    topics: [
      { id: 'company',  label: 'Set up a company / nonprofit', page: 'workflows', icon: 'M3 21h18M5 21V7l7-4 7 4v14M9 9h1M9 13h1M9 17h1M14 9h1M14 13h1M14 17h1',
        seed: 'Help me set up a self-running company. Walk me through idea, structure, and which agent teams I need (planning, email, site, sales, production, development, accounting).' },
      { id: 'team',     label: 'Add an agent or team', page: 'agents', icon: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0 .01M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
        seed: 'What agents or teams should I create first, and how do I set each one up?' },
      { id: 'workflow', label: 'Create a workflow', page: 'workflows', icon: 'M6 3v12M6 21a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM6 9a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM18 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM18 9c0 4-6 4-6 8',
        seed: 'Help me design a workflow that connects my agents and runs a repeatable task end to end.' },
      { id: 'tools',    label: 'Connect a tool / MCP', page: 'skills', icon: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
        seed: 'What tools and MCP servers do I need for this, and how do I install and connect them?' },
      { id: 'services', label: 'Connect a CRM, voice, or accounting app', page: 'settings', settingsTab: 'services', icon: 'M20 7h-9M14 17H5M17 3a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM7 13a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
        seed: 'Help me set up the services my company needs — a CRM for contacts/donors, real-time voice calling, or accounting. Which should I install first (local via Docker, recommended, or web), and walk me through it. If a CRM is best, propose Twenty for local and offer to install and connect it.' },
      { id: 'channel',  label: 'Connect email / site / channel', page: 'channels', icon: 'M4 4h16v16H4zM22 6l-10 7L2 6',
        seed: 'Help me connect a channel — email, a website, or a domain — so my agents can act in the real world.' },
      { id: 'localai',  label: 'Set up free local AI', page: 'settings', icon: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20',
        seed: 'Set up free local AI (Gemma 4, running on this device) so I can work privately with no cloud cost.' }
    ],

    init: function() {
      var self = this;
      // Re-resolve the concierge agent whenever the shared agent list changes.
      this.$watch('$store.app.agents', function() { self._resolveAgent(); });
      this._resolveAgent();
      // Surface permission requests in this chat, where the user already is.
      this.loadApprovals();
      setInterval(function() { self.loadApprovals(); }, 5000);
      // Browsers load TTS voices asynchronously — grab them now and again when
      // the list becomes available, so the first spoken reply already has a
      // good male voice picked.
      if (window.speechSynthesis) {
        self._voice = self._pickVoice();
        window.speechSynthesis.onvoiceschanged = function() { self._voice = self._pickVoice(); };
      }
      this.booted = true;
    },

    _resolveAgent: function() {
      var agents = (window.Alpine && Alpine.store('app') && Alpine.store('app').agents) || [];
      if (!agents.length) { this.agent = null; this.noAgents = true; return; }
      this.noAgents = false;
      // Prefer an agent that is clearly the concierge; else fall back to the first.
      var pick = agents.find(function(a) {
        var n = (a.name || '').toLowerCase();
        return n.indexOf('freeco') !== -1 || n.indexOf('concierge') !== -1 || n.indexOf('assistant') !== -1;
      });
      this.agent = pick || agents[0];
    },

    // Freeco should work the moment you open it. If the workspace has no
    // agents, create the bundled concierge automatically instead of telling the
    // user to go build one — nobody should have to assemble their assistant
    // before they can ask it anything.
    _ensureConcierge: async function() {
      if (this.agent || this._creatingAgent) return;
      this._creatingAgent = true;
      try {
        var res = await OpenFangAPI.post('/api/agents', { template: 'freeco-concierge' });
        if (res && (res.agent_id || res.name)) {
          try { await Alpine.store('app').refreshAgents(); } catch (e) { /* optional */ }
          this._resolveAgent();
          if (!this.agent && res.name) {
            this.agent = { id: res.agent_id, name: res.name };
            this.noAgents = false;
          }
        }
      } catch (e) {
        // Leave noAgents true; send() will show the manual path.
      }
      this._creatingAgent = false;
    },

    toggle: function() {
      this.open = !this.open;
      if (this.open) {
        this._resolveAgent();
        if (!this.agent) this._ensureConcierge();
        if (!this.messages.length) this._greet();
        var self = this;
        this.$nextTick(function() {
          var el = document.getElementById('freeco-input');
          if (el) el.focus();
          self._scroll();
        });
      }
    },

    _greet: function() {
      var name = (window.Alpine && Alpine.store('app') && Alpine.store('app').sessionUser) || '';
      var hi = name ? ('Hi ' + name + ' — ') : 'Hi — ';
      this.messages.push({
        id: ++mId, role: 'freeco', ts: Date.now(),
        html: '<p>' + hi + "I'm <strong>Freeco</strong>, your Ethical Executive AI Assistant &amp; Concierge. Tell me what you want to build — a company, a nonprofit, a workflow, or just shop and chat — and I'll help you set up the agents, tools and channels to run it. Pick a shortcut below or just type/talk.</p>"
      });
    },

    // Quick-setup: jump to the relevant builder and hand Freeco a guiding prompt.
    quick: function(topic) {
      // Some destinations are a Settings sub-tab (e.g. Services). Stash the tab
      // so the Settings page opens straight to it.
      if (topic.settingsTab) { try { window.__freecoSettingsTab = topic.settingsTab; } catch (e) { /* ignore */ } }
      window.dispatchEvent(new CustomEvent('freeco-navigate', { detail: topic.page }));
      this.input = topic.seed;
      if (this.agent) { this.send(); }
      else {
        this.messages.push({ id: ++mId, role: 'freeco', ts: Date.now(),
          html: '<p>I’ve opened the <strong>' + topic.page + '</strong> page for you. Create your first agent there and I’ll guide the rest.</p>' });
        this._scroll();
      }
    },

    // ---- Conversation ergonomics -------------------------------------
    // Everything below exists because a chat you cannot copy from, correct,
    // retry or return to is a chat that loses your work. Preserving history
    // in the database was only half the job; this is the half that makes it
    // reachable.

    showHistory: false,
    sessions: [],
    sessionSearch: '',
    copiedId: null,

    get filteredSessions() {
      var q = (this.sessionSearch || '').toLowerCase().trim();
      if (!q) return this.sessions;
      return this.sessions.filter(function (s) {
        return ((s.label || '') + ' ' + (s.id || '')).toLowerCase().indexOf(q) !== -1;
      });
    },

    async loadSessions() {
      try {
        var data = await OpenFangAPI.get('/api/sessions');
        var list = Array.isArray(data) ? data : (data.sessions || []);
        // Newest first: the conversation you want is nearly always recent.
        this.sessions = list.sort(function (a, b) {
          return String(b.updated_at || '').localeCompare(String(a.updated_at || ''));
        });
      } catch (e) { this.sessions = []; }
      return this.sessions;
    },

    toggleHistory: function () {
      this.showHistory = !this.showHistory;
      if (this.showHistory) this.loadSessions();
    },

    // Start fresh without destroying anything. The previous conversation stays
    // in history rather than being overwritten.
    newChat: function () {
      this.messages = [];
      this.steps = [];
      this.queued = [];
      this.input = '';
      this.attachments = [];
      this.sessionId = null;
      this.showHistory = false;
      this._scroll();
    },

    async openSession(id) {
      this.showHistory = false;
      try {
        var data = await OpenFangAPI.get('/api/sessions/' + encodeURIComponent(id));
        var msgs = (data && (data.messages || data.session && data.session.messages)) || [];
        this.messages = msgs.map(function (m) {
          var text = typeof m.content === 'string' ? m.content
                   : (m.content && m.content.text) || '';
          return {
            id: ++mId,
            role: m.role === 'user' ? 'user' : 'freeco',
            ts: Date.parse(m.timestamp || '') || Date.now(),
            html: this._md ? this._md(text) : this._escape(text),
            raw: text
          };
        }.bind(this));
        this.sessionId = id;
        this._scroll();
      } catch (e) {
        OpenFangToast.error('Could not open that conversation: ' + e.message);
      }
    },

    // Rename a conversation. Auto-generated names are a starting point, and
    // the user's own name for something is always better than a guess.
    async renameSession(id, current) {
      var name = window.prompt('Name this conversation', current || '');
      if (name === null) return;
      try {
        await OpenFangAPI.put('/api/sessions/' + encodeURIComponent(id) + '/label',
                              { label: name.trim() || null });
        await this.loadSessions();
      } catch (e) { OpenFangToast.error('Rename failed: ' + e.message); }
    },

    _plain: function (m) {
      if (m.raw) return m.raw;
      var d = document.createElement('div');
      d.innerHTML = m.html || '';
      return d.textContent || '';
    },

    copyMessage: function (m) {
      var text = this._plain(m);
      var done = function () {
        this.copiedId = m.id;
        setTimeout(function () { this.copiedId = null; }.bind(this), 1500);
      }.bind(this);
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {});
      } else {
        // Older browsers and non-secure origins have no clipboard API.
        var ta = document.createElement('textarea');
        ta.value = text; document.body.appendChild(ta); ta.select();
        try { document.execCommand('copy'); done(); } catch (e) {}
        document.body.removeChild(ta);
      }
    },

    copyConversation: function () {
      var out = this.messages.filter(function (m) { return !m.thinking; })
        .map(function (m) {
          return (m.role === 'user' ? 'You: ' : 'Freeco: ') + this._plain(m);
        }.bind(this)).join('\n\n');
      this.copyMessage({ id: -1, raw: out });
      OpenFangToast.success('Conversation copied.');
    },

    exportConversation: function () {
      var out = this.messages.filter(function (m) { return !m.thinking; })
        .map(function (m) {
          return (m.role === 'user' ? '**You:** ' : '**Freeco:** ') + this._plain(m);
        }.bind(this)).join('\n\n');
      var blob = new Blob([out], { type: 'text/markdown' });
      var a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = 'freeco-conversation.md';
      a.click();
      setTimeout(function () { URL.revokeObjectURL(a.href); }, 1000);
    },

    // Retry drops the failed answer and re-sends the question, rather than
    // making the user retype it.
    retryFrom: function (m) {
      if (this.sending) return;
      var i = this.messages.indexOf(m);
      var q = null;
      for (var j = i; j >= 0; j--) {
        if (this.messages[j].role === 'user') { q = this.messages[j]; break; }
      }
      if (!q) return;
      this.messages = this.messages.slice(0, this.messages.indexOf(q));
      this.input = this._plain(q);
      this.attachments = (q.atts || []).slice();
      this.send();
    },

    // Edit rewinds to the question so a misunderstood request can be fixed at
    // the source instead of patched in a follow-up.
    editMessage: function (m) {
      if (this.sending) return;
      this.input = this._plain(m);
      this.attachments = (m.atts || []).slice();
      this.messages = this.messages.slice(0, this.messages.indexOf(m));
      this._scroll();
    },

    send: async function() {
      var text = (this.input || '').trim();
      var atts = this.attachments.slice();
      if (!text && !atts.length) return;
      // A reply in flight must never block the user. Queue the message and send
      // it as soon as the current run finishes — being unable to correct or add
      // to a request while the agent works is how a misunderstood task runs to
      // completion unchallenged.
      if (this.sending) {
        this.queued.push({ text: text, atts: atts });
        this.messages.push({
          id: ++mId, role: 'system', ts: Date.now(),
          html: 'Queued: ' + this._escape(text) + ' <span class="text-dim">(sends when the current task finishes — or press Stop)</span>'
        });
        this.input = '';
        this.attachments = [];
        this._scroll();
        return;
      }
      if (!this.agent) { this._resolveAgent(); }
      if (!this.agent) {
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
          html: 'You don’t have any agents yet. <a href="#" onclick="window.dispatchEvent(new CustomEvent(\'freeco-navigate\',{detail:\'agents\'}));return false;">Create your Freeco concierge agent</a> to get started.' });
        this.input = ''; this._scroll(); return;
      }
      var bubble = this._escape(text);
      if (atts.length) {
        bubble += '<div class="freeco-att-row">' + atts.map(function(a) {
          return '<span class="freeco-att">📎 ' + a.name.replace(/</g, '&lt;') + '</span>';
        }).join('') + '</div>';
      }
      // Keep the raw text: retry and edit need the original, not the escaped
      // HTML that was rendered from it.
      this.messages.push({ id: ++mId, role: 'user', ts: Date.now(), html: bubble, raw: text, atts: atts.slice() });
      this.input = '';
      this.attachments = [];
      this.sending = true;
      this.awaitingConfirm = false;
      this.steps = [];
      var thinking = { id: ++mId, role: 'freeco', ts: Date.now(), html: '<span class="freeco-typing">• • •</span>', thinking: true };
      this.messages.push(thinking);
      this._scroll();

      // Prefer the WebSocket: it streams text as it is generated and reports
      // every tool the agent runs, so the user can watch the work and stop it.
      // The blocking POST below stays as a fallback for when WS is unavailable.
      if (this._openStream(text, atts)) return;

      try {
        var payload = { message: text || '(see attached)' };
        if (atts.length) payload.attachments = atts.map(function(a) { return a.name; });
        var res = await OpenFangAPI.post('/api/agents/' + this.agent.id + '/message', payload);
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        var reply = res.response || '(no reply)';
        this.messages.push({ id: ++mId, role: 'freeco', ts: Date.now(), html: this._md(reply) });
        if (this.voiceOut) this.speak(reply);
      } catch (e) {
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        var msg = (e.message || 'request failed').toLowerCase();
        var friendly;
        // The most common cause on a fresh install: the default model points at
        // a local Ollama model that is not installed yet, or no cloud key is set.
        if (msg.indexOf('server error') !== -1 || msg.indexOf('connection') !== -1 ||
            msg.indexOf('model') !== -1 || msg.indexOf('11434') !== -1 || msg.indexOf('ollama') !== -1) {
          friendly = 'I couldn’t reach a language model. Two common causes: if you just changed the model, <strong>restart FreEco.ai</strong> — it reads the config at startup. Otherwise open <a href="#" onclick="window.dispatchEvent(new CustomEvent(\'freeco-navigate\',{detail:\'settings\'}));return false;">Settings → Providers</a> and either <strong>Set up free local AI</strong> (downloads Gemma 4 and runs it on this device) or add a cloud API key.';
        } else {
          friendly = 'Something went wrong: ' + this._escape(e.message || 'request failed');
        }
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: friendly });
      }
      this.sending = false;
      this._scroll();
      var self = this;
      this.$nextTick(function() { var el = document.getElementById('freeco-input'); if (el) el.focus(); });
    },

    // ---- Live streaming: watch the work, and stop it ----
    // Opens (or reuses) the agent WebSocket and sends the message over it.
    // Returns true when streaming took over, false to use the blocking POST.
    _openStream: function(text, atts) {
      if (!this.agent || !OpenFangAPI.wsConnect) return false;
      var self = this;
      var payload = { type: 'message', content: text || '(see attached)' };
      if (atts && atts.length) payload.attachments = atts.map(function(a) { return a.name; });

      var send = function() {
        try { return OpenFangAPI.wsSend(payload); } catch (e) { return false; }
      };

      // Already connected to this agent — just send.
      if (this._wsAgentId === this.agent.id && OpenFangAPI.isWsConnected && OpenFangAPI.isWsConnected()) {
        return send();
      }

      try {
        OpenFangAPI.wsConnect(this.agent.id, {
          onMessage: function(ev) { self._onStreamEvent(ev); },
          onOpen: function() { send(); },
          onClose: function() { self._wsAgentId = null; }
        });
        this._wsAgentId = this.agent.id;
        return true;
      } catch (e) {
        this._wsAgentId = null;
        return false;
      }
    },

    _step: function(text, kind) {
      this.steps.push({ text: text, kind: kind || 'note' });
      if (this.steps.length > 40) this.steps.shift();
      this.$nextTick(function() {
        var el = document.getElementById('freeco-steps');
        if (el) el.scrollTop = el.scrollHeight;
      });
    },

    _onStreamEvent: function(ev) {
      var self = this;
      var t = ev && ev.type;
      if (t === 'phase') {
        if (ev.phase && ev.phase !== 'done') this._step(ev.phase + (ev.detail ? ': ' + ev.detail : ''), 'phase');
        return;
      }
      if (t === 'tool_start') { this._step('running ' + (ev.tool || 'tool'), 'tool'); return; }
      if (t === 'tool_end' || t === 'tool_result') {
        if (ev.tool) this._step('finished ' + ev.tool + (ev.is_error ? ' (failed)' : ''), 'tool');
        return;
      }
      if (t === 'text_delta') {
        // Stream into the pending bubble so words appear as they are produced.
        var last = this.messages[this.messages.length - 1];
        if (!last || !last.streaming) {
          this.messages = this.messages.filter(function(m) { return !m.thinking; });
          last = { id: ++mId, role: 'freeco', ts: Date.now(), html: '', streaming: true, raw: '' };
          this.messages.push(last);
        }
        last.raw = (last.raw || '') + (ev.text || ev.delta || '');
        last.html = this._md(last.raw);
        this._scroll();
        return;
      }
      if (t === 'response' || t === 'message' || t === 'silent_complete') {
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        var full = ev.content || ev.response || '';
        var last2 = this.messages[this.messages.length - 1];
        if (last2 && last2.streaming) {
          last2.streaming = false;
          if (full) { last2.raw = full; last2.html = this._md(full); }
          full = last2.raw || full;
        } else if (full) {
          this.messages.push({ id: ++mId, role: 'freeco', ts: Date.now(), html: this._md(full) });
        }
        this.sending = false;
        this.steps = [];
        if (this.voiceOut && full) this.speak(full);
        this._scroll();
        this._drainQueue();
        return;
      }
      if (t === 'error') {
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
          html: 'Something went wrong: ' + self._escape(ev.message || ev.error || 'unknown') });
        this.sending = false;
        this.steps = [];
        this._scroll();
      }
    },

    // Send the next message the user typed while the agent was busy.
    _drainQueue: function() {
      if (!this.queued.length || this.sending) return;
      var next = this.queued.shift();
      var self = this;
      this.input = next.text;
      this.attachments = next.atts || [];
      this.$nextTick(function() { self.send(); });
    },

    // ---- Permission requests, answered right here ----
    // Approvals used to live on a separate page, so a task would sit blocked
    // while the user waited in the chat with no idea anything was needed.
    async loadApprovals() {
      try {
        var data = await OpenFangAPI.get('/api/approvals');
        var list = Array.isArray(data) ? data : (data.approvals || data.pending || []);
        this.approvals = list.filter(function(a) {
          var s = (a.status || a.state || 'pending').toLowerCase();
          return s === 'pending' || s === 'waiting';
        });
      } catch (e) { /* leave whatever we had */ }
    },

    // decision: 'once' | 'always' | 'deny' | 'pause'
    async answerApproval(id, decision) {
      // "Pause" is a local hold: leave the request pending and freeze agents so
      // the user can think without the task racing ahead or being denied.
      if (decision === 'pause') {
        try { await Alpine.store('app').toggleEmergencyFreeze(); } catch (e) { /* optional */ }
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
          html: 'Paused. The request is still waiting — answer it when you are ready.' });
        this._scroll();
        return;
      }
      var path = decision === 'deny' ? '/reject' : '/approve';
      try {
        // 'always' additionally remembers the choice, when the server supports it.
        await OpenFangAPI.post('/api/approvals/' + encodeURIComponent(id) + path,
          decision === 'always' ? { remember: true, scope: 'always' } : {});
        this.approvals = this.approvals.filter(function(a) { return (a.id || a.request_id) !== id; });
        var label = decision === 'always' ? 'Allow always' : (decision === 'deny' ? 'Deny' : 'Allow once');
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
          html: 'You chose <strong>' + label + '</strong>.' });
        this._scroll();
        try { Alpine.store('app').refreshApprovals(); } catch (e) { /* optional */ }
      } catch (e) {
        OpenFangToast.error('Could not record that: ' + (e.message || 'error'));
      }
    },

    // Stop a run in progress — the whole point of showing the steps.
    stopRun: async function() {
      if (!this.agent) return;
      try {
        await OpenFangAPI.post('/api/agents/' + this.agent.id + '/stop', {});
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: 'Stopped.' });
      } catch (e) {
        OpenFangToast.error('Could not stop: ' + (e.message || 'error'));
      }
      this.sending = false;
      this.steps = [];
      this._scroll();
    },

    // ---- Voice (hold-to-talk) ----
    startVoice: async function() {
      if (this.recording) return;

      // Preferred path: the browser's own speech recognition. It needs no API
      // key, no server and no Whisper install, so voice input works out of the
      // box. Only if it is unavailable do we fall back to record-and-upload,
      // which requires a speech-to-text service to be configured.
      var SR = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (SR) {
        var self = this;
        try {
          var recog = new SR();
          recog.lang = navigator.language || 'en-US';
          // continuous: keep listening across natural pauses. With it off the
          // recogniser stops at the first silence, which chopped requests
          // mid-sentence. interimResults lets the text appear while speaking so
          // a mishearing is visible immediately rather than after the fact.
          recog.interimResults = true;
          recog.maxAlternatives = 1;
          recog.continuous = true;
          recog.onresult = function(ev) {
            // With continuous + interim results, only results marked final are
            // settled text; the rest is the in-progress guess. Keep the settled
            // part and append the live guess so the box tracks speech in real
            // time without losing anything already recognised.
            var settled = '', live = '';
            for (var i = 0; i < ev.results.length; i++) {
              var r = ev.results[i];
              if (r.isFinal) settled += r[0].transcript;
              else live += r[0].transcript;
            }
            var said = (settled + live).trim();
            // Shown for review, never auto-sent. Speech recognition mishears
            // words ("YAK"), and auto-sending meant a whole task ran on a
            // misheard instruction before the user could see it.
            if (said) {
              self.input = said;
              self.awaitingConfirm = true;
            }
          };
          recog.onerror = function(ev) {
            self.recording = false;
            var why = (ev && ev.error) || 'unknown';
            if (why === 'not-allowed' || why === 'service-not-allowed') {
              if (typeof OpenFangToast !== 'undefined') OpenFangToast.error('Microphone access denied');
            } else if (why !== 'aborted' && why !== 'no-speech') {
              self.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
                html: 'Voice input failed (' + self._escape(why) + '). You can type instead.' });
              self._scroll();
            }
          };
          recog.onend = function() { self.recording = false; if (self._timer) { clearInterval(self._timer); self._timer = null; } };
          this._recog = recog;
          recog.start();
          this.recording = true;
          this.recordingTime = 0;
          this._timer = setInterval(function() { self.recordingTime++; }, 1000);
          return;
        } catch (e) {
          this._recog = null; // fall through to the upload path
        }
      }

      // No SpeechRecognition here (Firefox does not implement it). Recording and
      // uploading still works, but only if a speech-to-text service is
      // configured — so say that once, up front, in terms the user can act on,
      // instead of letting them record and then hit a config-file error.
      if (!this._warnedNoSR) {
        this._warnedNoSR = true;
        this.messages.push({
          id: ++mId, role: 'system', ts: Date.now(),
          html: 'This browser has no built-in speech recognition (Firefox does not support it). ' +
                'For <strong>free</strong> voice with no key, use the <strong>FreEco.ai desktop app</strong>, Chrome, or Edge. ' +
                'To use voice in <em>any</em> browser, add a Groq API key (free tier) in ' +
                '<a href="#" onclick="window.dispatchEvent(new CustomEvent(\'freeco-navigate\',{detail:\'settings\'}));return false;">Settings → Providers</a>. ' +
                'Recording anyway — it will work if a speech-to-text service is already set up.'
        });
        this._scroll();
      }

      if (!navigator.mediaDevices || !window.MediaRecorder) {
        if (typeof OpenFangToast !== 'undefined') OpenFangToast.error('Voice not supported in this browser');
        return;
      }
      var self = this;
      try {
        var stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        var mime = MediaRecorder.isTypeSupported('audio/webm;codecs=opus') ? 'audio/webm;codecs=opus'
                 : MediaRecorder.isTypeSupported('audio/webm') ? 'audio/webm' : 'audio/ogg';
        this._chunks = [];
        this._rec = new MediaRecorder(stream, { mimeType: mime });
        this._rec.ondataavailable = function(e) { if (e.data.size > 0) self._chunks.push(e.data); };
        this._rec.onstop = function() { stream.getTracks().forEach(function(t) { t.stop(); }); self._voiceDone(); };
        this._rec.start(250);
        this.recording = true;
        this.recordingTime = 0;
        this._timer = setInterval(function() { self.recordingTime++; }, 1000);
      } catch (e) {
        if (typeof OpenFangToast !== 'undefined') OpenFangToast.error('Microphone access denied');
      }
    },
    stopVoice: function() {
      if (!this.recording) return;
      // Browser speech recognition path
      if (this._recog) {
        try { this._recog.stop(); } catch (e) { /* ignore */ }
        this._recog = null;
        this.recording = false;
        if (this._timer) { clearInterval(this._timer); this._timer = null; }
        return;
      }
      if (!this._rec) return;
      this._rec.stop();
      this.recording = false;
      if (this._timer) { clearInterval(this._timer); this._timer = null; }
    },
    _voiceDone: async function() {
      if (!this._chunks.length || !this.agent) return;
      var blob = new Blob(this._chunks, { type: this._chunks[0].type || 'audio/webm' });
      this._chunks = [];
      if (blob.size < 100) return;
      var self = this;
      this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: '<span class="freeco-typing">transcribing…</span>', thinking: true });
      this._scroll();
      try {
        var ext = blob.type.indexOf('webm') !== -1 ? 'webm' : blob.type.indexOf('ogg') !== -1 ? 'ogg' : 'mp3';
        var file = new File([blob], 'voice_' + Date.now() + '.' + ext, { type: blob.type });
        var up = await OpenFangAPI.upload(this.agent.id, file);
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        var said = (up.transcription && up.transcription.trim()) ? up.transcription.trim() : '';
        if (said) {
          this.input = said;
          this.send();
        } else {
          this.messages.push({
            id: ++mId, role: 'system', ts: Date.now(),
            html: this._escape(up.transcription_error || 'Could not transcribe that audio. Check your speech-to-text setup in Settings, or type instead.')
          });
          this._scroll();
        }
      } catch (e) {
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: 'Voice upload failed: ' + self._escape(e.message || 'unknown') });
        this._scroll();
      }
    },
    voiceTime: function() {
      var m = Math.floor(this.recordingTime / 60), s = this.recordingTime % 60;
      return (m < 10 ? '0' : '') + m + ':' + (s < 10 ? '0' : '') + s;
    },

    // ---- Spoken replies (offline browser TTS) ----
    showVoiceMenu: false,
    voiceList: [],
    voiceName: (function() { try { return localStorage.getItem('freeco-voice-name') || ''; } catch (e) { return ''; } })(),

    // Every voice the OS offers, English first, so the user can actually choose
    // instead of being stuck with whatever we guessed.
    refreshVoiceList: function() {
      if (!window.speechSynthesis) { this.voiceList = []; return; }
      var all = window.speechSynthesis.getVoices() || [];
      // PRIVACY: a voice named "... Online (Natural)" is synthesised on
      // Microsoft's servers, which means the text Freeco speaks is sent to
      // them. `localService` is the spec's authoritative flag for on-device
      // synthesis; the name check is a belt-and-braces fallback for engines
      // that report it wrongly. Cloud voices are never promoted and are
      // labelled, so choosing one is a deliberate, informed act.
      var isCloud = function(v) {
        return v.localService === false || /\bonline\b/i.test(v.name || '');
      };
      var isNat = function(v) { return /natural|neural|premium|enhanced|siri/i.test(v.name || ''); };
      var isEn = function(v) { return /^en(-|_|$)/i.test(v.lang || ''); };
      // Order: local natural first, then other local, then anything cloud.
      var rank = function(v) {
        return (isCloud(v) ? 8 : 0) + (isNat(v) ? 0 : 2) + (isEn(v) ? 0 : 1);
      };
      this.voiceList = all.slice().sort(function(a, b) { return rank(a) - rank(b); })
        .map(function(v) {
          return {
            name: v.name,
            lang: v.lang,
            cloud: isCloud(v),
            natural: isNat(v) && !isCloud(v)
          };
        });
      // Only LOCAL natural voices count as "you already have good voices".
      this.hasNatural = this.voiceList.some(function(v) { return v.natural; });
    },
    hasNatural: false,

    // Speak a short sample so the user can hear a voice before choosing it.
    previewVoice: function(name) {
      if (!window.speechSynthesis) return;
      try { window.speechSynthesis.cancel(); } catch (e) { /* ignore */ }
      var v = (window.speechSynthesis.getVoices() || []).find(function(x) { return x.name === name; });
      var u = new SpeechSynthesisUtterance("Hi, I'm Freeco. This is how I sound.");
      if (v) u.voice = v;
      u.rate = 1.0; u.pitch = 0.95;
      window.speechSynthesis.speak(u);
    },

    // Cloud voices are OFF unless the user deliberately unlocks them: selecting
    // one means every word Freeco says is transmitted to a third party. A label
    // is not enough — the safe thing must be the default, and the unsafe thing
    // must take a conscious act.
    allowCloudVoices: false,

    unlockCloudVoices: function() {
      var ok = window.confirm(
        'Cloud voices send everything Freeco says to a third-party server to be spoken.\n\n' +
        'That includes anything private in its replies: names, finances, health, business plans.\n\n' +
        'Local voices never transmit anything. Enable cloud voices anyway?'
      );
      if (ok) this.allowCloudVoices = true;
    },

    chooseVoice: function(name) {
      var v = (window.speechSynthesis.getVoices() || []).find(function(x) { return x.name === name; });
      var entry = this.voiceList.find(function(x) { return x.name === name; });
      if (entry && entry.cloud && !this.allowCloudVoices) {
        this.unlockCloudVoices();
        if (!this.allowCloudVoices) return; // declined: keep the local voice
      }
      this.voiceName = name;
      try { localStorage.setItem('freeco-voice-name', name); } catch (e) { /* ignore */ }
      this._voice = v || null;
      this.showVoiceMenu = false;
      this.previewVoice(name);
    },

    // Honour an explicit choice; otherwise fall back to a sensible default.
    _pickVoice: function() {
      if (!window.speechSynthesis) return null;
      var voices = window.speechSynthesis.getVoices() || [];
      if (!voices.length) return null;
      // Never auto-select a voice that ships text off the machine. Only an
      // explicit, confirmed choice can do that.
      var allowCloud = this.allowCloudVoices;
      var safe = voices.filter(function(v) {
        return allowCloud || (v.localService !== false && !/\bonline\b/i.test(v.name || ''));
      });
      if (!safe.length) safe = voices; // nothing local available at all
      var want = this.voiceName;
      if (want) {
        var chosen = safe.find(function(v) { return v.name === want; });
        if (chosen) return chosen;
      }
      var en = safe.filter(function(v) { return /^en(-|_|$)/i.test(v.lang || ''); });
      var pool = en.length ? en : safe;
      // Preference order: clear, warm voices. "onyx"-style names first.
      var prefer = ['onyx', 'david', 'guy', 'daniel', 'james', 'george', 'ryan', 'brian', 'aaron', 'fred', 'male'];
      for (var i = 0; i < prefer.length; i++) {
        var hit = pool.find(function(v) { return (v.name || '').toLowerCase().indexOf(prefer[i]) !== -1; });
        if (hit) return hit;
      }
      var female = /zira|female|susan|hazel|linda|catherine|samantha|victoria|karen|moira|tessa|fiona/i;
      var notFemale = pool.find(function(v) { return !female.test(v.name || ''); });
      return notFemale || pool[0];
    },

    speak: function(text) {
      if (!window.speechSynthesis || !window.SpeechSynthesisUtterance) return;
      var clean = String(text || '')
        .replace(/```[\s\S]*?```/g, ' code block ')  // don't read code aloud
        .replace(/[*_`#>|]/g, '')
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')      // links -> link text
        .replace(/\s+/g, ' ')
        .trim();
      if (!clean) return;
      try { window.speechSynthesis.cancel(); } catch (e) { /* ignore */ }
      if (!this._voice) this._voice = this._pickVoice();
      var self = this;

      // Speak the WHOLE reply. Two things used to cut it off:
      //  - it was hard-truncated to 600 characters here;
      //  - Chrome silently stops a single utterance after roughly 15 seconds.
      // Splitting on sentence boundaries and queueing the pieces fixes both,
      // and gives natural pauses between sentences.
      var chunks = clean.match(/[^.!?]{1,180}([.!?]+|$)\s*/g) || [clean];
      this._queueLen = chunks.length;
      chunks.forEach(function(part, idx) {
        var u = new SpeechSynthesisUtterance(part.trim());
        if (self._voice) u.voice = self._voice;
        u.rate = 1.0;
        u.pitch = 0.95;
        u.volume = 1.0;
        if (idx === 0) u.onstart = function() { self.speaking = true; self.paused = false; };
        if (idx === chunks.length - 1) {
          u.onend = function() { self.speaking = false; self.paused = false; };
        }
        u.onerror = function() { self.speaking = false; self.paused = false; };
        window.speechSynthesis.speak(u);
      });
      return;
    },

    stopSpeaking: function() {
      try { window.speechSynthesis.cancel(); } catch (e) { /* ignore */ }
      this.speaking = false;
    },

    toggleVoiceOut: function() {
      this.voiceOut = !this.voiceOut;
      try { localStorage.setItem('freeco-voice-out', this.voiceOut ? 'on' : 'off'); } catch (e) { /* ignore */ }
      if (!this.voiceOut) this.stopSpeaking();
    },

    // Pause / resume the current spoken reply (go/pause/stop controls).
    pauseSpeaking: function() {
      try { window.speechSynthesis.pause(); this.paused = true; } catch (e) { /* ignore */ }
    },
    resumeSpeaking: function() {
      try { window.speechSynthesis.resume(); this.paused = false; } catch (e) { /* ignore */ }
    },

    // ---- Window: full-screen, move, resize ----
    toggleFullscreen: function() {
      this.fullscreen = !this.fullscreen;
      var self = this; this.$nextTick(function() { self._scroll(); });
    },
    // Computed inline style for the panel based on window state.
    panelStyle: function() {
      if (this.fullscreen) {
        return 'position:fixed;inset:12px;width:auto;height:auto;max-height:none;border-radius:14px;z-index:9600';
      }
      if (this.moved) {
        return 'position:fixed;left:' + this.pos.x + 'px;top:' + this.pos.y + 'px;right:auto;bottom:auto;' +
               'width:' + this.size.w + 'px;height:' + this.size.h + 'px;max-height:none;z-index:9600';
      }
      // default: docked bottom-right, user-set size
      return 'width:' + this.size.w + 'px;height:' + this.size.h + 'px';
    },
    startDrag: function(e) {
      if (this.fullscreen) return;
      // Switch to free positioning anchored at the current on-screen spot.
      var panel = e.currentTarget.closest('.freeco-panel');
      var r = panel.getBoundingClientRect();
      this.moved = true; this.pos = { x: r.left, y: r.top };
      var startX = e.clientX, startY = e.clientY, ox = this.pos.x, oy = this.pos.y, self = this;
      this._drag = function(ev) {
        self.pos = {
          x: Math.max(0, Math.min(window.innerWidth - 120, ox + ev.clientX - startX)),
          y: Math.max(0, Math.min(window.innerHeight - 40, oy + ev.clientY - startY))
        };
      };
      var up = function() { window.removeEventListener('mousemove', self._drag); window.removeEventListener('mouseup', up); };
      window.addEventListener('mousemove', this._drag);
      window.addEventListener('mouseup', up);
      e.preventDefault();
    },
    // Widen from the left edge: the panel is anchored bottom-right, so growing
    // leftwards means increasing width without moving the right edge.
    startResizeLeft: function(e) {
      if (this.fullscreen) return;
      var startX = e.clientX, ow = this.size.w, self = this;
      var ox = this.moved ? this.pos.x : null;
      var move = function(ev) {
        var dx = startX - ev.clientX;
        self.size = { w: Math.max(300, Math.min(window.innerWidth - 20, ow + dx)), h: self.size.h };
        if (ox !== null) self.pos = { x: Math.max(0, ox - dx), y: self.pos.y };
      };
      var up = function() { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
      window.addEventListener('mousemove', move);
      window.addEventListener('mouseup', up);
      e.preventDefault();
    },

    startResize: function(e) {
      if (this.fullscreen) return;
      var startX = e.clientX, startY = e.clientY, ow = this.size.w, oh = this.size.h, self = this;
      var move = function(ev) {
        self.size = {
          w: Math.max(300, Math.min(window.innerWidth - 20, ow + ev.clientX - startX)),
          h: Math.max(320, Math.min(window.innerHeight - 20, oh + ev.clientY - startY))
        };
      };
      var up = function() { window.removeEventListener('mousemove', move); window.removeEventListener('mouseup', up); };
      window.addEventListener('mousemove', move);
      window.addEventListener('mouseup', up);
      e.preventDefault();
    },

    // ---- Attach: file, folder, photo ----
    pickFile: function() { this.showAttach = false; var el = document.getElementById('freeco-file'); if (el) el.click(); },
    pickFolder: function() { this.showAttach = false; var el = document.getElementById('freeco-folder'); if (el) el.click(); },
    takePhoto: function() { this.showAttach = false; var el = document.getElementById('freeco-photo'); if (el) el.click(); },

    handleFiles: async function(fileList) {
      var files = Array.prototype.slice.call(fileList || []);
      if (!files.length) return;
      if (!this.agent) { this._resolveAgent(); if (!this.agent) await this._ensureConcierge(); }
      if (!this.agent) { OpenFangToast.error('Set up an agent first, then attach files.'); return; }
      this.attaching = true;
      for (var i = 0; i < files.length; i++) {
        var f = files[i];
        // Guard against a whole huge folder — cap per file at 25 MB.
        if (f.size > 25 * 1048576) {
          this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: this._escape(f.name + ' is too large (over 25 MB), skipped.') });
          continue;
        }
        try {
          var up = await OpenFangAPI.upload(this.agent.id, f);
          this.attachments.push({ name: up.filename || f.name, kind: (f.type || '').split('/')[0] || 'file' });
        } catch (e) {
          this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: 'Could not attach ' + this._escape(f.name) + ': ' + this._escape(e.message || 'error') });
        }
      }
      this.attaching = false;
      this._scroll();
    },
    removeAttachment: function(name) {
      this.attachments = this.attachments.filter(function(a) { return a.name !== name; });
    },

    onKey: function(e) {
      if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); this.send(); }
    },

    _scroll: function() {
      this.$nextTick(function() {
        var el = document.getElementById('freeco-thread');
        if (el) el.scrollTop = el.scrollHeight;
      });
    },
    _escape: function(s) {
      return String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    },
    _md: function(s) {
      try {
        if (window.marked) {
          var html = window.marked.parse(String(s));
          return html;
        }
      } catch (e) { /* fall through */ }
      return '<p>' + this._escape(s).replace(/\n/g, '<br>') + '</p>';
    }
  };
}
