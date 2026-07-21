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
      { id: 'channel',  label: 'Connect email / site / channel', page: 'channels', icon: 'M4 4h16v16H4zM22 6l-10 7L2 6',
        seed: 'Help me connect a channel — email, a website, or a domain — so my agents can act in the real world.' },
      { id: 'localai',  label: 'Set up free local AI', page: 'settings', icon: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20',
        seed: 'Set up free local AI (Ollama + Gemma) so I can run privately with no cloud cost.' }
    ],

    init: function() {
      var self = this;
      // Re-resolve the concierge agent whenever the shared agent list changes.
      this.$watch('$store.app.agents', function() { self._resolveAgent(); });
      this._resolveAgent();
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

    toggle: function() {
      this.open = !this.open;
      if (this.open) {
        this._resolveAgent();
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
        html: '<p>' + hi + "I'm <strong>Freeco</strong>, your AI concierge. Tell me what you want to build — a company, a nonprofit, a workflow — and I'll help you set up the agents, tools and channels to run it. Pick a shortcut below or just type/talk.</p>"
      });
    },

    // Quick-setup: jump to the relevant builder and hand Freeco a guiding prompt.
    quick: function(topic) {
      window.dispatchEvent(new CustomEvent('freeco-navigate', { detail: topic.page }));
      this.input = topic.seed;
      if (this.agent) { this.send(); }
      else {
        this.messages.push({ id: ++mId, role: 'freeco', ts: Date.now(),
          html: '<p>I’ve opened the <strong>' + topic.page + '</strong> page for you. Create your first agent there and I’ll guide the rest.</p>' });
        this._scroll();
      }
    },

    send: async function() {
      var text = (this.input || '').trim();
      if (!text || this.sending) return;
      if (!this.agent) { this._resolveAgent(); }
      if (!this.agent) {
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(),
          html: 'You don’t have any agents yet. <a href="#" onclick="window.dispatchEvent(new CustomEvent(\'freeco-navigate\',{detail:\'agents\'}));return false;">Create your Freeco concierge agent</a> to get started.' });
        this.input = ''; this._scroll(); return;
      }
      this.messages.push({ id: ++mId, role: 'user', ts: Date.now(), html: this._escape(text) });
      this.input = '';
      this.sending = true;
      var thinking = { id: ++mId, role: 'freeco', ts: Date.now(), html: '<span class="freeco-typing">• • •</span>', thinking: true };
      this.messages.push(thinking);
      this._scroll();
      try {
        var res = await OpenFangAPI.post('/api/agents/' + this.agent.id + '/message', { message: text });
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        this.messages.push({ id: ++mId, role: 'freeco', ts: Date.now(), html: this._md(res.response || '(no reply)') });
      } catch (e) {
        this.messages = this.messages.filter(function(m) { return !m.thinking; });
        this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: 'Error: ' + this._escape(e.message || 'request failed') });
      }
      this.sending = false;
      this._scroll();
      var self = this;
      this.$nextTick(function() { var el = document.getElementById('freeco-input'); if (el) el.focus(); });
    },

    // ---- Voice (hold-to-talk) ----
    startVoice: async function() {
      if (this.recording) return;
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
      if (!this.recording || !this._rec) return;
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
        if (said) { this.input = said; this.send(); }
        else { this.messages.push({ id: ++mId, role: 'system', ts: Date.now(), html: 'Could not transcribe audio — no speech-to-text provider is configured. Set one up in Settings, or type instead.' }); this._scroll(); }
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
