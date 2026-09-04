import { useRef, useEffect, useState, lazy, useCallback } from "react";
import { Outlet, useOutletContext, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

import {
    makeStyles,
    Table,
    TableHeader,
    TableRow,
    TableHeaderCell,
    TableCell,
    TableBody,
    Divider,
    Spinner,
    Label,
    Input,
    Button,
} from "@fluentui/react-components";

import { useSelector } from "react-redux";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";

import { RootState } from "../store.ts";

import "../css/style.css";
import { ErrorCircle20Color, SearchRegular } from "@fluentui/react-icons";


const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "row",
        rowGap: "4px",
        columnGap: "4px",
        height: "fill",
        // Top offset cancels the global `.scroll-bar-wrap { margin: -0.4em }` lift in Root, which
        // otherwise pulls the list's sticky header up into the navbar's bottom divider.
        margin: "0.4em 0.3em 0 0.3em",
    },
});

interface ProblemListItem {
    problem_number: string;
    problem_name: string;
}

function useProblemList(
    request: globals.WebSocketRequest,
    setElapsedTime: React.Dispatch<React.SetStateAction<number>>
) {
    const [problemList, setProblemList] = useState<ProblemListItem[] | undefined>(undefined);
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

    const loadProblemList = useCallback(async (signal?: AbortSignal) => {
        setProblemList(undefined);
        try {
            const response = await request<{
                type: string;
                content: {
                    request_key: string;
                    problem_set?: string[];
                    problems?: { problem_number: number; problem_name: string }[];
                };
            }>(
                "problem_set",
                {},
                { signal },
            );
            if (response.content.problems) {
                setProblemList(response.content.problems.map((problem) => ({
                    problem_number: String(problem.problem_number),
                    problem_name: problem.problem_name,
                })));
            } else if (response.content.problem_set) {
                // Older judge processes return only numbers. Resolve names through the existing
                // problem_statement endpoint so the list remains useful during a rolling update.
                const problemNumbers = response.content.problem_set;
                const problems = await Promise.all(problemNumbers.map(async (problem_number) => {
                    try {
                        const statement = await request<{
                            type: string;
                            content: { problem_number: number; problem_name: string; request_key: string };
                        }>("problem_statement", { problem_number: Number(problem_number) }, { signal });
                        return {
                            problem_number,
                            problem_name: statement.content.problem_name,
                        };
                    } catch (error) {
                        if (error instanceof Error && error.name === "AbortError") throw error;
                        return { problem_number, problem_name: "" };
                    }
                }));
                setProblemList(problems);
            }
        } catch (error) {
            if (error instanceof Error && error.name === "AbortError") return;
            // Keep the existing elapsed-time failure UI for an unavailable backend.
        }
    }, [request]);

    return { problemList, loadProblemList };
}

