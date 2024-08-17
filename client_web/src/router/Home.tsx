import { Label, makeStyles } from "@fluentui/react-components";

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
    return (
        <>
            <div className={useStyles().root}>
                <div style={{ padding: "0.45em 0.9em" }}>
                    <Label>This is the main page of Genshin OJ.</Label>
                </div>
            </div>
        </>
    );
}