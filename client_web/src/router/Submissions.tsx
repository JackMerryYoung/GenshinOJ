import { useEffect, useState, lazy } from "react";
import { Outlet, useLocation, useNavigate, useOutletContext } from "react-router-dom";
import { Divider, Label, makeStyles } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";

import SubmissionsList from "./SubmissionsList.tsx";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "row",
        gap: "0.5em",
        height: "calc(100% - 0.4em)",
        minHeight: 0,
        margin: "0.4em 0.3em 0 0.3em",
        boxSizing: "border-box",
        overflow: "hidden",
        "@media (max-width: 700px)": {
            flexDirection: "column",
            height: "auto",
            overflow: "visible",
        },
    },
    list: {
        flex: "0 0 24.5%",
        minWidth: 0,
        minHeight: 0,
        overflow: "auto",
        "@media (max-width: 700px)": {
            flexBasis: "auto",
            maxHeight: "55vh",
        },
    },
    divider: {
        flex: "0 0 auto",
        height: "100%",
        "@media (max-width: 700px)": {
            display: "none",
        },
    },
    detail: {
        flex: "1 1 0",
        minWidth: 0,
        minHeight: 0,
        overflow: "auto",
        padding: "0 0.25em",
    },
});

export default function Submissions() {
    const styles = useStyles();
    const context = useOutletContext<globals.WebSocketHook>();
    const { t } = useTranslation("submissionsList");
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const location = useLocation();
    const hasSelection = location.pathname !== "/submission" && location.pathname !== "/submission/";

    useEffect(() => {
        setDialogRequireLoginOpenState(loginStatus.value === false);
    }, [loginStatus.value]);

    return <>
        <div className={styles.root}>
            <aside className={styles.list}>
                <SubmissionsList />
            </aside>
            <Divider vertical className={styles.divider} />
            <main className={styles.detail}>
                {hasSelection ? <Outlet context={context} /> : <Label>{t("emptySelection")}</Label>}
            </main>
        </div>
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text={t("pleaseLoginFirst")}
            onClose={() => navigate("/login")} />
    </>;
}