function TableCellForProblemList({ problem }: {
    problem: ProblemListItem;
}) {
    const navigate = useNavigate();
    const handleClick = () => {
        navigate("/problem/" + problem.problem_number);
    };
    return <TableRow key={problem.problem_number}>
        <TableCell onClick={handleClick} style={{ cursor: "pointer" }}>{problem.problem_number}</TableCell>
        <TableCell onClick={handleClick} style={{ cursor: "pointer" }}>{problem.problem_name || "-"}</TableCell>
    </TableRow>;
}
export function ProblemList({ request }: { request: globals.WebSocketRequest }) {
    const { t } = useTranslation("problem");
    const [elapsedTime, setElapsedTime] = useState(0);
    const { problemList, loadProblemList } = useProblemList(request, setElapsedTime);
    const [searchText, setSearchText] = useState("");
    const [page, setPage] = useState(1);
    const pageSize = 10;

    const normalizedSearchText = searchText.trim().toLocaleLowerCase();
    const filteredProblemList = problemList?.filter((problem) =>
        normalizedSearchText === "" ||
        problem.problem_number.toLocaleLowerCase().includes(normalizedSearchText) ||
        problem.problem_name.toLocaleLowerCase().includes(normalizedSearchText),
    ) ?? [];
    const totalPages = Math.max(1, Math.ceil(filteredProblemList.length / pageSize));
    const visiblePage = Math.min(page, totalPages);
    const visibleProblems = filteredProblemList.slice((visiblePage - 1) * pageSize, visiblePage * pageSize);

    useEffect(() => {
        const controller = new AbortController();
        void loadProblemList(controller.signal);
        return () => controller.abort();
    }, [loadProblemList]);

    useEffect(() => {
        setPage(1);
    }, [normalizedSearchText]);

    useEffect(() => {
        if (page > totalPages) setPage(totalPages);
    }, [page, totalPages]);

    return <>
        {
            problemList !== undefined
                ?
                <div style={{ overflowY: "auto", maxHeight: "60vh" }}>
                    <Input
                        value={searchText}
                        onChange={(_event, data) => setSearchText(data.value)}
                        placeholder={t("search.placeholder")}
                        contentBefore={<SearchRegular />}
                        style={{ width: "100%", marginBottom: "0.5em" }} />
                    <Table size="medium">
                        <TableHeader style={{ position: "sticky", top: 0, zIndex: 1, background: "#fff" }}>
                            <TableRow>
                                <TableHeaderCell>{t("list.number")}</TableHeaderCell>
                                <TableHeaderCell>{t("list.name")}</TableHeaderCell>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {
                                visibleProblems.length > 0
                                    ? visibleProblems.map((problem) => <TableCellForProblemList key={problem.problem_number} problem={problem} />)
                                    : <TableRow><TableCell colSpan={2}><Label>{t("search.empty")}</Label></TableCell></TableRow>
                            }
                        </TableBody>
                    </Table>
                    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "0.25em", marginTop: "0.5em" }}>
                        <Button
                            size="small"
                            disabled={visiblePage <= 1}
                            onClick={() => setPage((current) => Math.max(1, current - 1))}>
                            {t("pagination.previous")}
                        </Button>
                        <Label>{t("pagination.page", { current: visiblePage, total: totalPages })}</Label>
                        <Button
                            size="small"
                            disabled={visiblePage >= totalPages}
                            onClick={() => setPage((current) => Math.min(totalPages, current + 1))}>
                            {t("pagination.next")}
                        </Button>
                    </div>
                </div>
                :
                <div style={{ display: "flex", blockSize: "100%" }}>
                    {
                        elapsedTime >= 10
                            ?
                            <div style={{ display: "flex", margin: "50% 10% 50% 10%" }}>
                                <div style={{ display: "flex", blockSize: "100% 80%" }}>
                                    <ErrorCircle20Color style={{ margin: "auto" }} />
                                    <Label style={{ margin: "auto", padding: "0 0 0 8px", fontSize: "14px" }}>
                                        {t("error.fetchFailed")}
                                    </Label>
                                </div>
                            </div>
                            :
                            <div style={{ display: "flex", margin: "50% 0% 50% 35%" }}>
                                <Spinner size="tiny" label={t("loading.waiting")} delay={500} style={{ margin: "auto" }} />
                            </div>
                    }
                </div>
        }
    </>;
}

export default function Problem() {
    const { t } = useTranslation("problem");
    const { request, sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const style = useStyles();

    useEffect(() => {
        setDialogRequireLoginOpenState(loginStatus.value === false);
    }, [loginStatus]);

    const handleCloseDialogRequireLogin = () => {
        navigate("/login");
    };

    return <>
        {
            loginStatus.value && (
                <div className={style.root}>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.15)" }}>
                        <ProblemList
                            request={request} />
                    </div>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.01)" }}>
                        <Divider vertical style={{ height: "calc(100vh - 8.8em)" }} />
                    </div>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.84)" }}>
                        <Outlet context={{ request, sendJsonMessage, lastJsonMessage }} />
                    </div>
                </div>
            )
        }
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text={t("dialog.pleaseLoginFirst")}
            onClose={handleCloseDialogRequireLogin} />
    </>;
}
