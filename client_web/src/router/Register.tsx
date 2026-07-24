import { useEffect, useState, lazy } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { makeStyles, Input, Field, Button } from "@fluentui/react-components";
import { PersonRegular, PasswordRegular } from "@fluentui/react-icons";

import { useDispatch, useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { logoutReducer } from "../../redux/loginStatusSlice.ts";

import "../css/style.css";

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "column",
        rowGap: "4px",
        columnGap: "4px",
        maxWidth: "300px",
        padding: "4px 0 0 12px",
    },
});

function useRegisterSession(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [registerSessionState, setRegisterSessionState] = useState<boolean | undefined>(undefined);
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const dispatch = useDispatch();

    const registerSession = (registerUsername: string, registerPassword: string) => {
        const _requestKey = nanoid();
        setRegisterSessionState(undefined);
        sendJsonMessage({
            type: "register",
            content: {
                username: registerUsername,
                password: registerPassword,
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
        websocketMessageHistory.map((_message, index) => {
            interface RegisterSessionResult {
                type: string;
                content: {
                    reason: string,
                    request_key: string
                };
            }

            function isRegisterSession(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'request_key' in (x.content as object) && 'reason' in (x.content as object) && x.type === 'quit';
                }

                return false;
            }
            if (_message && isRegisterSession(_message)) {
                const message = _message as RegisterSessionResult;
                if (message.content.reason == "registration_success" && message.content.request_key === requestKey) {
                    dispatch(logoutReducer());
                    setRegisterSessionState(true);
                    delete _websocketMessageHistory[index];
                }

                if (message.type == "quit" && message.content.reason == "registration_failure" && message.content.request_key === requestKey) {
                    dispatch(logoutReducer());
                    setRegisterSessionState(false);
                    delete _websocketMessageHistory[index];
                }
            }
        });
    }, [websocketMessageHistory, requestKey]);

    return { registerSession, registerSessionState };
}

export default function Register() {
    const { t } = useTranslation("register");
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const [registerUsernameFromInput, setRegisterUsernameFromInput] = useState("");
    const [registerPasswordFromInput, setRegisterPasswordFromInput] = useState("");
    const [registerPasswordConfirmFromInput, setRegisterPasswordConfirmFromInput] = useState("");
    const [dialogLoggedInOpenState, setDialogLoggedInOpenState] = useState(false);
    const [dialogPasswordInputAndConfirmNotTheSameOpenState, setDialogPasswordInputAndConfirmNotTheSameOpenState] = useState(false);
    const [dialogRegisterFailureOpenState, setDialogRegisterFailureOpenState] = useState(false);
    const [dialogRegisterSuccessOpenState, setDialogRegisterSuccessOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const { registerSession, registerSessionState } = useRegisterSession(sendJsonMessage, lastJsonMessage);
    const navigate = useNavigate();

    const handleRegistrationClick = () => {
        if (registerPasswordFromInput != registerPasswordConfirmFromInput)
            setDialogPasswordInputAndConfirmNotTheSameOpenState(true);
        else
            registerSession(registerUsernameFromInput, registerPasswordFromInput);
    };

    useEffect(() => { if (loginStatus.value === true) setDialogLoggedInOpenState(true); }, [loginStatus]);

    useEffect(() => {
        if (registerSessionState !== undefined) {
            if (registerSessionState === true) setDialogRegisterSuccessOpenState(true);
            if (registerSessionState === false) setDialogRegisterFailureOpenState(true);
        }
    }, [registerSessionState]);

    return (
        <>
            <div className={useStyles().root}>
                <form>
                    <Field label={t("field.username")} required>
                        <Input contentBefore={<PersonRegular />}
                            onChange={(props) => setRegisterUsernameFromInput(props.target.value)} />
                    </Field>
                    <Field label={t("field.password")} required>
                        <Input contentBefore={<PasswordRegular />} type="password"
                            onChange={(props) => setRegisterPasswordFromInput(props.target.value)} />
                    </Field>
                    <Field label={t("field.confirmPassword")} required>
                        <Input contentBefore={<PasswordRegular />} type="password"
                            onChange={(props) => setRegisterPasswordConfirmFromInput(props.target.value)} />
                    </Field>
                    <Button appearance="primary" onClick={handleRegistrationClick} style={{ marginTop: "1em" }}>{t("action.register")}</Button>
                </form>
                <PopupDialog
                    open={dialogLoggedInOpenState && !dialogRegisterSuccessOpenState}
                    setPopupDialogOpenState={setDialogLoggedInOpenState}
                    text={t("dialog.alreadyLoggedIn")}
                    onClose={() => navigate("/home")} />
                <PopupDialog
                    open={dialogPasswordInputAndConfirmNotTheSameOpenState}
                    setPopupDialogOpenState={setDialogPasswordInputAndConfirmNotTheSameOpenState}
                    text={t("dialog.passwordMismatch")} />
                <PopupDialog
                    open={dialogRegisterFailureOpenState}
                    setPopupDialogOpenState={setDialogRegisterFailureOpenState}
                    text={t("dialog.registerFailure")} />
                <PopupDialog
                    open={dialogRegisterSuccessOpenState}
                    setPopupDialogOpenState={setDialogRegisterSuccessOpenState}
                    text={t("dialog.registerSuccess")}
                    onClose={() => navigate("/login")} />
            </div>
        </>
    );
}