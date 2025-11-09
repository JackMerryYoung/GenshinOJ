import { useEffect, useState, lazy, useRef } from "react";
import {
    makeStyles,
    Button,
    Label,
    Field,
    Input,
    Table,
    TableHeader,
    TableRow,
    TableHeaderCell,
    TableCell,
    TableBody,
    Spinner,
} from "@fluentui/react-components";
import { useNavigate, useOutletContext } from "react-router-dom";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { ErrorCircle20Color } from "@fluentui/react-icons";

function getColorByResult(result: string) {
    if (result === "AC") return "#3AAF00";
    else if (result === "CE") return "#FDDB10";
    else if (result === "WA") return "#DA3737";
    else if (result === "UKE") return "#1F103F";
    else return "#4183C4";
}

function getColorByScore(score: number) {
    if (score >= 80) return "#3AAF00";
    else if (score >= 60) return "#FDDB10";
    else return "#DA3737";
}

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "column",
        rowGap: "0.5em",
        columnGap: "0.5em",
        maxWidth: "40%",
        margin: "0.5em 0 0.5em 0",
    },
});

interface SubmissionResult {
    submission_id: number;
    result: string;
    general_score: number;
    statuses: string[];
    scores: number[];
    problem_number: number;
}

interface SubmissionResultOthers {
    submission_id: number;
    result: string;
    problem_number: number;
}

interface SubmissionsListFromFetch {
    type: string;
    content: {
        submissions_list: SubmissionResult[] | SubmissionResultOthers[];
        request_key: string;
    };
}

function isSubmissionsListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'submissions_list' in (x.content as object) &&
            'request_key' in (x.content as object);
    }

    return false;
}

