import type { Page } from '@playwright/test';

export type WebSocketRequest = {
  type: string;
  content: Record<string, unknown> & { request_key: string };
};

export type WebSocketResponse = {
  type: string;
  content: Record<string, unknown>;
};

type Responder = (
  request: WebSocketRequest,
) => WebSocketResponse | undefined | Promise<WebSocketResponse | undefined>;

export async function mockWebSocket(
  page: Page,
  responder: Responder,
  requests: WebSocketRequest[] = [],
) {
  await page.routeWebSocket(/\/wsapi$/, (socket) => {
    socket.onMessage(async (message) => {
      const request = JSON.parse(message.toString()) as WebSocketRequest;
      requests.push(request);
      const response = await responder(request);
      if (response) socket.send(JSON.stringify(response));
    });
  });

  return requests;
}

export async function useEnglish(page: Page) {
  await page.addInitScript(() => localStorage.setItem('language', 'en'));
}

export async function useSavedSession(
  page: Page,
  username = 'alice',
  sessionToken = 'saved-session-token',
) {
  await page.addInitScript(
    ({ username: savedUsername, sessionToken: savedToken }) => {
      localStorage.setItem('language', 'en');
      localStorage.setItem('loginUsername', savedUsername);
      localStorage.setItem('loginSessionToken', savedToken);
      localStorage.setItem('loginPassword', 'must-be-removed');
      localStorage.setItem('loginStatus', 'true');
    },
    { username, sessionToken },
  );
}

export function authenticatedResponse(request: WebSocketRequest): WebSocketResponse | undefined {
  if (request.type === 'session_restore') {
    return {
      type: 'session_restored',
      content: {
        request_key: request.content.request_key,
        username: request.content.username,
        session_token: request.content.session_token,
      },
    };
  }

  if (request.type === 'notifications_unread_count') {
    return {
      type: 'notifications_unread_count',
      content: { request_key: request.content.request_key, unread_count: 2 },
    };
  }

  return undefined;
}
