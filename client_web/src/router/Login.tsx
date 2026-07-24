import { useEffect, useState, lazy } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { makeStyles, Input, Field, Button } from "@fluentui/react-components";
import { PersonRegular, PasswordRegular } from "@fluentui/react-icons";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import { useSelector } from "react-redux";

import * as globals from "../Globals.ts";

import "../css/style.css";
import { RootState } from "../store.ts";

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

export default function Login() {
    const { t } = useTranslation("login");
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const [loginUsernameFromInput, setLoginUsernameFromInput] = useState("");
    const [loginPasswordFromInput, setLoginPasswordFromInput] = useState("");

    const { loginSession, loginSessionState } = globals.useLoginSession(sendJsonMessage, lastJsonMessage);
    const [dialogLoggedInOpenState, setDialogLoggedInOpenState] = useState(false);
    const [dialogLoginFailureOpenState, setDialogLoginFailureOpenState] = useState(false);
    const [dialogLoginSuccessOpenState, setDialogLoginSuccessOpenState] = useState(false);
    const [dialogLoginSuccessOpenedState, setDialogLoginSuccessOpenedState] = useState(false);
    const [dialogSuppressedState, setDialogSuppressedState] = useState(false);

    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const navigate = useNavigate();

    const handleClickLogin = () => {
        setDialogSuppressedState(true);
        loginSession(loginUsernameFromInput, loginPasswordFromInput);
    };

    const handleNavigateBackward = () => {
        navigate(-1);
    };

    useEffect(() => {
        if (dialogSuppressedState === true) {
            setDialogLoggedInOpenState(false);
        }
        else {
            if (loginStatus.value === true && dialogLoginSuccessOpenState === false && dialogLoginSuccessOpenedState === false) setDialogLoggedInOpenState(true);
        }
    }, [loginStatus, dialogLoginSuccessOpenState, dialogSuppressedState, dialogLoginSuccessOpenedState]);

    useEffect(() => {
        if (loginSessionState !== undefined) {
            if (loginSessionState === true) {
                localStorage.setItem("loginUsername", loginUsernameFromInput);
                localStorage.setItem("loginPassword", loginPasswordFromInput);
                setDialogLoginSuccessOpenState(true);
                setDialogLoginSuccessOpenedState(true);
            }
            setDialogSuppressedState(false);
        }
    }, [loginUsernameFromInput, loginPasswordFromInput, loginSessionState]);

    useEffect(() => {
        if (loginSessionState !== undefined) {
            if (loginSessionState === false) setDialogLoginFailureOpenState(true);
        }
    }, [loginSessionState]);

    return (
        <>
            <div className={useStyles().root}>
                <form>
                    <Field label={t("field.username")} required>
                        <Input contentBefore={<PersonRegular />}
                            onChange={(props) => setLoginUsernameFromInput(props.target.value)} />
                    </Field>
                    <Field label={t("field.password")} required>
                        <Input contentBefore={<PasswordRegular />} type="password"
                            onChange={(props) => setLoginPasswordFromInput(props.target.value)} />
                    </Field>
                    <Button appearance="primary" style={{ marginTop: "1em" }}
                        onClick={handleClickLogin}>{t("button.login")}</Button>
                </form>
                <PopupDialog
                    open={dialogLoggedInOpenState}
                    setPopupDialogOpenState={setDialogLoggedInOpenState}
                    text={t("dialog.alreadyLoggedIn")}
                    onClose={handleNavigateBackward} />
                <PopupDialog
                    open={dialogLoginFailureOpenState}
                    setPopupDialogOpenState={setDialogLoginFailureOpenState}
                    text={t("dialog.loginFailed")} />
                <PopupDialog
                    open={dialogLoginSuccessOpenState}
                    setPopupDialogOpenState={setDialogLoginSuccessOpenState}
                    text={t("dialog.loginSuccess")}
                    onClose={handleNavigateBackward} />
            </div>
        </>
    );
}