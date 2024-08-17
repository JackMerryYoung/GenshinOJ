import { nanoid } from "nanoid";
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { RootState } from "./store.ts";
import { loginReducer, logoutReducer } from "../redux/loginStatusSlice.ts";
import { modifySessionTokenReducer } from "../redux/sessionTokenSlice.ts";
import { modifyLoginUsernameReducer } from "../redux/loginUsernameSlice.ts";
import { clearLoginUsernameReducer } from "../redux/loginUsernameSlice.ts";
import { clearSessionTokenReducer } from "../redux/sessionTokenSlice.ts";

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
};

export function useLoginSession(
    sendJsonMessage: SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [loginUsername, setLoginUsername] = useState("");
    const [loginSessionState, setLoginSessionState] = useState<boolean | undefined>(undefined);
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const dispatch = useDispatch();

    const loginSession = (_loginUsername: string, _loginPassword: string) => {
        setLoginSessionState(undefined);
        const _requestKey = nanoid();
        setLoginUsername(_loginUsername);
        sendJsonMessage({
            type: "login",
            content: {
                username: _loginUsername,
                password: _loginPassword,
                request_key: _requestKey
            }
        });

        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null)
            setWebsocketMessageHistory((previousMessageHistory) => previousMessageHistory.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, index) => {
            interface LoginSessionResult {
                type: string;
                content: {
                    request_key: string
                };
            }

            interface LoginSessionResultSuccess extends LoginSessionResult {
                type: string;
                content: {
                    request_key: string,
                    session_token: string
                };
            }

            function isLoginSession(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'request_key' in (x.content as object);
                }

                return false;
            }

            if (_message && isLoginSession(_message)) {
                if ((_message as LoginSessionResult).content.request_key === requestKey) {
                    if ((_message as LoginSessionResult).type === "quit") {
                        setLoginSessionState(false);
                        dispatch(logoutReducer());
                        localStorage.setItem("loginStatus", JSON.stringify(false));
                    }
                    if ((_message as LoginSessionResult).type === "session_token") {
                        setLoginSessionState(true);
                        dispatch(loginReducer());
                        dispatch(modifySessionTokenReducer((_message as LoginSessionResultSuccess).content.session_token));
                        dispatch(modifyLoginUsernameReducer(loginUsername));
                        localStorage.setItem("loginStatus", JSON.stringify(true));
                    }

                    delete _websocketMessageHistory[index];
                }
            }
        });

        if (!compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, loginUsername, requestKey]);

    return { loginSession, loginSessionState };
}

export function useLogoutSession() {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const dispatch = useDispatch();
    const handleQuit = () => {
        if (loginStatus.value === true) {
            dispatch(logoutReducer());
            dispatch(clearLoginUsernameReducer());
            dispatch(clearSessionTokenReducer());
            localStorage.removeItem("loginUsername");
            localStorage.removeItem("loginPassword");
            localStorage.setItem("loginStatus", JSON.stringify(false));
        }

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