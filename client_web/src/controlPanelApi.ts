const configuredUrl = (import.meta.env.VITE_CONTROL_PANEL_URL as string | undefined)
  ?.trim()
  .replace(/\/+$/, '');

export const CONTROL_PANEL_URL = configuredUrl || 'http://localhost:9990';

const TOKEN_KEY = 'rsoj_control_panel_token';
let inMemoryToken: string | null = null;

export function getControlPanelToken(): string | null {
  try {
    return sessionStorage.getItem(TOKEN_KEY) ?? inMemoryToken;
  } catch {
    return inMemoryToken;
  }
}

export function setControlPanelToken(token: string): void {
  inMemoryToken = token;
  try {
    sessionStorage.setItem(TOKEN_KEY, token);
  } catch {
    // Keep the token in memory for this page when browser storage is unavailable.
  }
}

export function clearControlPanelToken(): void {
  inMemoryToken = null;
  try {
    sessionStorage.removeItem(TOKEN_KEY);
  } catch {
    // The server remains the source of truth when storage is unavailable.
  }
}

export function controlPanelFetch(
  path: string,
  init: RequestInit = {},
  token: string | null = getControlPanelToken(),
): Promise<Response> {
  const headers = new Headers(init.headers);
  if (!headers.has('Content-Type') && init.body && !(init.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }
  if (token) headers.set('Authorization', `Bearer ${token}`);

  return fetch(`${CONTROL_PANEL_URL}${path}`, { ...init, headers });
}
