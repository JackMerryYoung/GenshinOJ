import { useEffect, useRef, useState } from "react";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { FluentProvider, webLightTheme, Toaster, useToastController, useId, Toast, ToastTitle, ToastBody } from "@fluentui/react-components";

import { useSelector, useDispatch } from "react-redux";

import useWebSocketModule from "react-use-websocket";

const { default: useWebSocket = useWebSocketModule } = useWebSocketModule as unknown as {
    default: typeof useWebSocketModule;
};

import NavBar from "./NavBar.tsx";

import { RootState } from "../store.ts";
import { loginReducer, logoutReducer, restoreLoginReducer } from "../../redux/loginStatusSlice.ts";
import { clearLoginUsernameReducer, modifyLoginUsernameReducer } from "../../redux/loginUsernameSlice.ts";
import { clearSessionTokenReducer, modifySessionTokenReducer } from "../../redux/sessionTokenSlice.ts";

import { WebSocketRequestBroker } from "../websocket/requestBroker.ts";

import "../css/style.css";
import Footer from "./Footer.tsx";

// Matches ws_server's AFK_CLOSE_CODE: the WebSocket close code the backend sends when it drops a
// connection for being idle past the 10-minute AFK timeout.
const AFK_CLOSE_CODE = 4000;

export default function Root() {
    const { t } = useTranslation("root");
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const dispatch = useDispatch();
    const [requestBroker] = useState(() => new WebSocketRequestBroker());
    const restoreAttemptedForConnection = useRef(false);

    const toasterId = useId("ws-toaster");
    const { dispatchToast } = useToastController(toasterId);
    const {
        sendJsonMessage,
        lastJsonMessage,
        readyState
    } = useWebSocket((location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/wsapi", {
        share: true,
        shouldReconnect: () => true,
        reconnectAttempts: 10,
        reconnectInterval: 3000,
        onClose: (event) => {
            requestBroker.rejectAll();
            restoreAttemptedForConnection.current = false;
            // A transport failure is not an explicit logout. Keep the persisted token and mark
            // authentication as indeterminate until the next connection restores it.
            dispatch(restoreLoginReducer());
            dispatch(clearLoginUsernameReducer());
            dispatch(clearSessionTokenReducer());

            // The backend sends this dedicated close code/reason only on the AFK idle timeout;
            // surface it so the user knows why they were disconnected (and logged out).
            if (event.code === AFK_CLOSE_CODE || event.reason === "afk_timeout") {
                dispatch(logoutReducer());
                localStorage.removeItem("loginUsername");
                localStorage.removeItem("loginSessionToken");
                localStorage.removeItem("loginPassword");
                dispatchToast(
                    <Toast>
                        <ToastTitle>{t("toast.disconnected.title")}</ToastTitle>
                        <ToastBody>{t("toast.disconnected.body")}</ToastBody>
                    </Toast>,
                    { intent: "warning", timeout: 8000 }
                );
            }
        },
    });

    useEffect(() => {
        requestBroker.setSender(sendJsonMessage);
        return () => requestBroker.setSender(undefined);
    }, [requestBroker, sendJsonMessage]);

    useEffect(() => {
        if (lastJsonMessage !== null) requestBroker.handleMessage(lastJsonMessage);
    }, [lastJsonMessage, requestBroker]);

    useEffect(() => {
        if (readyState !== 1) {
            restoreAttemptedForConnection.current = false;
            return;
        }
        if (restoreAttemptedForConnection.current) return;

        const username = localStorage.getItem("loginUsername");
        const storedSessionToken = localStorage.getItem("loginSessionToken");
        if (!username || !storedSessionToken) {
            dispatch(logoutReducer());
            return;
        }

        restoreAttemptedForConnection.current = true;
        requestBroker.request<{
            type: string;
            content: {
                request_key: string;
                username?: string;
                session_token?: string;
                reason?: string;
            };
        }>("session_restore", {
            username,
            session_token: storedSessionToken,
        }, { responseTypes: ["session_restored", "session_restore_failed"] }).then((response) => {
            if (response.type === "session_restored" && response.content.session_token) {
                dispatch(loginReducer());
                const restoredUsername = response.content.username ?? username;
                dispatch(modifyLoginUsernameReducer(restoredUsername));
                dispatch(modifySessionTokenReducer(response.content.session_token));
                localStorage.setItem("loginUsername", restoredUsername);
                localStorage.setItem("loginSessionToken", response.content.session_token);
                return;
            }

            throw new Error(response.content.reason ?? "The saved session is no longer valid.");
        }).catch((error) => {
            // The broker rejects in-flight requests when the transport closes. That is a
            // reconnect condition, not proof that the persisted token is invalid.
            if (error instanceof Error && error.message === "The WebSocket connection was closed.") return;
            dispatch(logoutReducer());
            dispatch(clearLoginUsernameReducer());
            dispatch(clearSessionTokenReducer());
            localStorage.removeItem("loginUsername");
            localStorage.removeItem("loginSessionToken");
            localStorage.removeItem("loginPassword");
        });
    }, [dispatch, readyState, requestBroker]);

    const navigate = useNavigate();
    const nowLocation = useLocation();

    useEffect(() => {
        if (nowLocation.pathname === "/") navigate("/home", { replace: true });
        if (nowLocation.pathname === "/user" && loginUsername.value) {
            navigate("/user/" + loginUsername.value, { replace: true });
        }
    }, [loginUsername.value, navigate, nowLocation.pathname]);

    useEffect(() => {
        // Passwords and stale client-side login flags must never be used to restore a session.
        localStorage.removeItem("loginPassword");
        localStorage.removeItem("loginStatus");
    }, []);

    return (
        <FluentProvider theme={webLightTheme}>
            <Toaster toasterId={toasterId} />
            <NavBar request={requestBroker.request} />
            <div className="scroll-bar-wrap">
                <div className="react-router-outlet scroll-box" style={{ overflowY: "auto", height: "calc(100vh - 7.5em)" }}>
                    <Outlet context={{ sendJsonMessage, lastJsonMessage, readyState, request: requestBroker.request }} />
                </div>
            </div>
            <Footer />
        </FluentProvider>
    );

}
