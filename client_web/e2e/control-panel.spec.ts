import { expect, test, type Page, type Route } from '@playwright/test';

const corsHeaders = {
  'access-control-allow-origin': 'http://127.0.0.1:4173',
  'access-control-allow-headers': 'authorization,content-type',
  'access-control-allow-methods': 'GET,POST,PUT,OPTIONS',
  'content-type': 'application/json',
};

async function mockControlPanel(page: Page, seenAuthorization: string[]) {
  await page.route('http://localhost:9990/api/**', async (route: Route) => {
    const request = route.request();
    if (request.method() === 'OPTIONS') {
      await route.fulfill({ status: 204, headers: corsHeaders });
      return;
    }

    seenAuthorization.push(request.headers().authorization ?? '');
    const path = new URL(request.url()).pathname;
    if (path === '/api/verify-admin') {
      await route.fulfill({
        status: 200,
        headers: corsHeaders,
        body: JSON.stringify({
          ok: true,
          is_admin: true,
          username: 'alice',
          admin_role: 'problem_admin',
        }),
      });
      return;
    }
    if (path === '/api/stats') {
      await route.fulfill({ status: 200, headers: corsHeaders, body: JSON.stringify({ ok: true, metrics: [] }) });
      return;
    }
    if (path === '/api/problems') {
      await route.fulfill({
        status: 200,
        headers: corsHeaders,
        body: JSON.stringify({ ok: true, data: { problems: [], total: 0, total_pages: 1 } }),
      });
      return;
    }

    await route.fulfill({
      status: 403,
      headers: corsHeaders,
      body: JSON.stringify({ ok: false, message: 'forbidden' }),
    });
  });
}

test('unlocks a role-scoped control panel and sends the bearer token', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('language', 'en');
    localStorage.setItem('loginUsername', 'alice');
  });
  const seenAuthorization: string[] = [];
  await mockControlPanel(page, seenAuthorization);

  await page.goto('/control-panel');
  await page.getByPlaceholder('Control-panel admin token').fill('problem-secret');
  await page.getByRole('button', { name: 'Unlock' }).click();

  await expect(page.getByText('Problem Administrator')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Problems' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Users' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Settings' })).toHaveCount(0);

  await page.getByRole('button', { name: 'Problems' }).click();
  await expect(page.getByText('Problem Management', { exact: true })).toBeVisible();
  expect(seenAuthorization.length).toBeGreaterThan(0);
  expect(seenAuthorization.every((header) => header === 'Bearer problem-secret')).toBe(true);
  expect(await page.evaluate(() => sessionStorage.getItem('rsoj_control_panel_token'))).toBe('problem-secret');
});
