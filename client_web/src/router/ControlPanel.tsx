import { useState, useEffect } from 'react';
import Highcharts from 'highcharts';
import * as HighchartsReactModule from 'highcharts-react-official';
const HighchartsReact: any =
  (HighchartsReactModule as any).HighchartsReact ||
  (HighchartsReactModule as any).default ||
  HighchartsReactModule;
import ProblemEditor from './ProblemEditor';
import {
  FluentProvider,
  webLightTheme,
  makeStyles,
  shorthands,
  tokens,
  Button,
  Input,
  Card,
  Text,
  Title3,
  Title2,
  Spinner,
  Badge,
  Table,
  TableBody,
  TableCell,
  TableRow,
  TableHeader,
  TableHeaderCell,
} from '@fluentui/react-components';
import {
  DeleteRegular,
  EditRegular,
  AddRegular,
  SearchRegular,
  DatabaseRegular,
  PeopleRegular,
  DocumentRegular,
  ChartMultipleRegular,
  SettingsRegular,
} from '@fluentui/react-icons';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    minHeight: '100vh',
    backgroundColor: tokens.colorNeutralBackground2,
  },
  sidebar: {
    width: '240px',
    backgroundColor: tokens.colorNeutralBackground1,
    borderRight: `1px solid ${tokens.colorNeutralStroke1}`,
    ...shorthands.padding('20px', '0'),
  },
  brand: {
    ...shorthands.padding('12px', '24px'),
    marginBottom: '20px',
  },
  navItem: {
    width: '100%',
    justifyContent: 'flex-start',
    ...shorthands.padding('12px', '24px'),
    ...shorthands.borderLeft('3px', 'solid', 'transparent'),
    ...shorthands.border('none'),
    backgroundColor: 'transparent',
    cursor: 'pointer',
  },
  navItemActive: {
    borderLeftColor: tokens.colorBrandBackground,
    backgroundColor: tokens.colorNeutralBackground3,
    color: tokens.colorBrandForeground1,
  },
  content: {
    flex: 1,
    ...shorthands.padding('32px'),
    overflowY: 'auto',
  },
  authContainer: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '100vh',
    backgroundColor: tokens.colorNeutralBackground2,
  },
  authCard: {
    width: '400px',
    ...shorthands.padding('32px'),
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    ...shorthands.gap('16px'),
    marginBottom: '24px',
  },
  statCard: {
    ...shorthands.padding('20px'),
  },
  statValue: {
    fontSize: '28px',
    fontWeight: 600,
    color: tokens.colorBrandForeground1,
  },
  searchBox: {
    marginBottom: '16px',
  },
  actions: {
    display: 'flex',
    ...shorthands.gap('8px'),
  },
  pagination: {
    display: 'flex',
    ...shorthands.gap('8px'),
    justifyContent: 'center',
    marginTop: '16px',
    alignItems: 'center',
  },
  modalContent: {
    display: 'flex',
    flexDirection: 'column',
    ...shorthands.gap('16px'),
  },
});

const CONTROL_PANEL_URL = 'http://localhost:9990';

