import { expect, test } from '@playwright/test';
import {
  authenticatedResponse,
  mockWebSocket,
  useEnglish,
  useSavedSession,
  type WebSocketRequest,
} from './support/mockBackend';

test('logs in through the request broker and persists only the session token', async ({ page }) => {
  await useEnglish(page);
  const requests: WebSocketRequest[] = [];
  await mockWebSocket(page, (request) => {
    if (request.type === 'login') {
      return {
        type: 'session_token',
        content: { request_key: request.content.request_key, session_token: 'fresh-token' },
      };
    }
    return authenticatedResponse(request);
  }, requests);

  await page.goto('/login');
  await page.getByLabel('Login Username').fill('alice');
  await page.getByLabel('Login Password').fill('correct horse battery staple');
  await page.getByRole('button', { name: 'Login' }).click();

  await expect(page.getByText('Login successfully.')).toBeVisible();
  await expect(page.getByRole('tab', { name: 'Sign out' })).toBeVisible();
  await expect.poll(() => page.evaluate(() => localStorage.getItem('loginSessionToken')))
    .toBe('fresh-token');
  expect(await page.evaluate(() => localStorage.getItem('loginPassword'))).toBeNull();

  const loginRequest = requests.find((request) => request.type === 'login');
  expect(loginRequest?.content).toMatchObject({ username: 'alice' });
  expect(loginRequest?.content.request_key).toEqual(expect.any(String));
});

test('restores a saved session before exposing authenticated navigation', async ({ page }) => {
  await useSavedSession(page);
  const requests: WebSocketRequest[] = [];
  await mockWebSocket(page, authenticatedResponse, requests);

  await page.goto('/home');

  await expect(page.getByRole('tab', { name: 'Sign out' })).toBeVisible();
  await expect(page.getByRole('tab', { name: /Notification/ })).toContainText('2');
  expect(requests.find((request) => request.type === 'session_restore')?.content).toMatchObject({
    username: 'alice',
    session_token: 'saved-session-token',
  });
  expect(await page.evaluate(() => localStorage.getItem('loginPassword'))).toBeNull();
  expect(await page.evaluate(() => localStorage.getItem('loginStatus'))).toBeNull();
});
