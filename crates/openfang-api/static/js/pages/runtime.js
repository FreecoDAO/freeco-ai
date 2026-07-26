// Runtime page — system overview and provider status
document.addEventListener('alpine:init', function() {
  Alpine.data('runtimePage', function() {
    return {
      loading: true,
      uptime: '-',
      agentCount: 0,
      version: '-',
      defaultModel: '-',
      platform: '-',
      arch: '-',
      apiListen: '-',
      homeDir: '-',
      logLevel: '-',
      networkEnabled: false,
      providers: [],
      company: { teams: [], workflows: [], approvals: { security_pending: 0 } },
      backupStatus: '',
      backups: [],
      restoreBusy: false,
      restoreResult: null,

      async loadData() {
        this.loading = true;
        try {
          var results = await Promise.all([
            OpenFangAPI.get('/api/status'),
            OpenFangAPI.get('/api/version'),
            OpenFangAPI.get('/api/providers'),
            OpenFangAPI.get('/api/agents'),
            OpenFangAPI.get('/api/company-chart')
          ]);
          var status = results[0];
          var ver = results[1];
          var prov = results[2];
          var agents = results[3];
          this.company = results[4] || this.company;

          this.version = ver.version || '-';
          this.platform = ver.platform || '-';
          this.arch = ver.arch || '-';
          this.agentCount = Array.isArray(agents) ? agents.length : 0;
          this.defaultModel = status.default_model || '-';
          this.apiListen = status.api_listen || status.listen || '-';
          this.homeDir = status.home_dir || '-';
          this.logLevel = status.log_level || '-';
          this.networkEnabled = !!status.network_enabled;

          // Compute uptime from uptime_seconds
          var diff = status.uptime_seconds || 0;
          if (diff < 60) this.uptime = diff + 's';
          else if (diff < 3600) this.uptime = Math.floor(diff / 60) + 'm ' + (diff % 60) + 's';
          else if (diff < 86400) this.uptime = Math.floor(diff / 3600) + 'h ' + Math.floor((diff % 3600) / 60) + 'm';
          else this.uptime = Math.floor(diff / 86400) + 'd ' + Math.floor((diff % 86400) / 3600) + 'h';

          this.providers = (prov.providers || []).filter(function(p) {
            return p.auth_status === 'Configured' || p.reachable || p.is_local;
          });
        } catch(e) {
          console.error('Runtime load error:', e);
        }
        this.loading = false;
      },

      async createBackup() {
        this.backupStatus = 'Creating encrypted backup...';
        try {
          var result = await OpenFangAPI.post('/api/backups', { retention: 7 });
          this.backupStatus = 'Backup created: ' + (result.files || '') + ' files';
          OpenFangToast.success(this.backupStatus);
          await this.loadBackups();
        } catch (e) {
          this.backupStatus = 'Backup failed';
          OpenFangToast.error('Encrypted backup failed');
        }
      },

      async loadBackups() {
        try {
          var data = await OpenFangAPI.get('/api/backups');
          this.backups = (data.backups || []).map(function(b) {
            return {
              name: b.name,
              sizeMb: (b.size / 1048576).toFixed(1),
              when: b.modified ? new Date(b.modified * 1000).toLocaleString() : ''
            };
          });
        } catch (e) { this.backups = []; }
      },

      // Verify a backup first (dry run) — never destructive.
      async verifyRestore(name) {
        this.restoreBusy = true;
        this.restoreResult = null;
        try {
          var r = await OpenFangAPI.post('/api/backups/restore', { archive_name: name, dry_run: true });
          this.restoreResult = { name: name, ok: true, detail: 'Verified: ' + (r.files || 0) + ' files, archive is intact and decryptable.' };
        } catch (e) {
          this.restoreResult = { name: name, ok: false, detail: 'Verify failed: ' + (e.message || 'archive unreadable') };
        }
        this.restoreBusy = false;
      },

      restoreBackup(name) {
        var self = this;
        OpenFangToast.confirm(
          'Restore backup',
          'Restore "' + name + '"? This overwrites current data with the archive contents. A restart is needed afterwards. Verify first if unsure.',
          async function() {
            self.restoreBusy = true;
            try {
              await OpenFangAPI.post('/api/backups/restore', { archive_name: name, dry_run: false });
              OpenFangToast.success('Restore complete — restart FreEco.ai to load the restored data.');
              self.restoreResult = { name: name, ok: true, detail: 'Restored. Restart to apply.' };
            } catch (e) {
              OpenFangToast.error('Restore failed: ' + (e.message || 'error'));
            }
            self.restoreBusy = false;
          }
        );
      }
    };
  });
});
