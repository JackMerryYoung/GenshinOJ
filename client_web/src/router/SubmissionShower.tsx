import { useState, useEffect, BaseSyntheticEvent, lazy, ReactNode, Suspense } from "react";
import { useLoaderData, useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { ClipboardCheckmarkRegular, ClipboardCodeRegular, ClipboardTaskFilled, ReOrderDotsHorizontalRegular, StatusRegular } from "@fluentui/react-icons";
import { Label, Spinner, Table, TableBody, TableHeader, TableHeaderCell, TableCell, TableRow } from "@fluentui/react-components";

import { useSelector } from "react-redux";
const copy = (await import("copy-to-clipboard")).default;
import { nanoid } from "nanoid";

const SyntaxHighlighter = lazy(() => import("react-syntax-highlighter"));

const BadgeButton = lazy(() => import("./BadgeButton.tsx"));
const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";

import { RootState } from "../store.ts";

import "../css/font.css";

interface SubmissionInfoFromLoader {
    submissionId: number;
}

interface SubmissionResult {
    submission_id: number;
    result: string;
    general_score?: number;
    statuses?: string[];
    scores?: number[];
    problem_number: number;
    code: string[];
    language: string;
    username: string;
}

function CopyToTheClipboardButton({ content }: {
    content: string;
}) {
    const { t } = useTranslation("submissionShower");
    const [tipText, setTipText] = useState(t("action.copy"));
    const [tipIcon, setTipIcon] = useState<ReactNode>(<ClipboardCodeRegular fontSize="1.1em" style={{ margin: "0 0.25em 0 0" }} />);
    useEffect(() => {
        if (tipText === t("action.copied"))
            setTimeout(() => {
                if (tipText === t("action.copied")) setTipText(t("action.copy")), setTipIcon(<ClipboardCodeRegular fontSize="1.1em" style={{ margin: "0 0.25em 0 0" }} />);
            }, 500);
    }, [tipText]);

    const handleOnClick = (_: BaseSyntheticEvent) => {
        copy(content);
        setTipText(t("action.copied"));
        setTipIcon(<ClipboardTaskFilled fontSize="1.1em" style={{ margin: "0 0.25em 0 0" }} />);
    };

    return <BadgeButton style={{ marginLeft: "auto", marginRight: "0.25em" }} onClick={handleOnClick}>{tipIcon}<span style={{ fontSize: "1.1em" }}>{tipText}</span></BadgeButton>
}

function useSubmissionResult(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    submissionId: number
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [submissionResult, setSubmissionResult] = useState<SubmissionResult | undefined>(undefined);

    const loadSubmissionResult = (submissionId: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "submission_result",
            content: {
                submission_id: submissionId,
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
            interface SubmissionResultFromFetch {
                type: string;
                content: {
                    submission_id: number;
                    result: string;
                    general_score: number;
                    statuses: string[];
                    scores: number[];
                    problem_number: number;
                    code: string[];
                    language: string;
                    username: string;
                    request_key?: string;
                };
            }

            function isSubmissionResultFromFetch(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'submission_id' in (x.content as object) &&
                        'result' in (x.content as object) &&
                        'general_score' in (x.content as object) &&
                        'statuses' in (x.content as object) &&
                        'scores' in (x.content as object) &&
                        'problem_number' in (x.content as object) &&
                        'code' in (x.content as object);
                }

                return false;
            }

            interface SubmissionResultOthersFromFetch {
                type: string;
                content: {
                    submission_id: number;
                    result: string;
                    general_score?: number;
                    statuses?: string[];
                    scores?: number[];
                    problem_number?: number;
                    code?: string[];
                    language?: string;
                    username?: string;
                    request_key?: string;
                };
            }

            function isSubmissionResultOthersFromFetch(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'submission_id' in (x.content as object) &&
                        'result' in (x.content as object);
                }

                return false;
            }

            if (_message) {
                console.log(_message);
                if (isSubmissionResultFromFetch(_message)) {
                    const message = _message as SubmissionResultFromFetch;
                    console.log(message);
                    if ((message.content.request_key !== undefined && message.content.request_key === requestKey) || message.content.submission_id === submissionId) {
                        setSubmissionResult(message.content as SubmissionResult);
                        delete _websocketMessageHistory[_index];
                    }
                }
                else if (isSubmissionResultOthersFromFetch(_message)) {
                    const message = _message as SubmissionResultOthersFromFetch;
                    console.log(message);
                    if ((message.content.request_key !== undefined && message.content.request_key === requestKey) || message.content.submission_id === submissionId) {
                        setSubmissionResult(message.content as SubmissionResult);
                        delete _websocketMessageHistory[_index];
                    }
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory]);

    return { submissionResult, loadSubmissionResult };
}

export default function SubmissionShower() {
    const { submissionId } = (useLoaderData() as SubmissionInfoFromLoader);
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [dialogSubmissionNotFoundOpenState, setDialogSubmissionNotFoundOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const { submissionResult, loadSubmissionResult } = useSubmissionResult(sendJsonMessage, lastJsonMessage, submissionId);
    const navigate = useNavigate();
    const { t } = useTranslation("submissionShower");

    const convertCodeToRenderString = (x: string[]) => {
        let renderString: string = "";
        x.map((_s, index) => {
            renderString = renderString + _s.replace("\t", "    ");
            if (index !== x.length - 1) renderString = renderString + '\n';
        });

        return renderString;
    };

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true)
            loadSubmissionResult(submissionId);
    }, [loginStatus]);

    useEffect(() => {
        if (submissionResult !== undefined && submissionResult.result === "SNF") setDialogSubmissionNotFoundOpenState(true);
    }, [submissionResult]);

    const handleNavigateLogin = () => {
        navigate("/login");
    };

    const handleNavigateBackward = () => {
        navigate(-1);
    };

    return <>
        {
            loginStatus.value && (
                <div style={{ padding: "0.25em 0", maxWidth: "90%" }}>
                    <div style={{ display: "flex", marginBottom: "0.75em" }}>
                        <Label style={{ margin: "0.20em 0.15em 0.20em 0.85em" }}>{t("submissionId")}: {submissionResult?.submission_id}</Label>
                        {
                            submissionResult !== undefined && submissionResult.result !== "SNF"
                                ?
                                <>
                                    <div style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("submissionProblem")}:&nbsp;
                                        <Label style={{ color: "#4183C4" }}
                                            onMouseEnter={(e) => { (e.target as HTMLTableCellElement).style.color = "#0056B3"; (e.target as HTMLTableCellElement).style.cursor = "pointer"; }}
                                            onMouseLeave={(e) => { (e.target as HTMLTableCellElement).style.color = "#4183C4"; (e.target as HTMLTableCellElement).style.cursor = "default"; }}
                                            onClick={() => { navigate("/problem/" + String(submissionResult.problem_number)); }}>
                                            {submissionResult.problem_number}
                                        </Label>
                                    </div>
                                    {
                                        (
                                            submissionResult.result === "PD"
                                                ?
                                                <>
                                                    <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("status")}:&nbsp;</Label>
                                                    <Spinner style={{ margin: "0.20em 0" }} size="tiny" label={t("waiting")} delay={500} />
                                                </>
                                                :
                                                (
                                                    submissionResult.result === "AC"
                                                        ?
                                                        <>
                                                            <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("status")}:&nbsp;</Label>
                                                            <Label style={{ margin: "0.20em 0", color: "#3AAF00" }}>AC</Label>
                                                        </>
                                                        :
                                                        (
                                                            submissionResult.result === "CE"
                                                                ?
                                                                <>
                                                                    <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("status")}:&nbsp;</Label>
                                                                    <Label style={{ margin: "0.20em 0", color: "#FDDB10" }}>CE</Label>
                                                                </>
                                                                :
                                                                (
                                                                    submissionResult.result === "WA"
                                                                        ?
                                                                        <>
                                                                            <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("status")}:&nbsp;</Label>
                                                                            <Label style={{ margin: "0.20em 0", color: "#DA3737" }}>WA</Label>
                                                                        </>
                                                                        :
                                                                        <></>
                                                                )

                                                        )

                                                )
                                        )
                                    }
                                    <Label style={{ margin: "0.20em 0.15em 0.20em 1.85em" }}>{t("score")}:&nbsp;</Label>
                                    {
                                        submissionResult.general_score !== undefined
                                            ?
                                            <Label style={{
                                                margin: "0.20em 0",
                                                color: (submissionResult.general_score >= 80) ? "#3AAF00" : ((submissionResult.general_score >= 60) ? "#FDDB10" : "#DA3737")
                                            }}>
                                                {submissionResult.general_score}
                                            </Label>
                                            :
                                            <Label>{`-`}</Label>
                                    }

                                    <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("language")}:&nbsp;{submissionResult.language}</Label>
                                    <Label style={{ margin: "0.20em 0.15em 0.20em 1.55em" }}>{t("submitter")}:&nbsp;{submissionResult.username}</Label>
                                </>
                                :
                                <></>
                        }
                    </div>
                    {
                        submissionResult !== undefined && submissionResult.scores !== undefined && submissionResult.statuses !== undefined
                            ?
                            <div style={{ margin: "0.20em 0.75em", marginBottom: "0.75em" }}>
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHeaderCell><ReOrderDotsHorizontalRegular fontSize="1.4em" /><Label>{t("column.testCaseId")}</Label></TableHeaderCell>
                                            <TableHeaderCell><StatusRegular fontSize="1.4em" /><Label>{t("status")}</Label></TableHeaderCell>
                                            <TableHeaderCell><ClipboardCheckmarkRegular fontSize="1.4em" /><Label>{t("score")}</Label></TableHeaderCell>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {
                                            submissionResult.statuses.map((status, index) => {
                                                const _: number[] = submissionResult.scores as number[];
                                                return <TableRow key={index}>
                                                    <TableCell>{index + 1}</TableCell>
                                                    <TableCell style={{
                                                        color: status === "AC" ? "#3AAF00" : status === "WA" ? "#DA3737" : "#000643"
                                                    }}>{status}</TableCell>
                                                    <TableCell style={{
                                                        color: _[index] !== 0 ? "#3AAF00" : "#DA3737"
                                                    }}>{_[index]}</TableCell>
                                                </TableRow>;
                                            })
                                        }
                                    </TableBody>
                                </Table>
                            </div>
                            :
                            <></>
                    }

                    {
                        submissionResult !== undefined && submissionResult.code !== undefined
                            ?
                            <div>
                                <div style={{ marginLeft: "0.85em", display: "flex" }}>
                                    <Label style={{ marginRight: "auto" }}>{t("code")}:&nbsp;</Label>
                                    <CopyToTheClipboardButton content={convertCodeToRenderString(submissionResult.code)} />
                                </div>
                                <div>
                                    <Suspense fallback={<Spinner size="tiny" delay={500} />}>
                                        <SyntaxHighlighter showLineNumbers={true} >{convertCodeToRenderString(submissionResult.code)}</SyntaxHighlighter>
                                    </Suspense>

                                </div>
                            </div>
                            :
                            <></>
                    }
                </div>
            )
        }
        <PopupDialog
            open={dialogSubmissionNotFoundOpenState}
            setPopupDialogOpenState={setDialogSubmissionNotFoundOpenState}
            text={t("submissionNotFound", { submissionId })}
            onClose={handleNavigateBackward} />
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text={t("pleaseLoginFirst")}
            onClose={handleNavigateLogin} />
    </>
}