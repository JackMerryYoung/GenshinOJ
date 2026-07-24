import { Label } from "@fluentui/react-components";
import { useTranslation } from "react-i18next";

export default function Footer() {
    const { t } = useTranslation("footer");

    return <>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", padding: "1em 0 0 0" }}>
            <Label>{t("copyright")}</Label>
        </div>

    </>;
}