export function ControlPanel() {
  const styles = useStyles();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [activeTab, setActiveTab] = useState('dashboard');
  const [username, setUsername] = useState<string>('');
  const [adminRole, setAdminRole] = useState<string>('');

  useEffect(() => {
    checkAdminAccess();
  }, []);

  const checkAdminAccess = async () => {
    // Check if user is logged in via localStorage
    const username = localStorage.getItem('loginUsername');

    if (!username) {
      setError('You must be logged in to access the control panel');
      setLoading(false);
      return;
    }

    try {
      // Verify if the logged-in user is an admin
      const response = await fetch(`${CONTROL_PANEL_URL}/api/verify-admin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      });
      const data = await response.json();

      if (data.ok && data.is_admin) {
        setUsername(data.username);
        setAdminRole(data.admin_role);
        setIsAuthenticated(true);
      } else {
        setError('Access denied: Administrator privileges required');
      }
    } catch (err) {
      setError('Failed to verify admin access');
    } finally {
      setLoading(false);
    }
  };

  const hasPermission = (requiredPermissions: string[]): boolean => {
    if (adminRole === 'super_admin') return true;
    return requiredPermissions.includes(adminRole);
  };

  const getRoleName = (role: string): string => {
    const roleNames: { [key: string]: string } = {
      'problem_admin': 'Problem Administrator',
      'community_admin': 'Community Administrator',
      'super_admin': 'Super Administrator',
    };
    return roleNames[role] || 'Administrator';
  };

  if (loading) {
    return (
      <FluentProvider theme={webLightTheme}>
        <div className={styles.authContainer}>
          <Spinner label="Loading..." />
        </div>
      </FluentProvider>
    );
  }

  if (!isAuthenticated) {
    return (
      <FluentProvider theme={webLightTheme}>
        <div className={styles.authContainer}>
          <Card className={styles.authCard}>
            <Title2>RsOJ Control Panel</Title2>
            <Text style={{ marginTop: '16px', color: tokens.colorPaletteRedForeground1 }}>
              {error}
            </Text>
            {error.includes('logged in') && (
              <Button
                appearance="primary"
                onClick={() => window.location.href = '/login'}
                style={{ marginTop: '16px' }}
              >
                Go to Login
              </Button>
            )}
          </Card>
        </div>
      </FluentProvider>
    );
  }

  return (
    <FluentProvider theme={webLightTheme}>
      <div className={styles.container}>
        <div className={styles.sidebar}>
          <div className={styles.brand}>
            <Title3>Control Panel</Title3>
            <Text size={200} style={{ marginTop: '8px', color: tokens.colorNeutralForeground3 }}>
              {username}
            </Text>
            <Badge style={{ marginTop: '4px' }} color="success">
              {getRoleName(adminRole)}
            </Badge>
          </div>
          {hasPermission(['problem_admin', 'community_admin', 'super_admin']) && (
            <Button
              appearance="subtle"
              icon={<ChartMultipleRegular />}
              className={`${styles.navItem} ${activeTab === 'dashboard' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('dashboard')}
            >
              Dashboard
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<DatabaseRegular />}
              className={`${styles.navItem} ${activeTab === 'system' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('system')}
            >
              System Monitor
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<PeopleRegular />}
              className={`${styles.navItem} ${activeTab === 'users' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('users')}
            >
              Users
            </Button>
          )}
          {hasPermission(['problem_admin', 'super_admin']) && (
            <Button
              appearance="subtle"
              icon={<DocumentRegular />}
              className={`${styles.navItem} ${activeTab === 'problems' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('problems')}
            >
              Problems
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<SettingsRegular />}
              className={`${styles.navItem} ${activeTab === 'settings' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('settings')}
            >
              Settings
            </Button>
          )}
        </div>
        <div className={styles.content}>
          {activeTab === 'dashboard' && hasPermission(['problem_admin', 'community_admin', 'super_admin']) && <DashboardTab />}
          {activeTab === 'system' && hasPermission(['super_admin']) && <SystemMonitorTab />}
          {activeTab === 'users' && hasPermission(['super_admin']) && <UsersTab />}
          {activeTab === 'problems' && hasPermission(['problem_admin', 'super_admin']) && <ProblemsTab />}
          {activeTab === 'settings' && hasPermission(['super_admin']) && <SettingsTab />}
        </div>
      </div>
    </FluentProvider>
  );
}

function DashboardTab() {
  const [stats, setStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/stats`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        setStats(data.metrics);
      }
    } catch (err) {
      console.error('Failed to load stats', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <Spinner label="Loading statistics..." />;

  return (
    <>
      <Title2>Dashboard</Title2>
      {stats && stats.length > 0 && (
        <div style={{ marginTop: '24px' }}>
          {stats.map((metric: any) => {
            // Prepare data for Highcharts
            const categories = metric.points.map((p: any) => p.label);
            const dailyData = metric.points.map((p: any) => p.daily);
            const cumulativeData = metric.points.map((p: any) => p.cumulative);

            const chartOptions: Highcharts.Options = {
              chart: {
                type: 'column',
                height: 400,
              },
              title: {
                text: metric.title,
                align: 'left',
              },
              xAxis: {
                categories: categories,
                crosshair: true,
              },
              yAxis: [
                {
                  title: {
                    text: 'Daily Count',
                  },
                },
                {
                  title: {
                    text: 'Cumulative Total',
                  },
                  opposite: true,
                },
              ],
              tooltip: {
                shared: true,
              },
              legend: {
                align: 'right',
                verticalAlign: 'top',
              },
              plotOptions: {
                column: {
                  pointPadding: 0.2,
                  borderWidth: 0,
                },
              },
              series: [
                {
                  name: 'Daily',
                  type: 'column',
                  data: dailyData,
                  color: '#0078d4',
                  yAxis: 0,
                },
                {
                  name: 'Cumulative',
                  type: 'line',
                  data: cumulativeData,
                  color: '#107c10',
                  yAxis: 1,
                  marker: {
                    enabled: true,
                    radius: 4,
                  },
                },
              ],
              credits: {
                enabled: false,
              },
            };

            return (
              <Card key={metric.key} style={{ marginBottom: '24px', padding: '24px' }}>
                <HighchartsReact highcharts={Highcharts} options={chartOptions} />
              </Card>
            );
          })}
        </div>
      )}
      {(!stats || stats.length === 0) && (
        <Card style={{ marginTop: '24px', padding: '24px' }}>
          <Text>No statistics available yet. Use the system to generate data.</Text>
        </Card>
      )}
    </>
  );
}

function SystemMonitorTab() {
  const styles = useStyles();
  const [systemStatus, setSystemStatus] = useState<any>(null);
  const [databaseStats, setDatabaseStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadData = async () => {
    try {
      const [sysRes, dbRes] = await Promise.all([
        fetch(`${CONTROL_PANEL_URL}/api/system-status`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({}),
        }),
        fetch(`${CONTROL_PANEL_URL}/api/database-stats`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({}),
        }),
      ]);

      const sysData = await sysRes.json();
      const dbData = await dbRes.json();

      if (sysData.ok) setSystemStatus(sysData.data);
      if (dbData.ok) setDatabaseStats(dbData.data);
    } catch (err) {
      console.error('Failed to load system data', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) return <Spinner label="Loading system status..." />;

  return (
    <>
      <Title2>System Monitor</Title2>
      <div className={styles.statsGrid} style={{ marginTop: '24px' }}>
        {systemStatus && (
          <>
            <Card className={styles.statCard}>
              <Text size={200}>Database</Text>
              <div className={styles.statValue}>
                <Badge color={systemStatus.database_connected ? 'success' : 'danger'}>
                  {systemStatus.database_connected ? 'Connected' : 'Disconnected'}
                </Badge>
              </div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>Active WebSocket</Text>
              <div className={styles.statValue}>{systemStatus.active_ws_connections}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>Uptime</Text>
              <div className={styles.statValue}>
                {Math.floor(systemStatus.uptime_seconds / 3600)}h
              </div>
              <Text size={200}>{systemStatus.uptime_seconds}s</Text>
            </Card>
            {systemStatus.memory_usage_mb && (
              <Card className={styles.statCard}>
                <Text size={200}>Memory Usage</Text>
                <div className={styles.statValue}>
                  {systemStatus.memory_usage_mb.toFixed(1)} MB
                </div>
              </Card>
            )}
          </>
        )}
      </div>

      {databaseStats && (
        <Card style={{ marginTop: '24px', padding: '24px' }}>
          <Title3>Database Statistics</Title3>
          <div className={styles.statsGrid} style={{ marginTop: '16px' }}>
            <Card className={styles.statCard}>
              <Text size={200}>Total Users</Text>
              <div className={styles.statValue}>{databaseStats.total_users}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>Total Problems</Text>
              <div className={styles.statValue}>{databaseStats.total_problems}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>Total Submissions</Text>
              <div className={styles.statValue}>{databaseStats.total_submissions}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>Total Discussions</Text>
              <div className={styles.statValue}>{databaseStats.total_discussions}</div>
            </Card>
            {databaseStats.database_size_mb && (
              <Card className={styles.statCard}>
                <Text size={200}>Database Size</Text>
                <div className={styles.statValue}>
                  {databaseStats.database_size_mb.toFixed(2)} MB
                </div>
              </Card>
            )}
          </div>
        </Card>
      )}
    </>
  );
}

function UsersTab() {
  const styles = useStyles();
  const [users, setUsers] = useState<any[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadUsers();
  }, [page, search]);

  const loadUsers = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/users`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ page, page_size: 10, search: search || undefined }),
      });
      const data = await response.json();
      if (data.ok) {
        setUsers(data.data.users);
        setTotal(data.data.total);
        setTotalPages(data.data.total_pages);
      }
    } catch (err) {
      console.error('Failed to load users', err);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (userId: number) => {
    if (!confirm('Are you sure you want to delete this user? This action cannot be undone.')) {
      return;
    }

    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/users/${userId}/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        loadUsers();
      } else {
        alert(`Failed to delete user: ${data.message}`);
      }
    } catch (err) {
      alert('Failed to delete user');
    }
  };

  return (
    <>
      <Title2>User Management</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <div className={styles.searchBox}>
          <Input
            placeholder="Search users..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            contentBefore={<SearchRegular />}
          />
        </div>

        {loading ? (
          <Spinner label="Loading users..." />
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHeaderCell>ID</TableHeaderCell>
                  <TableHeaderCell>Username</TableHeaderCell>
                  <TableHeaderCell>Accepted</TableHeaderCell>
                  <TableHeaderCell>Submissions</TableHeaderCell>
                  <TableHeaderCell>Discussions</TableHeaderCell>
                  <TableHeaderCell>Created</TableHeaderCell>
                  <TableHeaderCell>Actions</TableHeaderCell>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell>{user.id}</TableCell>
                    <TableCell>{user.username}</TableCell>
                    <TableCell>{user.accepted}</TableCell>
                    <TableCell>{user.submission_count}</TableCell>
                    <TableCell>{user.discussion_count}</TableCell>
                    <TableCell>{new Date(user.created_at).toLocaleDateString()}</TableCell>
                    <TableCell>
                      <Button
                        appearance="subtle"
                        icon={<DeleteRegular />}
                        onClick={() => handleDelete(user.id)}
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            <div className={styles.pagination}>
              <Button disabled={page <= 1} onClick={() => setPage(page - 1)}>
                Previous
              </Button>
              <Text>
                Page {page} of {totalPages} ({total} total)
              </Text>
              <Button disabled={page >= totalPages} onClick={() => setPage(page + 1)}>
                Next
              </Button>
            </div>
          </>
        )}
      </Card>
    </>
  );
}

