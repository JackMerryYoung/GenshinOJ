import { useEffect } from "react";
import { Outlet, useNavigate, useLocation } from "react-router-dom";

import { FluentProvider, webLightTheme } from "@fluentui/react-components";

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

export default function Root() {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);
    const dispatch = useDispatch();
    const {
        sendJsonMessage,
        lastJsonMessage,
        readyState
    } = useWebSocket("ws://" + location.host + "/wsapi", {
        share: true,
        shouldReconnect: () => true,
        reconnectAttempts: 10,
        reconnectInterval: 3000,
        onClose: () => {
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
            <NavBar />
            <div className="scroll-bar-wrap">
                <div className="react-router-outlet scroll-box" style={{ overflowY: "auto", height: "calc(100vh - 7.5em)" }}>
                    <Outlet context={{ sendJsonMessage, lastJsonMessage, readyState }} />
                </div>
                <div className="cover-bar" />
            </div>
            <Footer />
        </FluentProvider>
    );

}
