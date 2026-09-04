import { nanoid } from "nanoid";

type SendJsonMessage = <T = unknown>(message: T, keep?: boolean) => void;

export interface WebSocketResponse<
    TContent extends { request_key: string } = { request_key: string }
> {
    type: string;
    content: TContent;
}

export interface WebSocketRequestOptions {
    signal?: AbortSignal;
    timeoutMs?: number;
    responseTypes?: readonly string[];
}

export type WebSocketRequest = <TResponse extends WebSocketResponse = WebSocketResponse>(
    type: string,
    content: object,
    options?: WebSocketRequestOptions,
) => Promise<TResponse>;

type PendingRequest = {
    reject: (reason: Error) => void;
    resolve: (response: WebSocketResponse) => void;
    responseTypes?: readonly string[];
    timeoutId: ReturnType<typeof setTimeout>;
    removeAbortListener?: () => void;
};

const DEFAULT_TIMEOUT_MS = 15_000;

function readResponse(message: unknown): WebSocketResponse | undefined {
    if (typeof message !== "object" || message === null || !("type" in message) || !("content" in message)) {
        return undefined;
    }

    const { type, content } = message;
    if (typeof type !== "string" || typeof content !== "object" || content === null || !("request_key" in content)) {
        return undefined;
    }
    if (typeof content.request_key !== "string") return undefined;

    return message as WebSocketResponse;
}

export class WebSocketRequestBroker {
    private readonly pendingRequests = new Map<string, PendingRequest>();
    private sender: SendJsonMessage | undefined;

    setSender(sender: SendJsonMessage | undefined) {
        this.sender = sender;
    }

    readonly request: WebSocketRequest = (type, content, options = {}) => {
        const sender = this.sender;
        if (!sender) {
            return Promise.reject(new Error("The WebSocket connection is not ready."));
        }

        const requestKey = nanoid();
        const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;

        return new Promise((resolve, reject) => {
            if (options.signal?.aborted) {
                reject(new Error("The WebSocket request was cancelled."));
                return;
            }

            const timeoutId = setTimeout(() => {
                this.rejectRequest(requestKey, new Error(`The WebSocket request timed out after ${timeoutMs} ms.`));
            }, timeoutMs);

            const pendingRequest: PendingRequest = {
                resolve: (response) => resolve(response as never),
                reject,
                responseTypes: options.responseTypes,
                timeoutId,
            };

            if (options.signal) {
                const handleAbort = () => {
                    this.rejectRequest(requestKey, new Error("The WebSocket request was cancelled."));
                };
                options.signal.addEventListener("abort", handleAbort, { once: true });
                pendingRequest.removeAbortListener = () => options.signal?.removeEventListener("abort", handleAbort);
            }

            this.pendingRequests.set(requestKey, pendingRequest);

            try {
                sender({
                    type,
                    content: { ...content, request_key: requestKey },
                });
            } catch (error) {
                this.rejectRequest(
                    requestKey,
                    error instanceof Error ? error : new Error("Failed to send the WebSocket request."),
                );
            }
        });
    };

    handleMessage(message: unknown) {
        const response = readResponse(message);
        if (!response) return false;

        const pendingRequest = this.pendingRequests.get(response.content.request_key);
        if (!pendingRequest) return false;
        if (pendingRequest.responseTypes && !pendingRequest.responseTypes.includes(response.type)) return false;

        this.pendingRequests.delete(response.content.request_key);
        this.cleanup(pendingRequest);
        pendingRequest.resolve(response);
        return true;
    }

    rejectAll(reason = "The WebSocket connection was closed.") {
        for (const requestKey of [...this.pendingRequests.keys()]) {
            this.rejectRequest(requestKey, new Error(reason));
        }
    }

    private rejectRequest(requestKey: string, error: Error) {
        const pendingRequest = this.pendingRequests.get(requestKey);
        if (!pendingRequest) return;

        this.pendingRequests.delete(requestKey);
        this.cleanup(pendingRequest);
        pendingRequest.reject(error);
    }

    private cleanup(pendingRequest: PendingRequest) {
        clearTimeout(pendingRequest.timeoutId);
        pendingRequest.removeAbortListener?.();
    }
}