function useSubmissionsList(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    setElapsedTime: React.Dispatch<React.SetStateAction<number>>
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [submissionsList, setSubmissionsList] = useState<SubmissionResult[] | SubmissionResultOthers[] | undefined>(undefined);

    const elapsedTimerSinceLoaded = useRef(0);

    useEffect(() => {
        elapsedTimerSinceLoaded.current = setInterval(() => {
            setElapsedTime(x => x + 1);
        }, 1000);

        return () => {
            if (elapsedTimerSinceLoaded.current) {
                clearInterval(elapsedTimerSinceLoaded.current);
            }
        };
    }, []);

    const loadSubmissionsList = (_submissionsListIndex: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "submissions_list",
            content: {
                index: _submissionsListIndex,
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
        _websocketMessageHistory.map((_message, _index) => {
            if (_message) {
                if (isSubmissionsListFromFetch(_message)) {
                    const message = _message as SubmissionsListFromFetch;
                    if (message.content.request_key == requestKey) {
                        console.log(message);
                        setSubmissionsList(message.content.submissions_list);
                        delete _websocketMessageHistory[_index];
                    }
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { submissionsList, loadSubmissionsList };
}

interface TotalSubmissionsListIndexFromFetch {
    type: string;
    content: {
        total_submissions_list_index: number;
        request_key: string;
    };
}

function isTotalSubmissionsListIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_submissions_list_index' in (x.content as object) &&
            'request_key' in (x.content as object);
    }

    return false;
}

function useTotalSubmissionsListIndex(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalSubmissionsListIndex, setTotalSubmissionsListIndex] = useState(1);

    const loadTotalSubmissionsListIndex = () => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "total_submissions_list_index",
            content: {
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
        _websocketMessageHistory.map((_message, _index) => {
            if (_message) {
                if (isTotalSubmissionsListIndexFromFetch(_message)) {
                    const message = _message as TotalSubmissionsListIndexFromFetch;
                    if (message.content.request_key == requestKey) {
                        console.log(message);
                        setTotalSubmissionsListIndex(message.content.total_submissions_list_index);
                        delete _websocketMessageHistory[_index];
                    }
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { totalSubmissionsListIndex, loadTotalSubmissionsListIndex };
}

export default function SubmissionsList() {
    const [elapsedTime, setElapsedTime] = useState(0);
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const [submissionId, setSubmissionId] = useState("-1");
    const [submissionsListIndex, setSubmissionsListIndex] = useState(1);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const { totalSubmissionsListIndex, loadTotalSubmissionsListIndex } = useTotalSubmissionsListIndex(sendJsonMessage, lastJsonMessage)
    const { submissionsList, loadSubmissionsList } = useSubmissionsList(sendJsonMessage, lastJsonMessage, setElapsedTime)
    const navigate = useNavigate();
    const rootStyle = useStyles().root;

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false)
            setDialogRequireLoginOpenState(true);

    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true)
            loadTotalSubmissionsListIndex();

    }, [loginStatus]);

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);

    }, [submissionsListIndex, loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true)
            loadSubmissionsList(submissionsListIndex);

    }, [submissionsListIndex, loginStatus]);

    const handleNavigateLogin = () => {
        navigate("/login");
    };

    return <>
        {
            loginStatus.value &&
            (
                submissionsList === undefined ?
                    <div style={{ padding: "0.5em 0.5em 0 0.5em" }}>
                        {
                            elapsedTime >= 10
                                ?
                                <div style={{ display: "flex", blockSize: "100%" }}>
                                    <ErrorCircle20Color style={{ margin: "auto 0 0 auto" }} />
                                    <Label style={{ margin: "auto auto 0 0", padding: "0 0 0 8px", fontSize: "14px" }}>
                                        Failed to fetch the submission list.
                                    </Label></div>
                                :
                                <Spinner size="large" label="Waiting..." delay={500} />
                        }
                    </div>
                    :
                    <>
                        <div className={rootStyle}>
                            <form style={{ padding: "0 0.9em" }}>
                                <Field label="Submission ID">
                                    <div style={{ display: "flex", columnGap: "0.25em" }}>
                                        <Input onChange={(props) => setSubmissionId(props.target.value)} style={{ flex: "75%" }} />
                                        <Button appearance="primary"
                                            style={{ flex: "25%" }}
                                            onClick={() => { navigate("/submission/" + String(submissionId)); }}>Jump to</Button>
                                    </div>
                                </Field>
                                <div style={{ marginTop: "1em" }}>
                                    <div style={{ margin: "0 0 0.25em 0" }}>
                                        <Label>Submission Index Page of {submissionsListIndex} / {Math.max(totalSubmissionsListIndex, 1)}</Label>
                                    </div>
                                    <div style={{ display: "flex", columnGap: "0.25em" }}>
                                        <Button appearance="primary"
                                            style={{ flex: "25%" }}
                                            onClick={() => { setSubmissionsListIndex((c) => (Math.max(1, c - 1))); }}>Previous</Button>
                                        <Button appearance="primary"
                                            style={{ flex: "25%" }}
                                            onClick={() => { setSubmissionsListIndex((c) => (Math.max(1, Math.min(totalSubmissionsListIndex, c + 1)))); }}>Next</Button>
                                        <Button appearance="secondary"
                                            style={{ flex: "25%" }}
                                            onClick={() => { setSubmissionsListIndex((c) => (Math.max(1, c - 10))); }}>Backward 10</Button>
                                        <Button appearance="secondary"
                                            style={{ flex: "25%" }}
                                            onClick={() => { setSubmissionsListIndex((c) => (Math.max(1, Math.min(1, totalSubmissionsListIndex, c + 10)))); }}>Forward 10</Button>
                                    </div>
                                </div>
                            </form>
                        </div>
                        <div style={{ margin: "0.6em 0.25em 0 0.75em", maxWidth: "90%", marginTop: "1em" }} >
                            <Table size="medium" className="scroll-bar-wrap">
                                <TableHeader className="my-table-sticky">
                                    <TableRow className="my-table-row-header">
                                        <TableHeaderCell style={{ width: "25%" }} className="my-table-cell">Submission ID</TableHeaderCell>
                                        <TableHeaderCell style={{ width: "25%" }}>Problem Number</TableHeaderCell>
                                        <TableHeaderCell style={{ width: "25%" }}>Status</TableHeaderCell>
                                        <TableHeaderCell style={{ width: "25%" }}>Score</TableHeaderCell>
                                    </TableRow>
                                </TableHeader>
                                <TableBody className="scroll-box my-table-scrollbar">
                                    {
                                        submissionsList.map((submissionResult: SubmissionResult | SubmissionResultOthers, key) => (
                                            <TableRow key={key} className="my-table-row-body">
                                                <TableCell style={{ color: "#4183C4", width: "25%" }} className="my-table-cell"
                                                    onMouseEnter={(e) => { (e.target as HTMLTableCellElement).style.color = "#0056B3"; (e.target as HTMLTableCellElement).style.cursor = "pointer"; }}
                                                    onMouseLeave={(e) => { (e.target as HTMLTableCellElement).style.color = "#4183C4"; (e.target as HTMLTableCellElement).style.cursor = "default"; }}
                                                    onClick={() => { navigate("/submission/" + String(submissionResult.submission_id)); }}>
                                                    {submissionResult.submission_id}
                                                </TableCell>
                                                <TableCell style={{ color: "#4183C4", width: "25%" }} className="my-table-cell"
                                                    onMouseEnter={(e) => { (e.target as HTMLTableCellElement).style.color = "#0056B3"; (e.target as HTMLTableCellElement).style.cursor = "pointer"; }}
                                                    onMouseLeave={(e) => { (e.target as HTMLTableCellElement).style.color = "#4183C4"; (e.target as HTMLTableCellElement).style.cursor = "default"; }}
                                                    onClick={() => { navigate("/problem/" + String(submissionResult.problem_number)); }}>
                                                    {submissionResult.problem_number}
                                                </TableCell>
                                                <TableCell style={{ color: getColorByResult('result' in submissionResult ? submissionResult.result : "PD"), width: "25%" }} className="my-table-cell">{'result' in submissionResult ? submissionResult.result : "PD"}</TableCell>
                                                <TableCell style={{ color: "general_score" in submissionResult ? getColorByScore(submissionResult.general_score) : undefined, width: "25%" }} className="my-table-cell">{"general_score" in submissionResult ? submissionResult.general_score : "-"}</TableCell>
                                            </TableRow>
                                        )
                                        )
                                    }
                                </TableBody>
                                <div className="cover-bar" />
                            </Table>
                        </div>
                    </>
            )
        }
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text="Please login first."
            onClose={handleNavigateLogin} />
    </>;
}

414.55
418.3