function ProblemsTab() {
  const styles = useStyles();
  const [problems, setProblems] = useState<any[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingProblem, setEditingProblem] = useState<number | undefined>(undefined);

  useEffect(() => {
    loadProblems();
  }, [page, search]);

  const loadProblems = async () => {
    setLoading(true);
    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/problems`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ page, page_size: 10, search: search || undefined }),
      });
      const data = await response.json();
      if (data.ok) {
        setProblems(data.data.problems);
        setTotal(data.data.total);
        setTotalPages(data.data.total_pages);
      }
    } catch (err) {
      console.error('Failed to load problems', err);
    } finally {
      setLoading(false);
    }
  };

  const handleEdit = (problemNumber: number) => {
    setEditingProblem(problemNumber);
    setShowCreateDialog(true);
  };

  const handleDelete = async (problemNumber: number) => {
    if (!confirm('Are you sure you want to delete this problem? All submissions and solutions will be deleted.')) {
      return;
    }

    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/problems/${problemNumber}/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        loadProblems();
      } else {
        alert(`Failed to delete problem: ${data.message}`);
      }
    } catch (err) {
      alert('Failed to delete problem');
    }
  };

  const getDifficultyBadge = (difficulty: number) => {
    const map: any = {
      0: { label: 'Unknown', color: '#999999' },
      1: { label: 'Beginner', color: '#DA3737' },
      2: { label: 'Primary', color: '#CC7700' },
      3: { label: 'Junior', color: '#FDDB10' },
      4: { label: 'Senior', color: '#3AAF00' },
      5: { label: 'Advanced', color: '#2744C2' },
      6: { label: 'Hard', color: '#773388' },
      7: { label: 'Grand', color: '#1C1C3C' },
    };
    const config = map[difficulty] || map[0];
    return (
      <Badge appearance="outline" style={{ color: config.color, borderColor: config.color }}>
        {config.label}
      </Badge>
    );
  };

  return (
    <>
      <Title2>Problem Management</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '16px' }}>
          <Input
            placeholder="Search problems..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            contentBefore={<SearchRegular />}
            style={{ flex: 1, marginRight: '16px' }}
          />
          <Button
            appearance="primary"
            icon={<AddRegular />}
            onClick={() => setShowCreateDialog(true)}
          >
            Create Problem
          </Button>
        </div>

        {loading ? (
          <Spinner label="Loading problems..." />
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHeaderCell>Number</TableHeaderCell>
                  <TableHeaderCell>Name</TableHeaderCell>
                  <TableHeaderCell>Difficulty</TableHeaderCell>
                  <TableHeaderCell>Submissions</TableHeaderCell>
                  <TableHeaderCell>Accepted</TableHeaderCell>
                  <TableHeaderCell>Accept Rate</TableHeaderCell>
                  <TableHeaderCell>Actions</TableHeaderCell>
                </TableRow>
              </TableHeader>
              <TableBody>
                {problems.map((problem) => (
                  <TableRow key={problem.problem_number}>
                    <TableCell>{problem.problem_number}</TableCell>
                    <TableCell>{problem.problem_name}</TableCell>
                    <TableCell>{getDifficultyBadge(problem.difficulty)}</TableCell>
                    <TableCell>{problem.submission_count}</TableCell>
                    <TableCell>{problem.accepted_count}</TableCell>
                    <TableCell>{problem.acceptance_rate.toFixed(1)}%</TableCell>
                    <TableCell>
                      <Button
                        appearance="subtle"
                        icon={<EditRegular />}
                        onClick={() => handleEdit(problem.problem_number)}
                        style={{ marginRight: '8px' }}
                      />
                      <Button
                        appearance="subtle"
                        icon={<DeleteRegular />}
                        onClick={() => handleDelete(problem.problem_number)}
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            <div className={styles.pagination}>
              <Button disabled={page <= 1} onClick={() => setPage(page - 1)}>
                Previous
              </Button>
              <Text>
                Page {page} of {totalPages} ({total} total)
              </Text>
              <Button disabled={page >= totalPages} onClick={() => setPage(page + 1)}>
                Next
              </Button>
            </div>
          </>
        )}
      </Card>

      <ProblemEditor
        isOpen={showCreateDialog}
        onClose={() => {
          setShowCreateDialog(false);
          setEditingProblem(undefined);
        }}
        problemNumber={editingProblem}
        onSuccess={() => {
          setShowCreateDialog(false);
          setEditingProblem(undefined);
          loadProblems();
        }}
      />
    </>
  );
}

function SettingsTab() {
  const [loading, setLoading] = useState(false);

  const handleClearDatabase = async () => {
    if (!confirm('Are you sure you want to clear the entire database? This action cannot be undone!')) {
      return;
    }

    if (!confirm('This will delete ALL data including users, problems, submissions, and discussions. Are you absolutely sure?')) {
      return;
    }

    setLoading(true);
    try {
      const response = await fetch(`${CONTROL_PANEL_URL}/api/clear-database`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        alert(data.message);
      } else {
        alert(`Failed: ${data.message}`);
      }
    } catch (err) {
      alert('Failed to clear database');
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Title2>Settings</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <Title3>Danger Zone</Title3>
        <Text style={{ marginTop: '12px', marginBottom: '16px' }}>
          These actions are irreversible and will affect all users.
        </Text>
        <Button appearance="primary" onClick={handleClearDatabase} disabled={loading}>
          Clear Database
        </Button>
      </Card>
    </>
  );
}

export default ControlPanel;
