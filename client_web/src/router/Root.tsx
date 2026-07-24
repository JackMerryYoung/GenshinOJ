import { useEffect } from "react";
import { Outlet, useNavigate, useLocation } from "react-router-dom";

import { FluentProvider, webLightTheme, Toaster, useToastController, useId, Toast, ToastTitle, ToastBody } from "@fluentui/react-components";

import { useSelector, useDispatch } from "react-redux";

import useWebSocketModule from "react-use-websocket";

const { default: useWebSocket = useWebSocketModule } = useWebSocketModule as unknown as {
    default: typeof useWebSocketModule;
};

import { nanoid } from "nanoid";

import NavBar from "./NavBar.tsx";

import { RootState } from "../store.ts";
import { logoutReducer } from "../../redux/loginStatusSlice.ts";

import * as globals from "../Globals.ts";

import "../css/style.css";
import Footer from "./Footer.tsx";

// Matches ws_server's AFK_CLOSE_CODE: the WebSocket close code the backend sends when it drops a
// connection for being idle past the 10-minute AFK timeout.
const AFK_CLOSE_CODE = 4000;

export default function Root() {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);
    const dispatch = useDispatch();

    const toasterId = useId("ws-toaster");
    const { dispatchToast } = useToastController(toasterId);
    const {
        sendJsonMessage,
        lastJsonMessage,
        readyState
    } = useWebSocket("ws://" + location.host + "/wsapi", {
        share: true,
        shouldReconnect: () => true,
        reconnectAttempts: 10,
        reconnectInterval: 3000,
        onClose: (event) => {
            if (loginStatus.value === true) {
                sendJsonMessage({
                    type: "quit",
                    content: {
                        username: loginUsername.value,
                        session_token: sessionToken.value,
                        request_key: nanoid()
                    }
                });

                dispatch(logoutReducer());
            }

            // The backend sends this dedicated close code/reason only on the AFK idle timeout;
            // surface it so the user knows why they were disconnected (and logged out).
            if (event.code === AFK_CLOSE_CODE || event.reason === "afk_timeout") {
                dispatchToast(
                    <Toast>
                        <ToastTitle>Disconnected</ToastTitle>
                        <ToastBody>You were disconnected after 10 minutes of inactivity. Please sign in again.</ToastBody>
                    </Toast>,
                    { intent: "warning", timeout: 8000 }
                );
            }
        },
    });

    const beforeunload = (ev: Event) => {
        if (ev) {
            if (loginStatus.value === true) {
                sendJsonMessage({
                    type: "quit",
                    content: {
                        username: loginUsername.value,
                        session_token: sessionToken.value,
                        request_key: nanoid()
                    }
                });

                sendJsonMessage({
                    type: "close_connection",
                    content: {
                        request_key: nanoid()
                    }
                });

                dispatch(logoutReducer());
            }
        }
    };

    useEffect(() => {
        if (loginUsername.value !== undefined && loginUsername.value !== "") window.addEventListener('beforeunload', beforeunload);
        return () => window.removeEventListener('beforeunload', beforeunload);
    }, [loginUsername]);

    const navigate = useNavigate();
    const nowLocation = useLocation();

    useEffect(() => {
        if (nowLocation.pathname == "/") navigate("/home");
        if (nowLocation.pathname == "/user") navigate("/user/" + loginUsername.value);
    }, [nowLocation]);

    const { loginSession } = globals.useLoginSession(sendJsonMessage, lastJsonMessage);

    useEffect(() => {
        const loginUsernameFromLocalStorage = localStorage.getItem("loginUsername");
        const loginPasswordFromLocalStorage = localStorage.getItem("loginPassword");
        if (loginUsernameFromLocalStorage !== null && loginPasswordFromLocalStorage !== null) {
            loginSession(loginUsernameFromLocalStorage, loginPasswordFromLocalStorage);
        }
    }, []);

    return (
        <FluentProvider theme={webLightTheme}>
            <Toaster toasterId={toasterId} />
            <NavBar sendJsonMessage={sendJsonMessage} lastJsonMessage={lastJsonMessage} />
            <div className="scroll-bar-wrap">
                <div className="react-router-outlet scroll-box" style={{ overflowY: "auto", height: "calc(100vh - 7.5em)" }}>
                    <Outlet context={{ sendJsonMessage, lastJsonMessage, readyState }} />
                </div>
            </div>
            <Footer />
        </FluentProvider>
    );

}
