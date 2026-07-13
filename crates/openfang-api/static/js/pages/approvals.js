// FreEco.ai Approvals Page — Execution approval queue for sensitive agent actions
'use strict';

function approvalsPage() {
  return {
    approvals: [],
    filterStatus: 'all',
    loading: true,
    loadError: '',
    refreshTimer: null,

    init() {
      var self = this;
      this.loadData();
      this.refreshTimer = setInterval(function() {
        self.loadData();
      }, 5000);
    },

    destroy() {
      if (this.refreshTimer) {
        clearInterval(this.refreshTimer);
        this.refreshTimer = null;
      }
    },

    get filtered() {
      var f = this.filterStatus;
      if (f === 'all') return this.approvals;
      return this.approvals.filter(function(a) { return a.status === f; });
    },

    get pendingCount() {
      return this.approvals.filter(function(a) { return a.status === 'pending'; }).length;
    },

    // Plain-language consequence of approving, for non-technical users.
    // Returns { text, severity } where severity is 'danger'|'warn'|'info'.
    // Derived from the action string so it needs no backend change.
    consequence(a) {
      var act = ((a.action || '') + ' ' + (a.description || '')).toLowerCase();
      var has = function(){ for (var i=0;i<arguments.length;i++){ if (act.indexOf(arguments[i])!==-1) return true; } return false; };
      if (has('shell', 'exec', 'command', 'bash', 'powershell'))
        return { severity: 'danger', text: 'This lets the agent run a command on your computer — it could read, change, or delete files. Only approve if you understand the command.' };
      if (has('delete', 'remove', 'destroy', 'wipe', 'drop'))
        return { severity: 'danger', text: 'This permanently removes something and cannot be undone.' };
      if (has('pay', 'buy', 'purchase', 'order', 'checkout', 'transfer', 'wallet', 'transaction'))
        return { severity: 'danger', text: 'This can spend money or make a purchase on your behalf.' };
      if (has('send', 'email', 'post', 'publish', 'message', 'tweet', 'whatsapp', 'telegram'))
        return { severity: 'warn', text: 'This sends something out to other people or the internet — it leaves your device.' };
      if (has('write', 'save', 'modify', 'edit', 'update', 'file'))
        return { severity: 'warn', text: 'This creates or changes a file on your computer.' };
      if (has('fetch', 'browse', 'http', 'url', 'web', 'network', 'download'))
        return { severity: 'warn', text: 'This connects to the internet, which may share some information with an outside website.' };
      return { severity: 'info', text: 'Review what this agent is about to do before approving.' };
    },

    async loadData() {
      this.loading = true;
      this.loadError = '';
      try {
        var data = await OpenFangAPI.get('/api/approvals');
        this.approvals = data.approvals || [];
      } catch(e) {
        this.loadError = e.message || 'Could not load approvals.';
      }
      this.loading = false;
    },

    async approve(id) {
      try {
        await OpenFangAPI.post('/api/approvals/' + id + '/approve', {});
        OpenFangToast.success('Approved');
        await this.loadData();
      } catch(e) {
        OpenFangToast.error(e.message);
      }
    },

    async reject(id) {
      var self = this;
      OpenFangToast.confirm('Reject Action', 'Are you sure you want to reject this action?', async function() {
        try {
          await OpenFangAPI.post('/api/approvals/' + id + '/reject', {});
          OpenFangToast.success('Rejected');
          await self.loadData();
        } catch(e) {
          OpenFangToast.error(e.message);
        }
      });
    },

    timeAgo(dateStr) {
      if (!dateStr) return '';
      var d = new Date(dateStr);
      var secs = Math.floor((Date.now() - d.getTime()) / 1000);
      if (secs < 60) return secs + 's ago';
      if (secs < 3600) return Math.floor(secs / 60) + 'm ago';
      if (secs < 86400) return Math.floor(secs / 3600) + 'h ago';
      return Math.floor(secs / 86400) + 'd ago';
    }
  };
}
