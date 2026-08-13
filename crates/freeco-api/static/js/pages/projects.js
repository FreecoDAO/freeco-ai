// FreEco.ai — Companies, projects and teams.
//
// The structure exists in the database whether or not there is a screen for
// it, but a conversation filed under a project the user cannot see is the
// same as one filed nowhere. This is the screen that makes it real.
'use strict';

function projectsPage() {
  return {
    loading: true,
    loadError: '',
    projects: [],
    companies: [],
    overview: [],
    // Archived work is hidden by default so finished projects do not crowd
    // the list, and reachable in one click so it is never lost.
    showArchived: false,
    selected: null,
    sessions: [],
    sessionsLoading: false,
    newProject: '',
    newCompany: '',
    creating: false,

    async init() {
      await this.load();
    },

    async load() {
      this.loading = true;
      this.loadError = '';
      try {
        var q = this.showArchived ? '?archived=true' : '';
        var results = await Promise.all([
          OpenFangAPI.get('/api/org/projects' + q),
          OpenFangAPI.get('/api/org/companies' + q),
          OpenFangAPI.get('/api/org/overview')
        ]);
        this.projects = (results[0] && results[0].projects) || [];
        this.companies = (results[1] && results[1].companies) || [];
        this.overview = (results[2] && results[2].projects) || [];
      } catch (e) {
        // Say what failed. A silently empty page reads as "you have no
        // projects", which is a different and more alarming message.
        this.loadError = 'Could not load projects: ' + e.message;
      }
      this.loading = false;
    },

    // Conversation counts come from the overview, which is a single query
    // rather than one per project.
    countFor(id) {
      var row = this.overview.find(function (o) { return o.project_id === id; });
      return row ? row.sessions : 0;
    },

    companyName(id) {
      if (!id) return '';
      var c = this.companies.find(function (x) { return x.id === id; });
      return c ? c.name : '';
    },

    async createProject() {
      var name = (this.newProject || '').trim();
      if (!name) return;
      this.creating = true;
      try {
        var res = await OpenFangAPI.post('/api/org/projects', { name: name });
        // The API routes rather than blindly creating, so tell the user when
        // they landed in an existing project instead of a new one.
        if (res && res.reused) {
          OpenFangToast.info('That project already existed — opened it instead of making a second one.');
        } else {
          OpenFangToast.success('Project created.');
        }
        this.newProject = '';
        await this.load();
      } catch (e) {
        OpenFangToast.error('Could not create project: ' + e.message);
      }
      this.creating = false;
    },

    // Teams belong to a project and scope what their agents can see. Created
    // from inside a project so a team can never end up orphaned, which is the
    // state that makes scoping meaningless.
    newTeam: '',
    async createTeam() {
      var name = (this.newTeam || '').trim();
      if (!name || !this.selected) return;
      try {
        await OpenFangAPI.post('/api/org/teams', { name: name, project_id: this.selected.id });
        this.newTeam = '';
        OpenFangToast.success('Team added to ' + this.selected.name + '.');
      } catch (e) {
        OpenFangToast.error('Could not create team: ' + e.message);
      }
    },

    async createCompany() {
      var name = (this.newCompany || '').trim();
      if (!name) return;
      try {
        await OpenFangAPI.post('/api/org/companies', { name: name });
        this.newCompany = '';
        await this.load();
        OpenFangToast.success('Company created.');
      } catch (e) {
        OpenFangToast.error('Could not create company: ' + e.message);
      }
    },

    async select(project) {
      this.selected = project;
      this.sessionsLoading = true;
      this.sessions = [];
      try {
        var res = await OpenFangAPI.get('/api/org/projects/' + encodeURIComponent(project.id) + '/sessions');
        this.sessions = (res && res.sessions) || [];
      } catch (e) {
        OpenFangToast.error('Could not load this project’s conversations: ' + e.message);
      }
      this.sessionsLoading = false;
    },

    // Archive is "done, keep it". Distinct from trash on purpose: if the only
    // way to clear finished work off a list is to make it look deleted,
    // people stop clearing it.
    async setArchived(session, value) {
      try {
        await OpenFangAPI.put('/api/sessions/' + encodeURIComponent(session.id) + '/archive', { value: value });
        session.archived = value;
      } catch (e) {
        OpenFangToast.error('Could not archive: ' + e.message);
      }
    },

    // Trash hides, it does not delete. Nothing here removes a row.
    async trash(session) {
      try {
        await OpenFangAPI.put('/api/sessions/' + encodeURIComponent(session.id) + '/trash', { value: true });
        this.sessions = this.sessions.filter(function (s) { return s.id !== session.id; });
        OpenFangToast.info('Moved to trash. Nothing was deleted — it can be restored.');
      } catch (e) {
        OpenFangToast.error('Could not move to trash: ' + e.message);
      }
    },

    when(ts) {
      if (!ts) return '';
      return String(ts).slice(0, 16).replace('T', ' ');
    }
  };
}
