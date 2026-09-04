import { lazy, useActionState, useEffect, useState } from "react";
import { useFormStatus } from "react-dom";
import { useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button, Field, Input, makeStyles } from "@fluentui/react-components";
import { PasswordRegular, PersonRegular } from "@fluentui/react-icons";
import { useSelector } from "react-redux";

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
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

type RegisterActionState = {
    status: "idle" | "success" | "failure" | "validation";
    message?: string;
};

function RegisterSubmitButton() {
    const { t } = useTranslation("register");
    const { pending } = useFormStatus();

    return (
        <Button appearance="primary" type="submit" disabled={pending} style={{ marginTop: "1em" }}>
            {pending ? t("action.pending") : t("action.register")}
        </Button>
    );
}

export default function Register() {
    const { t } = useTranslation("register");
    const { request } = useOutletContext<globals.WebSocketHook>();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const navigate = useNavigate();
    const [dialogLoggedInOpenState, setDialogLoggedInOpenState] = useState(false);
    const [dialogRegisterFailureOpenState, setDialogRegisterFailureOpenState] = useState(false);
    const [dialogRegisterSuccessOpenState, setDialogRegisterSuccessOpenState] = useState(false);

    const [actionState, registerAction, pending] = useActionState<RegisterActionState, FormData>(
        async (_previousState, formData) => {
            const username = String(formData.get("username") ?? "").trim();
            const password = String(formData.get("password") ?? "");
            const passwordConfirmation = String(formData.get("passwordConfirmation") ?? "");

            if (!username || !password || !passwordConfirmation) {
                return { status: "validation", message: t("validation.required") };
            }
            if (password !== passwordConfirmation) {
                return { status: "validation", message: t("dialog.passwordMismatch") };
            }

            try {
                const response = await request<{
                    type: "quit";
                    content: { request_key: string; reason?: string };
                }>("register", { username, password }, { responseTypes: ["quit"] });

                if (response.content.reason !== "registration_success") {
                    return { status: "failure", message: t("dialog.registerFailure") };
                }

                return { status: "success" };
            } catch (error) {
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

    useEffect(() => {
        setDialogLoggedInOpenState(loginStatus.value === true && !pending && actionState.status === "idle");
    }, [actionState.status, loginStatus.value, pending]);

    useEffect(() => {
        if (actionState.status === "success") setDialogRegisterSuccessOpenState(true);
        if (actionState.status === "failure" || actionState.status === "validation") {
            setDialogRegisterFailureOpenState(true);
        }
    }, [actionState]);

    return (
        <div className={useStyles().root}>
            <form action={registerAction}>
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
                        autoComplete="new-password"
                        required
                    />
                </Field>
                <Field label={t("field.confirmPassword")} required>
                    <Input
                        contentBefore={<PasswordRegular />}
                        type="password"
                        name="passwordConfirmation"
                        autoComplete="new-password"
                        required
                    />
                </Field>
                <RegisterSubmitButton />
            </form>
            <PopupDialog
                open={dialogLoggedInOpenState && actionState.status !== "success"}
                setPopupDialogOpenState={setDialogLoggedInOpenState}
                text={t("dialog.alreadyLoggedIn")}
                onClose={() => navigate("/home")}
            />
            <PopupDialog
                open={dialogRegisterFailureOpenState}
                setPopupDialogOpenState={setDialogRegisterFailureOpenState}
                text={actionState.message ?? t("dialog.registerFailure")}
            />
            <PopupDialog
                open={dialogRegisterSuccessOpenState}
                setPopupDialogOpenState={setDialogRegisterSuccessOpenState}
                text={t("dialog.registerSuccess")}
                onClose={() => navigate("/login")}
            />
        </div>
    );
}
