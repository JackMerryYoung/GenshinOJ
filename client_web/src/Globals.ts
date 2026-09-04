import { nanoid } from "nanoid";
import { useDispatch } from "react-redux";
import { logoutReducer } from "../redux/loginStatusSlice.ts";
import { clearLoginUsernameReducer } from "../redux/loginUsernameSlice.ts";
import { clearSessionTokenReducer } from "../redux/sessionTokenSlice.ts";
import type { WebSocketRequest } from "./websocket/requestBroker.ts";

export type { WebSocketRequest } from "./websocket/requestBroker.ts";

export function getProperty(obj: unknown, key: string) {
    if (typeof obj === "object" && obj !== null) {
        return (obj as { [k: string]: unknown })[key];
    } else {
        return undefined;
    }
}

function isObject(object: unknown) {
    return object != null && typeof object === 'object';
}

function deepEqual(object1: object, object2: object) {
    const keys1 = Object.keys(object1);
    const keys2 = Object.keys(object2);

    if (keys1.length !== keys2.length) {
        return false;
    }

    for (let index = 0; index < keys1.length; index++) {
        const val1 = getProperty(object1, keys1[index]), val2 = getProperty(object2, keys2[index]), areObjects = isObject(val1) && isObject(val2);
        if ((areObjects && !deepEqual(val1 as object, val2 as object)) ||
            !areObjects && val1 !== val2) {
            return false;
        }
    }

    return true;
}


export function compareArray(a: unknown[], b: unknown[]) {
    return (a.length === b.length) && (a.every((v, i) => (isObject(v) && isObject(b[i])) ? deepEqual(v as object, b[i] as object) : (v === b[i])));
}

export declare type WebSocketMessage = string | ArrayBuffer | SharedArrayBuffer | Blob | ArrayBufferView;
export declare type SendMessage = (message: WebSocketMessage, keep?: boolean) => void;
export declare type SendJsonMessage = <T = unknown>(jsonMessage: T, keep?: boolean) => void;
export declare type WebSocketLike = WebSocket | EventSource;

export declare type WebSocketHook<T = unknown, P = WebSocketEventMap['message'] | null> = {
    sendMessage: SendMessage;
    sendJsonMessage: SendJsonMessage;
    lastMessage: P;
    lastJsonMessage: T;
    readyState: ReadyState;
    getWebSocket: () => (WebSocketLike | null);
    request: WebSocketRequest;
};
export function useLogoutSession() {
    const dispatch = useDispatch();
    const handleQuit = () => {
        // Explicit logout always clears the persisted recovery token, even if the transport
        // already dropped and Redux no longer reports an active connection.
        dispatch(logoutReducer());
        dispatch(clearLoginUsernameReducer());
        dispatch(clearSessionTokenReducer());
        localStorage.removeItem("loginUsername");
        localStorage.removeItem("loginSessionToken");
        localStorage.removeItem("loginPassword");
        localStorage.removeItem("loginStatus");

    };

    return { handleQuit };
}

export function handleLogout(sendJsonMessage: SendJsonMessage, loginUsername: string, sessionToken: string) {
    sendJsonMessage({
        type: "quit",
        content: {
            username: loginUsername,
            session_token: sessionToken,
            request_key: nanoid(),
        }
    });
}
