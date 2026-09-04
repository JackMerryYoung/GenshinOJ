import { expect, test } from '@playwright/test';
import { mockWebSocket, useEnglish } from './support/mockBackend';

test('switches visible copy and the route title without navigating', async ({ page }) => {
  await useEnglish(page);
  await mockWebSocket(page, () => undefined);
  await page.goto('/login');

  await expect(page).toHaveTitle('Sign in');
  await expect(page.getByLabel('Login Username')).toBeVisible();

  await page.getByRole('combobox').click();
  await page.getByRole('option', { name: '简体中文' }).click();

  await expect(page).toHaveTitle('登录');
  await expect(page.getByLabel('登录用户名')).toBeVisible();
  await expect(page.getByRole('tab', { name: '注册' })).toBeVisible();
});
