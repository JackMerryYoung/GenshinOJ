import {
    makeStyles,
    shorthands,
    Input,
    Field,
    Button,
    Dialog,
    DialogBody,
    DialogContent,
    DialogTrigger,
    DialogSurface,
    DialogActions
} from "@fluentui/react-components";

import {
    PersonRegular,
    PasswordRegular
} from "@fluentui/react-icons";

import { useEffect, useState } from "react";

import { useNavigate } from "react-router-dom";

import "../css/style.css";

import * as globals from "./globals";
import React from "react";

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "column",
        ...shorthands.gap("4px"),
        maxWidth: "250px",
    },
});

export default function Login() {
    const [loginUsername, setLoginUsername] = useState("");
    const [loginPassword, setLoginPassword] = useState("");
    const [dialogLoggedInOpenState, setDialogLoggedInOpenState] = useState(false);
    const navigate = useNavigate();
    useEffect(() => { if (globals.getIsLoggedIn()) setDialogLoggedInOpenState(true); }, [])

    return (
        <>
            <div className={useStyles().root}>
                <form>
                    <Field label="Login Username" required>
                        <Input contentBefore={<PersonRegular />} 
                        onInput={(props) => { setLoginUsername(props.currentTarget.value); }}/>
                    </Field>
                    <Field label="Login Password" required>
                        <Input contentBefore={<PasswordRegular />} type="password" 
                        onInput={(props) => { setLoginPassword(props.currentTarget.value); }}/>
                    </Field>
                    <Button appearance="primary" onClick={() => {
                        globals.loginSession(loginUsername, loginPassword).then((result) => {
                            if (result) {
                                navigate(-1);
                            }
                        }).catch((error) => {
                            console.error("Error signing in:", error);
                        });
                    }}>Login</Button>
                </form>
            </div>
            <Dialog modalType="alert" open={dialogLoggedInOpenState} onOpenChange={(event, data) => {
                    setDialogLoggedInOpenState(data.open);
                }}>
                <DialogSurface>
                    <DialogBody>
                        <DialogContent>
                            You have already logged in.
                        </DialogContent>
                        <DialogActions>
                            <DialogTrigger disableButtonEnhancement>
                                <Button appearance="primary" onClick={
                                    () => {
                                        navigate(-1);
                                    }
                                }>Close</Button>
                            </DialogTrigger>
                        </DialogActions>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
        </>
    );
}