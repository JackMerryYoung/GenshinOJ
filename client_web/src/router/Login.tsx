import { lazy, useActionState, useEffect, useState } from "react";
import { useFormStatus } from "react-dom";
import { useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button, Field, Input, makeStyles } from "@fluentui/react-components";
import { PasswordRegular, PersonRegular } from "@fluentui/react-icons";
import { useDispatch, useSelector } from "react-redux";

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { loginReducer, logoutReducer } from "../../redux/loginStatusSlice.ts";
import { modifyLoginUsernameReducer } from "../../redux/loginUsernameSlice.ts";
import { modifySessionTokenReducer } from "../../redux/sessionTokenSlice.ts";
import "../css/style.css";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

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

type LoginActionState = {
    status: "idle" | "success" | "failure" | "validation";
    message?: string;
};

function LoginSubmitButton() {
    const { t } = useTranslation("login");
    const { pending } = useFormStatus();

    return (
        <Button appearance="primary" type="submit" disabled={pending} style={{ marginTop: "1em" }}>
            {pending ? t("button.pending") : t("button.login")}
        </Button>
    );
}

export default function Login() {
    const { t } = useTranslation("login");
    const { request } = useOutletContext<globals.WebSocketHook>();
    const dispatch = useDispatch();
    const [dialogLoggedInOpenState, setDialogLoggedInOpenState] = useState(false);
    const [dialogLoginFailureOpenState, setDialogLoginFailureOpenState] = useState(false);
    const [dialogLoginSuccessOpenState, setDialogLoginSuccessOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const navigate = useNavigate();

    const [actionState, loginAction, pending] = useActionState<LoginActionState, FormData>(
        async (_previousState, formData) => {
            const username = String(formData.get("username") ?? "").trim();
            const password = String(formData.get("password") ?? "");
            if (!username || !password) {
                return { status: "validation", message: t("validation.required") };
            }

            try {
                const response = await request<{
                    type: string;
                    content: { request_key: string; session_token?: string; reason?: string };
                }>("login", { username, password }, { responseTypes: ["session_token", "quit"] });

                if (response.type !== "session_token" || !response.content.session_token) {
                    dispatch(logoutReducer());
                    return { status: "failure", message: t("dialog.loginFailed") };
                }

                dispatch(loginReducer());
                dispatch(modifyLoginUsernameReducer(username));
                dispatch(modifySessionTokenReducer(response.content.session_token));
                localStorage.setItem("loginUsername", username);
                localStorage.setItem("loginSessionToken", response.content.session_token);
                return { status: "success" };
            } catch (error) {
                dispatch(logoutReducer());
                return {
                    status: "failure",
                    message: t("dialog.requestFailed", {
                        message: error instanceof Error ? error.message : t("dialog.unknownError"),
                    }),
                };
            }
        },
        { status: "idle" },
    );

    const handleNavigateBackward = () => navigate(-1);

    useEffect(() => {
        setDialogLoggedInOpenState(loginStatus.value === true && !pending && actionState.status === "idle");
    }, [actionState.status, loginStatus.value, pending]);

    useEffect(() => {
        if (actionState.status === "success") setDialogLoginSuccessOpenState(true);
        if (actionState.status === "failure" || actionState.status === "validation") {
            setDialogLoginFailureOpenState(true);
        }
    }, [actionState]);

    return (
        <div className={useStyles().root}>
            <form action={loginAction}>
                <Field label={t("field.username")} required>
                    <Input
                        contentBefore={<PersonRegular />}
                        name="username"
                        autoComplete="username"
                        required
                    />
                </Field>
                <Field label={t("field.password")} required>
                    <Input
                        contentBefore={<PasswordRegular />}
                        type="password"
                        name="password"
                        autoComplete="current-password"
                        required
                    />
                </Field>
                <LoginSubmitButton />
            </form>
            <PopupDialog
                open={dialogLoggedInOpenState}
                setPopupDialogOpenState={setDialogLoggedInOpenState}
                text={t("dialog.alreadyLoggedIn")}
                onClose={handleNavigateBackward}
            />
            <PopupDialog
                open={dialogLoginFailureOpenState}
                setPopupDialogOpenState={setDialogLoginFailureOpenState}
                text={actionState.message ?? t("dialog.loginFailed")}
            />
            <PopupDialog
                open={dialogLoginSuccessOpenState}
                setPopupDialogOpenState={setDialogLoginSuccessOpenState}
                text={t("dialog.loginSuccess")}
                onClose={handleNavigateBackward}
            />
        </div>
    );
}
