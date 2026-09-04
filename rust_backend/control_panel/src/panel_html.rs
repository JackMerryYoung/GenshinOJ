// Extended HTML panel with system monitoring, user management, and problem management
pub const PANEL_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>RsOJ Control Panel</title>
<style>
  :root {
    --colorBrandBackground: #0f6cbd;
    --colorBrandBackgroundHover: #115ea3;
    --colorBrandBackgroundPressed: #0c3b5e;
    --colorDangerBackground: #c50f1f;
    --colorDangerBackgroundHover: #b10e1c;
    --colorDangerBackgroundPressed: #960b18;
    --colorNeutralForeground1: #242424;
    --colorNeutralForeground3: #616161;
    --colorNeutralBackground1: #ffffff;
    --colorNeutralBackgroundCanvas: #f5f5f5;
    --colorNeutralBackground3: #f0f0f0;
    --colorNeutralStroke1: #d1d1d1;
    --colorSuccessForeground: #0e700e;
    --colorDangerForeground: #b10e1c;
    --shadow16: 0 8px 16px rgba(0,0,0,.14), 0 0 2px rgba(0,0,0,.12);
    --borderRadiusMedium: 4px;
    --borderRadiusLarge: 8px;
    --fontFamilyBase: "Segoe UI", "Segoe UI Web (West European)", -apple-system, system-ui, sans-serif;
  }
  body { font-family: var(--fontFamilyBase); margin: 0;
         background: var(--colorNeutralBackgroundCanvas); color: var(--colorNeutralForeground1); }
  .auth-wrap { min-height: 100vh; display: flex; align-items: center; justify-content: center; }
  .card { background: var(--colorNeutralBackground1); padding: 24px; border-radius: var(--borderRadiusLarge);
          width: 380px; box-shadow: var(--shadow16); }
  .app { min-height: 100vh; display: flex; }
  .sidebar { width: 220px; box-sizing: border-box; background: var(--colorNeutralBackground1);
             border-right: 1px solid var(--colorNeutralStroke1); padding: 16px 0; }
  .sidebar .brand { font-size: 16px; font-weight: 600; padding: 8px 20px 16px; }
  .nav-item { display: block; padding: 8px 20px; font-size: 14px; line-height: 20px; cursor: pointer;
              color: var(--colorNeutralForeground1); text-decoration: none;
              border-left: 3px solid transparent; }
  .nav-item:hover { background: var(--colorNeutralBackground3); }
  .nav-item.active { border-left-color: var(--colorBrandBackground); background: var(--colorNeutralBackground3);
                     color: var(--colorBrandBackground); font-weight: 600; }
  .content { flex: 1; padding: 32px; box-sizing: border-box; overflow-y: auto; }
  .panel-card { background: var(--colorNeutralBackground1); border-radius: var(--borderRadiusLarge);
                box-shadow: var(--shadow16); padding: 24px; margin-bottom: 20px; }
  h1 { font-size: 20px; line-height: 28px; font-weight: 600; margin: 0 0 20px; }
  h2 { font-size: 16px; line-height: 22px; font-weight: 600; margin: 0 0 12px; }
  label { display: block; font-size: 14px; line-height: 20px; margin-bottom: 4px;
          color: var(--colorNeutralForeground1); }
  input, textarea { width: 100%; box-sizing: border-box; padding: 0 10px;
          border-radius: var(--borderRadiusMedium); border: 1px solid var(--colorNeutralStroke1);
          border-bottom-color: #616161; background: var(--colorNeutralBackground1);
          color: var(--colorNeutralForeground1); font-size: 14px; font-family: inherit;
          margin-bottom: 16px; outline: none; }
  input { height: 32px; }
  textarea { padding: 10px; min-height: 100px; }
  input:focus, textarea:focus { border-color: var(--colorBrandBackground);
                border-bottom: 2px solid var(--colorBrandBackground); }
  button { min-height: 32px; padding: 5px 12px; border: 1px solid transparent;
           border-radius: var(--borderRadiusMedium); cursor: pointer; font-size: 14px;
           line-height: 20px; font-weight: 600; font-family: inherit; color: #fff; }
  button.primary { background: var(--colorBrandBackground); }
  button.primary:hover { background: var(--colorBrandBackgroundHover); }
  button.primary:active { background: var(--colorBrandBackgroundPressed); }
  button.danger { background: var(--colorDangerBackground); }
  button.danger:hover { background: var(--colorDangerBackgroundHover); }
  button.danger:active { background: var(--colorDangerBackgroundPressed); }
  button.secondary { background: var(--colorNeutralBackground1);
                     color: var(--colorNeutralForeground1); border: 1px solid var(--colorNeutralStroke1); }
  button.secondary:hover { background: var(--colorNeutralBackground3); }
  button:disabled { opacity: .5; cursor: not-allowed; }
  .hint { font-size: 12px; line-height: 16px; color: var(--colorNeutralForeground3);
          margin-top: 12px; }
  .status { margin-top: 16px; font-size: 14px; line-height: 20px; min-height: 20px; }
  .status.ok { color: var(--colorSuccessForeground); }
  .status.err { color: var(--colorDangerForeground); }
  .hidden { display: none; }
  table { width: 100%; border-collapse: collapse; }
  th, td { text-align: left; padding: 12px; border-bottom: 1px solid var(--colorNeutralStroke1); }
  th { font-weight: 600; background: var(--colorNeutralBackground3); }
  .badge { display: inline-block; padding: 2px 8px; border-radius: 12px; font-size: 12px; font-weight: 600; }
  .badge.success { background: #dff6dd; color: #0e700e; }
  .badge.danger { background: #fde7e9; color: #b10e1c; }
  .badge.warning { background: #fff4ce; color: #8a5700; }
  .stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 20px; }
  .stat-item { padding: 16px; background: var(--colorNeutralBackground3); border-radius: var(--borderRadiusMedium); }
  .stat-value { font-size: 24px; font-weight: 600; margin-bottom: 4px; }
  .stat-label { font-size: 12px; color: var(--colorNeutralForeground3); }
  .actions { display: flex; gap: 8px; margin-top: 16px; }
  .pagination { display: flex; gap: 8px; align-items: center; justify-content: center; margin-top: 16px; }
  .search-box { margin-bottom: 16px; }
  .card-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
  .modal { display: none; position: fixed; z-index: 1000; left: 0; top: 0; width: 100%; height: 100%;
           background-color: rgba(0,0,0,0.4); }
  .modal.hidden { display: none; }
  .modal:not(.hidden) { display: flex; align-items: center; justify-content: center; }
  .modal-content { background: var(--colorNeutralBackground1); padding: 24px; border-radius: var(--borderRadiusLarge);
                   width: 600px; max-width: 90%; max-height: 90vh; overflow-y: auto; box-shadow: var(--shadow16); }
  select { font-family: inherit; }
</style>
<script src="https://code.highcharts.com/highcharts.js"></script>
</head>
<body>
  <div id="auth" class="auth-wrap">
    <div class="card">
      <h1>RsOJ Control Panel</h1>
      <section id="setup" class="hidden">
        <h2>Server configuration required</h2>
        <p class="hint">Set a CONTROL_PANEL_*_TOKEN environment variable and restart the backend.</p>
      </section>
      <section id="login" class="hidden">
        <h2>Admin login</h2>
        <label for="loginToken">Admin token</label>
        <input id="loginToken" type="password" placeholder="admin token" autocomplete="off" />
        <button id="unlock" class="primary">Unlock</button>
      </section>
      <div id="authStatus" class="status"></div>
    </div>
  </div>

  <div id="app" class="app hidden">
    <aside class="sidebar">
      <div class="brand">RsOJ</div>
      <a class="nav-item" data-route="monitor">System Monitor</a>
      <a class="nav-item" data-route="dashboard">Analytics</a>
      <a class="nav-item" data-route="users">Users</a>
      <a class="nav-item" data-route="problems">Problems</a>
      <a class="nav-item" data-route="database">Database</a>
    </aside>
    <main class="content">
      <!-- System Monitor -->
      <section id="view-monitor" class="view hidden">
        <h1>System Monitor</h1>
        <div class="panel-card">
          <h2>System Status</h2>
          <div id="systemStatus"></div>
        </div>
        <div class="panel-card">
          <h2>Database Statistics</h2>
          <div id="dbStats"></div>
        </div>
      </section>

      <!-- Analytics Dashboard -->
      <section id="view-dashboard" class="view hidden">
        <h1>Analytics Dashboard</h1>
        <div id="charts"></div>
      </section>

      <!-- User Management -->
      <section id="view-users" class="view hidden">
        <h1>User Management</h1>
        <div class="panel-card">
          <div class="search-box">
            <input id="userSearch" type="text" placeholder="Search users..." />
          </div>
          <div id="userList"></div>
        </div>
      </section>

      <!-- Problem Management -->
      <section id="view-problems" class="view hidden">
        <h1>Problem Management</h1>
        <div class="panel-card">
          <div class="search-box">
            <input id="problemSearch" type="text" placeholder="Search problems..." />
          </div>
          <button id="createProblemBtn" class="primary" style="margin-bottom: 16px;">Create Problem</button>
          <div id="problemList"></div>
        </div>
      </section>

      <!-- Create Problem Modal -->
      <div id="createProblemModal" class="modal hidden">
        <div class="modal-content">
          <h2>Create New Problem</h2>
          <label for="newProblemNumber">Problem Number</label>
          <input id="newProblemNumber" type="number" placeholder="e.g., 1003" min="1" />
          <label for="newProblemName">Problem Name</label>
          <input id="newProblemName" type="text" placeholder="e.g., Two Sum" />
          <label for="newProblemDifficulty">Difficulty</label>
          <select id="newProblemDifficulty" style="width: 100%; height: 32px; padding: 0 10px; border-radius: 4px; border: 1px solid #d1d1d1; margin-bottom: 16px; font-size: 14px; font-family: inherit;">
            <option value="0">0 - Unknown</option>
            <option value="1">1 - Beginner</option>
            <option value="2">2 - Primary</option>
            <option value="3">3 - Junior</option>
            <option value="4">4 - Senior</option>
            <option value="5">5 - Advanced</option>
            <option value="6">6 - Hard</option>
            <option value="7">7 - Grand</option>
          </select>
          <label for="newProblemStatement">Problem Statement (JSON array format)</label>
          <textarea id="newProblemStatement" placeholder='["Problem description line 1", "Line 2", "Line 3"]' style="min-height: 120px;"></textarea>
          <p class="hint">Enter problem statement as a JSON array of strings. Each string is a paragraph.</p>
          <label for="newTestcaseConfig">Testcase Configuration (JSON format, optional)</label>
          <textarea id="newTestcaseConfig" placeholder='{"testcases": [{"number": 1, "score": 100, "input": "input1.txt", "answer": "answer1.txt", "time_limit": 1.0, "memory_limit": 256}]}' style="min-height: 120px;"></textarea>
          <p class="hint">Testcase config as JSON. Leave empty if test files will be uploaded separately.</p>
          <div class="actions">
            <button id="submitCreateProblem" class="primary">Create</button>
            <button id="cancelCreateProblem" class="secondary">Cancel</button>
          </div>
          <div id="createProblemStatus" class="status"></div>
        </div>
      </div>

      <!-- Database Management -->
      <section id="view-database" class="view hidden">
        <h1>Database Management</h1>
        <div class="panel-card">
          <h2>Clear Database</h2>
          <button id="clear" class="danger">Clear all data</button>
          <p class="hint">Truncates all OJ tables. Cannot be undone.</p>
        </div>
      </section>

      <div id="appStatus" class="status"></div>
    </main>
  </div>
<script>
  let sessionToken = '';
  const authEl = document.getElementById('auth');
  const appEl  = document.getElementById('app');
  const authStatusEl = document.getElementById('authStatus');
  const appStatusEl  = document.getElementById('appStatus');

  // ── helpers ──────────────────────────────────────────────────────────────
  function setStatus(el, msg, ok) {
    el.className = 'status' + (msg ? (ok ? ' ok' : ' err') : '');
    el.textContent = msg || '';
  }
  function setAuthStatus(msg, ok) { setStatus(authStatusEl, msg, ok); }
  function setAppStatus (msg, ok) { setStatus(appStatusEl,  msg, ok); }

  function btnBusy(btn, label) { btn.disabled = true;  btn.dataset.orig = btn.textContent; btn.textContent = label; }
  function btnDone(btn)        { btn.disabled = false; btn.textContent = btn.dataset.orig; }

  async function postJson(url, body) {
    const token = body?.token || sessionToken;
    const headers = { 'Content-Type': 'application/json' };
    if (token) headers.Authorization = 'Bearer ' + token;
    const payload = { ...(body || {}) };
    delete payload.token;
    const res = await fetch(url, {
      method: 'POST',
      headers,
      body: JSON.stringify(payload),
    });
    return { res, data: await res.json() };
  }

  // ── session ───────────────────────────────────────────────────────────────
  const SESSION_KEY    = 'rsoj_cp_session';
  const SESSION_TTL_MS = 30 * 60 * 1000;
  function persistSession(token) {
    try { sessionStorage.setItem(SESSION_KEY, JSON.stringify({ token, ts: Date.now() })); } catch (_) {}
  }
  function loadSession() {
    try {
      const s = JSON.parse(sessionStorage.getItem(SESSION_KEY));
      if (!s?.token || !s?.ts) return null;
      if (Date.now() - s.ts > SESSION_TTL_MS) { sessionStorage.removeItem(SESSION_KEY); return null; }
      return s;
    } catch (_) { return null; }
  }
  function clearSession() { try { sessionStorage.removeItem(SESSION_KEY); } catch (_) {} }

  // ── auth flow ─────────────────────────────────────────────────────────────
  async function init() {
    try {
      const res  = await fetch('/api/status');
      const data = await res.json();
      if (!data.configured) {
        document.getElementById('setup').classList.remove('hidden');
        setAuthStatus('No control-panel role token is configured.', false);
      } else {
        const session = loadSession();
        if (session && await authenticate(session.token, true)) return;
        clearSession();
        document.getElementById('login').classList.remove('hidden');
      }
      authEl.classList.remove('hidden');
    } catch (e) {
      setAuthStatus('Could not reach the server: ' + e, false);
      authEl.classList.remove('hidden');
      document.getElementById('login').classList.remove('hidden');
    }
  }

  // Login — button + Enter key
  const unlockBtn    = document.getElementById('unlock');
  const loginTokenEl = document.getElementById('loginToken');
  unlockBtn.addEventListener('click', doLogin);
  loginTokenEl.addEventListener('keydown', e => { if (e.key === 'Enter') doLogin(); });

  function doLogin() { authenticate(loginTokenEl.value.trim(), false); }

  async function authenticate(token, silent) {
    if (!token) {
      if (!silent) setAuthStatus('Please enter the admin token.', false);
      return false;
    }
    const btn = unlockBtn;
    if (!silent) { btnBusy(btn, 'Verifying…'); setAuthStatus('', true); }
    try {
      const { res, data } = await postJson('/api/login', { token });
      if (res.ok && data.ok) {
        sessionToken = token;
        persistSession(token);
        authEl.classList.add('hidden');
        appEl.classList.remove('hidden');
        if (!location.hash || location.hash === '#/') location.hash = '#/monitor';
        route();
        return true;
      }
      if (!silent) setAuthStatus(data.message || 'Invalid token.', false);
      return false;
    } catch (e) {
      if (!silent) setAuthStatus('Request failed: ' + e, false);
      return false;
    } finally {
      if (!silent) btnDone(btn);
    }
  }

  function logout(msg) {
    sessionToken = '';
    clearSession();
    appEl.classList.add('hidden');
    authEl.classList.remove('hidden');
    document.getElementById('login').classList.remove('hidden');
    if (msg) setAuthStatus(msg, false);
  }

  // ── routing ───────────────────────────────────────────────────────────────
  const views = {
    monitor:  document.getElementById('view-monitor'),
    dashboard:document.getElementById('view-dashboard'),
    users:    document.getElementById('view-users'),
    problems: document.getElementById('view-problems'),
    database: document.getElementById('view-database'),
  };
  function route() {
    let name = (location.hash || '').replace(/^#\/?/, '');
    if (!views[name]) name = 'monitor';
    for (const key in views) views[key].classList.toggle('hidden', key !== name);
    document.querySelectorAll('.nav-item').forEach(el =>
      el.classList.toggle('active', el.dataset.route === name));
    setAppStatus('', true);
    if      (name === 'monitor')   loadSystemMonitor();
    else if (name === 'dashboard') loadDashboard();
    else if (name === 'users')     loadUsers();
    else if (name === 'problems')  loadProblems();
  }
  window.addEventListener('hashchange', route);
  document.querySelectorAll('.nav-item').forEach(el =>
    el.addEventListener('click', () => { location.hash = '#/' + el.dataset.route; }));

  // ── system monitor ────────────────────────────────────────────────────────
  async function loadSystemMonitor() {
    document.getElementById('systemStatus').innerHTML = '<p class="hint">Loading…</p>';
    document.getElementById('dbStats').innerHTML      = '<p class="hint">Loading…</p>';
    try {
      const { res: r1, data: d1 } = await postJson('/api/system-status', { token: sessionToken });
      if (r1.status === 401) { logout('Session expired. Please log in again.'); return; }
      if (r1.ok && d1.ok) {
        const s = d1.data;
        document.getElementById('systemStatus').innerHTML =
          '<div class="stat-grid">' +
          stat(s.database_connected ? '✓ Connected' : '✗ Disconnected', 'Database', s.database_connected ? 'ok' : 'err') +
          stat(s.active_ws_connections, 'WS Connections') +
          stat(formatUptime(s.uptime_seconds), 'Uptime') +
          stat(s.memory_usage_mb ? s.memory_usage_mb.toFixed(1) + ' MB' : 'N/A', 'Memory') +
          '</div>';
      } else {
        document.getElementById('systemStatus').innerHTML = '<p class="hint err">Failed to load system status.</p>';
      }
    } catch (e) {
      document.getElementById('systemStatus').innerHTML = '<p class="hint err">Request failed: ' + e + '</p>';
    }
    try {
      const { res: r2, data: d2 } = await postJson('/api/database-stats', { token: sessionToken });
      if (r2.ok && d2.ok) {
        const s = d2.data;
        document.getElementById('dbStats').innerHTML =
          '<div class="stat-grid">' +
          stat(s.total_users,       'Users') +
          stat(s.total_problems,    'Problems') +
          stat(s.total_submissions, 'Submissions') +
          stat(s.database_size_mb ? s.database_size_mb.toFixed(1) + ' MB' : 'N/A', 'DB Size') +
          '</div>';
      } else {
        document.getElementById('dbStats').innerHTML = '<p class="hint err">Failed to load database stats.</p>';
      }
    } catch (e) {
      document.getElementById('dbStats').innerHTML = '<p class="hint err">Request failed: ' + e + '</p>';
    }
  }
  function stat(value, label, cls) {
    return '<div class="stat-item"><div class="stat-value' + (cls ? ' ' + cls : '') + '">' + value + '</div><div class="stat-label">' + label + '</div></div>';
  }
  function formatUptime(s) {
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60);
    return h + 'h ' + m + 'm';
  }

  // ── dashboard ──────────────────────────────────────────────────────────────
  const CUM_COLOR   = '#0f6cbd';
  const DAILY_COLOR = '#ca5010';

  async function loadDashboard() {
    const container = document.getElementById('charts');
    container.innerHTML = '<p class="hint">Loading…</p>';
    if (typeof Highcharts === 'undefined') {
      container.innerHTML = '<p class="hint err">Highcharts failed to load (no network?).</p>';
      return;
    }
    try {
      const { res, data } = await postJson('/api/stats', { token: sessionToken });
      if (res.status === 401) { logout('Session expired. Please log in again.'); return; }
      if (!res.ok || !data.ok) { container.innerHTML = '<p class="hint err">Failed to load stats.</p>'; return; }
      container.innerHTML = '';
      (data.metrics || []).forEach(m => {
        const card = document.createElement('div');
        card.className = 'panel-card';
        const chartEl = document.createElement('div');
        chartEl.style.height = '320px';
        card.appendChild(chartEl); container.appendChild(card);
        renderDualChart(chartEl, m.title, m.points || []);
      });
    } catch (e) {
      container.innerHTML = '<p class="hint err">Request failed: ' + e + '</p>';
    }
  }

  function renderDualChart(container, title, points) {
    if (!points.length) { container.innerHTML = '<p class="hint">No data yet.</p>'; return; }
    Highcharts.chart(container, {
      title: { text: title + ' (last 30 days)', align: 'left',
               style: { fontSize: '16px', fontWeight: '600' } },
      credits: { enabled: false },
      xAxis: { categories: points.map(p => p.label), tickInterval: Math.ceil(points.length / 8) },
      yAxis: [
        { title: { text: 'Cumulative' }, min: 0 },
        { title: { text: 'Daily' }, opposite: true, min: 0 },
      ],
      tooltip: { shared: true },
      series: [
        { name: 'Cumulative', type: 'line', yAxis: 0, color: CUM_COLOR,
          data: points.map(p => p.cumulative) },
        { name: 'Daily', type: 'column', yAxis: 1, color: DAILY_COLOR,
          data: points.map(p => p.daily) },
      ],
    });
  }

  // ── user management ─────────────────────────────────────────────────────────────────
  // Schema: id, username, created_at, accepted (AC count), submission_count, discussion_count
  let currentUserPage = 1;
  async function loadUsers(page = 1) {
    currentUserPage = page;
    const search = document.getElementById('userSearch').value;
    const listEl = document.getElementById('userList');
    listEl.innerHTML = '<p class="hint">Loading…</p>';
    try {
      const { res, data } = await postJson('/api/users', { token: sessionToken, page, page_size: 20, search });
      if (res.status === 401) { logout('Session expired. Please log in again.'); return; }
      if (!res.ok || !data.ok) { listEl.innerHTML = '<p class="hint err">' + (data.message || 'Failed to load users.') + '</p>'; return; }
      const r = data.data;
      if (!r.users.length) { listEl.innerHTML = '<p class="hint">No users found.</p>'; return; }
      let html = '<table><thead><tr><th>ID</th><th>Username</th><th>Submissions</th><th>AC</th><th>Discussions</th><th>Registered</th><th>Actions</th></tr></thead><tbody>';
      r.users.forEach(u => {
        const reg = new Date(u.created_at).toLocaleDateString();
        html += '<tr>' +
          '<td>' + u.id + '</td>' +
          '<td>' + esc(u.username) + '</td>' +
          '<td>' + u.submission_count + '</td>' +
          '<td>' + u.accepted + '</td>' +
          '<td>' + u.discussion_count + '</td>' +
          '<td>' + reg + '</td>' +
          '<td><button class="danger" data-uid="' + u.id + '" data-uname="' + esc(u.username) + '" onclick="deleteUser(this)">Delete</button></td>' +
          '</tr>';
      });
      html += '</tbody></table><div class="pagination">';
      if (page > 1) html += '<button class="secondary" onclick="loadUsers(' + (page-1) + ')">← Prev</button>';
      html += '<span>Page ' + page + ' / ' + r.total_pages + ' (' + r.total + ' total)</span>';
      if (page < r.total_pages) html += '<button class="secondary" onclick="loadUsers(' + (page+1) + ')">Next →</button>';
      html += '</div>';
      listEl.innerHTML = html;
    } catch (e) {
      listEl.innerHTML = '<p class="hint err">Request failed: ' + e + '</p>';
    }
  }

  window.deleteUser = async function(btn) {
    const userId = btn.dataset.uid;
    const username = btn.dataset.uname;
    if (!confirm('Delete user "' + username + '"? This will also remove their submissions and discussions.\nThis cannot be undone.')) return;
    setAppStatus('Deleting…', true);
    try {
      const { res, data } = await postJson('/api/users/' + userId + '/delete', { token: sessionToken });
      if (res.status === 401) { logout('Session expired.'); return; }
      setAppStatus(res.ok && data.ok ? data.message : (data.message || 'Delete failed.'), res.ok && data.ok);
      if (res.ok && data.ok) loadUsers(currentUserPage);
    } catch (e) { setAppStatus('Request failed: ' + e, false); }
  };

  let userSearchTimer;
  document.getElementById('userSearch').addEventListener('input', () => {
    clearTimeout(userSearchTimer);
    userSearchTimer = setTimeout(() => loadUsers(1), 300);
  });

  // ── problem management ────────────────────────────────────────────────────────────
  // Schema: problem_number (PK), problem_name, difficulty (int 0-7), submission_count, accepted_count
  const DIFF_LABELS = {
    0: 'Unknown', 1: 'Beginner', 2: 'Primary', 3: 'Junior',
    4: 'Senior', 5: 'Advanced', 6: 'Hard', 7: 'Grand'
  };
  function diffLabel(d) { return DIFF_LABELS[d] || (d ? 'Lv.' + d : '—'); }

  let currentProblemPage = 1;
  async function loadProblems(page = 1) {
    currentProblemPage = page;
    const search = document.getElementById('problemSearch').value.trim() || undefined;
    const listEl = document.getElementById('problemList');
    listEl.innerHTML = '<p class="hint">Loading…</p>';
    try {
      const { res, data } = await postJson('/api/problems', { token: sessionToken, page, page_size: 20, search });
      if (res.status === 401) { logout('Session expired. Please log in again.'); return; }
      if (!res.ok || !data.ok) { listEl.innerHTML = '<p class="hint err">' + (data.message || 'Failed to load problems.') + '</p>'; return; }
      const r = data.data;
      if (!r.problems.length) { listEl.innerHTML = '<p class="hint">No problems found.</p>'; return; }
      let html = '<table><thead><tr><th>#</th><th>Name</th><th>Difficulty</th><th>Submissions</th><th>AC</th><th>Acceptance</th><th>Actions</th></tr></thead><tbody>';
      r.problems.forEach(p => {
        html += '<tr>' +
          '<td>' + p.problem_number + '</td>' +
          '<td>' + esc(p.problem_name) + '</td>' +
          '<td>' + diffLabel(p.difficulty) + '</td>' +
          '<td>' + p.submission_count + '</td>' +
          '<td>' + p.accepted_count + '</td>' +
          '<td>' + p.acceptance_rate.toFixed(1) + '%</td>' +
          '<td><button class="danger" data-pid="' + p.problem_number + '" data-pname="' + esc(p.problem_name) + '" onclick="deleteProblem(this)">Delete</button></td>' +
          '</tr>';
      });
      html += '</tbody></table><div class="pagination">';
      if (page > 1) html += '<button class="secondary" onclick="loadProblems(' + (page-1) + ')">← Prev</button>';
      html += '<span>Page ' + page + ' / ' + r.total_pages + ' (' + r.total + ' total)</span>';
      if (page < r.total_pages) html += '<button class="secondary" onclick="loadProblems(' + (page+1) + ')">Next →</button>';
      html += '</div>';
      listEl.innerHTML = html;
    } catch (e) {
      listEl.innerHTML = '<p class="hint err">Request failed: ' + e + '</p>';
    }
  }

  window.deleteProblem = async function(btn) {
    const problemId = btn.dataset.pid;
    const title = btn.dataset.pname;
    if (!confirm('Delete problem "' + title + '"? This will also remove all submissions for it.\nThis cannot be undone.')) return;
    setAppStatus('Deleting…', true);
    try {
      const { res, data } = await postJson('/api/problems/' + problemId + '/delete', { token: sessionToken });
      if (res.status === 401) { logout('Session expired.'); return; }
      setAppStatus(res.ok && data.ok ? data.message : (data.message || 'Delete failed.'), res.ok && data.ok);
      if (res.ok && data.ok) loadProblems(currentProblemPage);
    } catch (e) { setAppStatus('Request failed: ' + e, false); }
  };

  let problemSearchTimer;
  document.getElementById('problemSearch').addEventListener('input', () => {
    clearTimeout(problemSearchTimer);
    problemSearchTimer = setTimeout(() => loadProblems(1), 300);
  });

  // Create problem modal
  const createProblemModal = document.getElementById('createProblemModal');
  const createProblemBtn = document.getElementById('createProblemBtn');
  const cancelCreateProblem = document.getElementById('cancelCreateProblem');
  const submitCreateProblem = document.getElementById('submitCreateProblem');
  const createProblemStatus = document.getElementById('createProblemStatus');

  createProblemBtn.addEventListener('click', () => {
    createProblemModal.classList.remove('hidden');
    document.getElementById('newProblemNumber').value = '';
    document.getElementById('newProblemName').value = '';
    document.getElementById('newProblemDifficulty').value = '1';
    document.getElementById('newProblemStatement').value = '';
    document.getElementById('newTestcaseConfig').value = '';
    setStatus(createProblemStatus, '', true);
  });

  cancelCreateProblem.addEventListener('click', () => {
    createProblemModal.classList.add('hidden');
  });

  submitCreateProblem.addEventListener('click', async () => {
    const problemNumber = parseInt(document.getElementById('newProblemNumber').value);
    const problemName = document.getElementById('newProblemName').value.trim();
    const difficulty = parseInt(document.getElementById('newProblemDifficulty').value);
    const problemStatement = document.getElementById('newProblemStatement').value.trim();
    const testcaseConfig = document.getElementById('newTestcaseConfig').value.trim();

    // Validation
    if (!problemNumber || problemNumber < 1) {
      setStatus(createProblemStatus, 'Please enter a valid problem number', false);
      return;
    }
    if (!problemName) {
      setStatus(createProblemStatus, 'Please enter a problem name', false);
      return;
    }
    if (!problemStatement) {
      setStatus(createProblemStatus, 'Please enter a problem statement', false);
      return;
    }

    // Validate problem statement JSON format
    try {
      const parsed = JSON.parse(problemStatement);
      if (!Array.isArray(parsed)) {
        setStatus(createProblemStatus, 'Problem statement must be a JSON array', false);
        return;
      }
    } catch (e) {
      setStatus(createProblemStatus, 'Invalid problem statement JSON: ' + e.message, false);
      return;
    }

    // Validate testcase config JSON format if provided
    if (testcaseConfig) {
      try {
        JSON.parse(testcaseConfig);
      } catch (e) {
        setStatus(createProblemStatus, 'Invalid testcase config JSON: ' + e.message, false);
        return;
      }
    }

    btnBusy(submitCreateProblem, 'Creating…');
    setStatus(createProblemStatus, '', true);

    try {
      const { res, data } = await postJson('/api/problems/create', {
        token: sessionToken,
        problem_number: problemNumber,
        problem_name: problemName,
        difficulty: difficulty,
        problem_statement: problemStatement,
        testcase_config: testcaseConfig || null
      });

      if (res.status === 401) { logout('Session expired.'); return; }

      if (res.ok && data.ok) {
        setStatus(createProblemStatus, 'Problem created successfully!', true);
        setTimeout(() => {
          createProblemModal.classList.add('hidden');
          loadProblems(1);
        }, 1000);
      } else {
        setStatus(createProblemStatus, data.message || 'Failed to create problem', false);
      }
    } catch (e) {
      setStatus(createProblemStatus, 'Request failed: ' + e, false);
    } finally {
      btnDone(submitCreateProblem);
    }
  });

  // Close modal when clicking outside
  createProblemModal.addEventListener('click', (e) => {
    if (e.target === createProblemModal) {
      createProblemModal.classList.add('hidden');
    }
  });

  // ── database management ───────────────────────────────────────────────────
  const clearBtn = document.getElementById('clear');
  clearBtn.addEventListener('click', async () => {
    if (!confirm('Really wipe ALL data in the RsOJ database?\n\nThis truncates every OJ table and cannot be undone.')) return;
    btnBusy(clearBtn, 'Clearing…');
    setAppStatus('', true);
    try {
      const { res, data } = await postJson('/api/clear-database', { token: sessionToken });
      if (res.status === 401) { logout('Session expired. Please log in again.'); return; }
      setAppStatus(data.message || (res.ok ? 'Done.' : 'Failed.'), res.ok && data.ok);
    } catch (e) {
      setAppStatus('Request failed: ' + e, false);
    } finally {
      btnDone(clearBtn);
    }
  });

  // ── XSS helper ────────────────────────────────────────────────────────────
  function esc(str) {
    return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
  }

  init();
</script>

</body>
</html>
"#;
