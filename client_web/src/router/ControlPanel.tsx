import { Activity, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import Highcharts from 'highcharts';
import * as HighchartsReactModule from 'highcharts-react-official';
const HighchartsReact: any =
  (HighchartsReactModule as any).HighchartsReact ||
  (HighchartsReactModule as any).default ||
  HighchartsReactModule;
import ProblemEditor from './ProblemEditor';
import LanguageSwitcher from './LanguageSwitcher.tsx';
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
import {
  controlPanelFetch,
  clearControlPanelToken,
  getControlPanelToken,
  setControlPanelToken,
} from '../controlPanelApi';

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

export function ControlPanel() {
  const { t } = useTranslation('controlPanel');
  const styles = useStyles();
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [notLoggedIn, setNotLoggedIn] = useState(false);
  const [activeTab, setActiveTab] = useState('dashboard');
  const [username, setUsername] = useState<string>('');
  const [adminRole, setAdminRole] = useState<string>('');
  const [tokenInput, setTokenInput] = useState('');

  useEffect(() => {
    const storedToken = getControlPanelToken();
    if (storedToken) checkAdminAccess(storedToken);
    else {
      const hasLoginIdentity = Boolean(localStorage.getItem('loginUsername'));
      setNotLoggedIn(!hasLoginIdentity);
      setError(hasLoginIdentity ? t('auth.tokenRequired') : t('auth.loginRequired'));
      setLoading(false);
    }
  }, []);

  const checkAdminAccess = async (token: string) => {
    // The control panel is intentionally outside Root's WebSocket layout, so Redux session
    // restoration is not mounted on this route. Use the persisted identity for its HTTP check.
    const username = localStorage.getItem('loginUsername');

    if (!username) {
      setError(t('auth.loginRequired'));
      setNotLoggedIn(true);
      setLoading(false);
      return;
    }

    try {
      // Verify if the logged-in user is an admin
      const response = await controlPanelFetch('/api/verify-admin', {
        method: 'POST',
        body: JSON.stringify({ username }),
      }, token);
      const data = await response.json();

      if (response.ok && data.ok && data.is_admin) {
        setUsername(data.username);
        setAdminRole(data.admin_role);
        setIsAuthenticated(true);
      } else {
        clearControlPanelToken();
        setError(
          response.status === 503
            ? t('auth.notConfigured')
            : t('auth.accessDenied'),
        );
      }
    } catch (err) {
      setError(t('auth.verifyFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleAdminLogin = async () => {
    const token = tokenInput.trim();
    if (!token) {
      setError(t('auth.tokenRequired'));
      return;
    }
    setLoading(true);
    setError('');
    setControlPanelToken(token);
    await checkAdminAccess(token);
    setTokenInput('');
  };

  const hasPermission = (requiredPermissions: string[]): boolean => {
    if (adminRole === 'super_admin') return true;
    return requiredPermissions.includes(adminRole);
  };

  const getRoleName = (role: string): string => {
    const roleNames: { [key: string]: string } = {
      'problem_admin': t('roles.problemAdmin'),
      'community_admin': t('roles.communityAdmin'),
      'super_admin': t('roles.superAdmin'),
    };
    return roleNames[role] || t('roles.default');
  };

  if (loading) {
    return (
      <FluentProvider theme={webLightTheme}>
        <div className={styles.authContainer}>
          <Spinner label={t('auth.loading')} />
        </div>
      </FluentProvider>
    );
  }

  if (!isAuthenticated) {
    return (
      <FluentProvider theme={webLightTheme}>
        <div className={styles.authContainer}>
          <Card className={styles.authCard}>
            <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: '8px' }}>
              <LanguageSwitcher />
            </div>
            <Title2>{t('auth.title')}</Title2>
            <Text style={{ marginTop: '16px', color: tokens.colorPaletteRedForeground1 }}>
              {error}
            </Text>
            {!notLoggedIn && (
              <>
                <Input
                  type="password"
                  value={tokenInput}
                  onChange={(event) => setTokenInput(event.target.value)}
                  placeholder={t('auth.tokenPlaceholder')}
                  style={{ marginTop: '16px' }}
                />
                <Button
                  appearance="primary"
                  onClick={handleAdminLogin}
                  style={{ marginTop: '12px' }}
                >
                  {t('auth.unlock')}
                </Button>
              </>
            )}
            {notLoggedIn && (
              <Button
                appearance="primary"
                onClick={() => window.location.href = '/login'}
                style={{ marginTop: '16px' }}
              >
                {t('auth.goToLogin')}
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
            <Title3>{t('sidebar.title')}</Title3>
            <Text size={200} style={{ marginTop: '8px', color: tokens.colorNeutralForeground3 }}>
              {username}
            </Text>
            <Badge style={{ marginTop: '4px' }} color="success">
              {getRoleName(adminRole)}
            </Badge>
            <div style={{ marginTop: '12px' }}>
              <LanguageSwitcher />
            </div>
          </div>
          {hasPermission(['problem_admin', 'community_admin', 'super_admin']) && (
            <Button
              appearance="subtle"
              icon={<ChartMultipleRegular />}
              className={`${styles.navItem} ${activeTab === 'dashboard' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('dashboard')}
            >
              {t('sidebar.dashboard')}
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<DatabaseRegular />}
              className={`${styles.navItem} ${activeTab === 'system' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('system')}
            >
              {t('sidebar.systemMonitor')}
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<PeopleRegular />}
              className={`${styles.navItem} ${activeTab === 'users' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('users')}
            >
              {t('sidebar.users')}
            </Button>
          )}
          {hasPermission(['problem_admin', 'super_admin']) && (
            <Button
              appearance="subtle"
              icon={<DocumentRegular />}
              className={`${styles.navItem} ${activeTab === 'problems' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('problems')}
            >
              {t('sidebar.problems')}
            </Button>
          )}
          {hasPermission(['super_admin']) && (
            <Button
              appearance="subtle"
              icon={<SettingsRegular />}
              className={`${styles.navItem} ${activeTab === 'settings' ? styles.navItemActive : ''}`}
              onClick={() => setActiveTab('settings')}
            >
              {t('sidebar.settings')}
            </Button>
          )}
        </div>
        <div className={styles.content}>
          {hasPermission(['problem_admin', 'community_admin', 'super_admin']) && (
            <Activity mode={activeTab === 'dashboard' ? 'visible' : 'hidden'}>
              <DashboardTab />
            </Activity>
          )}
          {hasPermission(['super_admin']) && (
            <Activity mode={activeTab === 'system' ? 'visible' : 'hidden'}>
              <SystemMonitorTab />
            </Activity>
          )}
          {hasPermission(['super_admin']) && (
            <Activity mode={activeTab === 'users' ? 'visible' : 'hidden'}>
              <UsersTab />
            </Activity>
          )}
          {hasPermission(['problem_admin', 'super_admin']) && (
            <Activity mode={activeTab === 'problems' ? 'visible' : 'hidden'}>
              <ProblemsTab />
            </Activity>
          )}
          {hasPermission(['super_admin']) && (
            <Activity mode={activeTab === 'settings' ? 'visible' : 'hidden'}>
              <SettingsTab />
            </Activity>
          )}
        </div>
      </div>
    </FluentProvider>
  );
}

function DashboardTab() {
  const { t } = useTranslation('controlPanel');
  const [stats, setStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStats();
  }, []);

  const loadStats = async () => {
    try {
      const response = await controlPanelFetch('/api/stats', {
        method: 'POST',
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

  if (loading) return <Spinner label={t('dashboard.loading')} />;

  return (
    <>
      <Title2>{t('dashboard.title')}</Title2>
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
                    text: t('dashboard.dailyCount'),
                  },
                },
                {
                  title: {
                    text: t('dashboard.cumulativeTotal'),
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
                  name: t('dashboard.daily'),
                  type: 'column',
                  data: dailyData,
                  color: '#0078d4',
                  yAxis: 0,
                },
                {
                  name: t('dashboard.cumulative'),
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
          <Text>{t('dashboard.noData')}</Text>
        </Card>
      )}
    </>
  );
}

function SystemMonitorTab() {
  const { t } = useTranslation('controlPanel');
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
        controlPanelFetch('/api/system-status', {
          method: 'POST',
          body: JSON.stringify({}),
        }),
        controlPanelFetch('/api/database-stats', {
          method: 'POST',
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

  if (loading) return <Spinner label={t('systemMonitor.loading')} />;

  return (
    <>
      <Title2>{t('systemMonitor.title')}</Title2>
      <div className={styles.statsGrid} style={{ marginTop: '24px' }}>
        {systemStatus && (
          <>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.database')}</Text>
              <div className={styles.statValue}>
                <Badge color={systemStatus.database_connected ? 'success' : 'danger'}>
                  {systemStatus.database_connected ? t('systemMonitor.connected') : t('systemMonitor.disconnected')}
                </Badge>
              </div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.activeWebSocket')}</Text>
              <div className={styles.statValue}>{systemStatus.active_ws_connections}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.uptime')}</Text>
              <div className={styles.statValue}>
                {Math.floor(systemStatus.uptime_seconds / 3600)}h
              </div>
              <Text size={200}>{systemStatus.uptime_seconds}s</Text>
            </Card>
            {systemStatus.memory_usage_mb && (
              <Card className={styles.statCard}>
                <Text size={200}>{t('systemMonitor.memoryUsage')}</Text>
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
          <Title3>{t('systemMonitor.databaseStatistics')}</Title3>
          <div className={styles.statsGrid} style={{ marginTop: '16px' }}>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.totalUsers')}</Text>
              <div className={styles.statValue}>{databaseStats.total_users}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.totalProblems')}</Text>
              <div className={styles.statValue}>{databaseStats.total_problems}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.totalSubmissions')}</Text>
              <div className={styles.statValue}>{databaseStats.total_submissions}</div>
            </Card>
            <Card className={styles.statCard}>
              <Text size={200}>{t('systemMonitor.totalDiscussions')}</Text>
              <div className={styles.statValue}>{databaseStats.total_discussions}</div>
            </Card>
            {databaseStats.database_size_mb && (
              <Card className={styles.statCard}>
                <Text size={200}>{t('systemMonitor.databaseSize')}</Text>
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
  const { t } = useTranslation('controlPanel');
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
      const response = await controlPanelFetch('/api/users', {
        method: 'POST',
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
    if (!confirm(t('users.confirmDelete'))) {
      return;
    }

    try {
      const response = await controlPanelFetch(`/api/users/${userId}/delete`, {
        method: 'POST',
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        loadUsers();
      } else {
        alert(t('users.deleteFailed', { message: data.message }));
      }
    } catch (err) {
      alert(t('users.deleteFailedGeneric'));
    }
  };

  return (
    <>
      <Title2>{t('users.title')}</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <div className={styles.searchBox}>
          <Input
            placeholder={t('users.searchPlaceholder')}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            contentBefore={<SearchRegular />}
          />
        </div>

        {loading ? (
          <Spinner label={t('users.loading')} />
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHeaderCell>{t('users.table.id')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.username')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.accepted')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.submissions')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.discussions')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.created')}</TableHeaderCell>
                  <TableHeaderCell>{t('users.table.actions')}</TableHeaderCell>
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
                {t('users.pagination.previous')}
              </Button>
              <Text>
                {t('users.pagination.pageInfo', { page, totalPages, total })}
              </Text>
              <Button disabled={page >= totalPages} onClick={() => setPage(page + 1)}>
                {t('users.pagination.next')}
              </Button>
            </div>
          </>
        )}
      </Card>
    </>
  );
}

function ProblemsTab() {
  const { t } = useTranslation('controlPanel');
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
      const response = await controlPanelFetch('/api/problems', {
        method: 'POST',
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
    if (!confirm(t('problems.confirmDelete'))) {
      return;
    }

    try {
      const response = await controlPanelFetch(`/api/problems/${problemNumber}/delete`, {
        method: 'POST',
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        loadProblems();
      } else {
        alert(t('problems.deleteFailed', { message: data.message }));
      }
    } catch (err) {
      alert(t('problems.deleteFailedGeneric'));
    }
  };

  const getDifficultyBadge = (difficulty: number) => {
    const map: any = {
      0: { label: t('problems.difficulty.unknown'), color: '#999999' },
      1: { label: t('problems.difficulty.beginner'), color: '#DA3737' },
      2: { label: t('problems.difficulty.primary'), color: '#CC7700' },
      3: { label: t('problems.difficulty.junior'), color: '#FDDB10' },
      4: { label: t('problems.difficulty.senior'), color: '#3AAF00' },
      5: { label: t('problems.difficulty.advanced'), color: '#2744C2' },
      6: { label: t('problems.difficulty.hard'), color: '#773388' },
      7: { label: t('problems.difficulty.grand'), color: '#1C1C3C' },
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
      <Title2>{t('problems.title')}</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '16px' }}>
          <Input
            placeholder={t('problems.searchPlaceholder')}
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
            {t('problems.createProblem')}
          </Button>
        </div>

        {loading ? (
          <Spinner label={t('problems.loading')} />
        ) : (
          <>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHeaderCell>{t('problems.table.number')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.name')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.difficulty')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.submissions')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.accepted')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.acceptRate')}</TableHeaderCell>
                  <TableHeaderCell>{t('problems.table.actions')}</TableHeaderCell>
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
                {t('problems.pagination.previous')}
              </Button>
              <Text>
                {t('problems.pagination.pageInfo', { page, totalPages, total })}
              </Text>
              <Button disabled={page >= totalPages} onClick={() => setPage(page + 1)}>
                {t('problems.pagination.next')}
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
  const { t } = useTranslation('controlPanel');
  const [loading, setLoading] = useState(false);

  const handleClearDatabase = async () => {
    if (!confirm(t('settings.confirmClear1'))) {
      return;
    }

    if (!confirm(t('settings.confirmClear2'))) {
      return;
    }

    setLoading(true);
    try {
      const response = await controlPanelFetch('/api/clear-database', {
        method: 'POST',
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (data.ok) {
        alert(data.message);
      } else {
        alert(t('settings.clearFailed', { message: data.message }));
      }
    } catch (err) {
      alert(t('settings.clearFailedGeneric'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Title2>{t('settings.title')}</Title2>
      <Card style={{ marginTop: '24px', padding: '24px' }}>
        <Title3>{t('settings.dangerZone')}</Title3>
        <Text style={{ marginTop: '12px', marginBottom: '16px' }}>
          {t('settings.dangerZoneDescription')}
        </Text>
        <Button appearance="primary" onClick={handleClearDatabase} disabled={loading}>
          {t('settings.clearDatabase')}
        </Button>
      </Card>
    </>
  );
}

export default ControlPanel;
