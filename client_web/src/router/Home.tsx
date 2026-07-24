import { Label, makeStyles } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";

import "../css/style.css";

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "column",
        rowGap: "0.25em",
        columnGap: "0.25em",
    },
});

export default function Home() {
    const { t } = useTranslation("home");

    return (
        <>
            <div className={useStyles().root}>
                <div style={{ padding: "0.45em 0.9em" }}>
                    <Label>{t("mainPageDescription")}</Label>
                </div>
            </div>
        </>
    );
}