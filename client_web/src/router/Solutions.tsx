import { useCallback, useEffect, useState, lazy, Suspense } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

import {
    Button,
    Dropdown,
    Option,
    Field,
    Input,
    Label,
    Spinner,
    Table,
    TableHeader,
    TableRow,
    TableHeaderCell,
    TableCell,
    TableBody,
    Tag,
    SelectionEvents,
    OptionOnSelectData,
} from "@fluentui/react-components";
import { StarFilled, ThumbLikeRegular, ThumbDislikeRegular } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import MentionTextarea from "./MentionTextarea.tsx";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));
const EmojiPicker = lazy(() => import("./EmojiPicker.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { replaceEmojiShortcuts } from "../EmojiShortcuts.ts";

export interface SolutionListItem {
    solution_id: number;
    problem_number: number;
    username: string;
    title: string;
    is_official: boolean;
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

export function formatTimestamp(createdAt: number) {
    return new Date(createdAt * 1000).toLocaleString();
}

function useSolutionsList(request: globals.WebSocketRequest) {
    const [solutionsList, setSolutionsList] = useState<SolutionListItem[] | undefined>(undefined);

    const loadSolutionsList = useCallback(async (problemNumber: number, index: number, sort: string, signal?: AbortSignal) => {
        setSolutionsList(undefined);
        try {
            const response = await request<{ type: string; content: { request_key: string; solutions_list?: SolutionListItem[] } }>(
                "solutions_list", { problem_number: problemNumber, index, sort }, { signal },
            );
            if (response.content.solutions_list) setSolutionsList(response.content.solutions_list);
        } catch (error) {
            if (error instanceof Error && error.name === "AbortError") return;
        }
    }, [request]);

    return { solutionsList, loadSolutionsList };
}

function useTotalSolutionsListIndex(request: globals.WebSocketRequest) {
    const [totalSolutionsListIndex, setTotalSolutionsListIndex] = useState(1);

    const loadTotalSolutionsListIndex = useCallback(async (problemNumber: number, signal?: AbortSignal) => {
        try {
            const response = await request<{ type: string; content: { request_key: string; total_solutions_list_index?: number } }>(
                "total_solutions_list_index", { problem_number: problemNumber }, { signal },
            );
            if (response.content.total_solutions_list_index !== undefined) {
                setTotalSolutionsListIndex(response.content.total_solutions_list_index);
            }
        } catch (error) {
            if (error instanceof Error && error.name === "AbortError") return;
        }
    }, [request]);

    return { totalSolutionsListIndex, loadTotalSolutionsListIndex };
}

function usePostSolution(request: globals.WebSocketRequest) {
    const [postedSolutionId, setPostedSolutionId] = useState<number | undefined>(undefined);
    const [postFailureReason, setPostFailureReason] = useState<string | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const postSolution = useCallback(async (problemNumber: number, title: string, content: string) => {
        setPostedSolutionId(undefined);
        setPostFailureReason(undefined);
        try {
            const response = await request<{ type: string; content: { request_key: string; solution_id?: number; reason?: string } }>("solution_post", {
                username: loginUsername.value,
                session_token: sessionToken.value,
                problem_number: problemNumber,
                title,
                content: content.split('\n'),
            });
            if (response.content.solution_id !== undefined) setPostedSolutionId(response.content.solution_id);
            else setPostFailureReason(response.content.reason ?? "unknown");
        } catch {
            setPostFailureReason("request_failed");
        }
    }, [loginUsername.value, request, sessionToken.value]);

    return { postedSolutionId, postFailureReason, postSolution };
}

export default function ProblemSolutions({ problemNumber, request, sendJsonMessage, lastJsonMessage }: {
    problemNumber: number;
    request: globals.WebSocketRequest;
    sendJsonMessage: globals.SendJsonMessage;
    lastJsonMessage: unknown;
}) {
    const { t } = useTranslation(["solutions", "common"]);
    const navigate = useNavigate();
    const [solutionsListIndex, setSolutionsListIndex] = useState(1);
    const [sort, setSort] = useState("time");
    const [newTitle, setNewTitle] = useState("");
    const [newContent, setNewContent] = useState("");
    const [showPostForm, setShowPostForm] = useState(false);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [dialogText, setDialogText] = useState("");

    const { solutionsList, loadSolutionsList } = useSolutionsList(request);
    const { totalSolutionsListIndex, loadTotalSolutionsListIndex } = useTotalSolutionsListIndex(request);
    const { postedSolutionId, postFailureReason, postSolution } = usePostSolution(request);

    useEffect(() => {
        const controller = new AbortController();
        void loadTotalSolutionsListIndex(problemNumber, controller.signal);
        return () => controller.abort();
    }, [loadTotalSolutionsListIndex, problemNumber, postedSolutionId]);

    useEffect(() => {
        const controller = new AbortController();
        void loadSolutionsList(problemNumber, solutionsListIndex, sort, controller.signal);
        return () => controller.abort();
    }, [loadSolutionsList, problemNumber, solutionsListIndex, sort, postedSolutionId]);

    useEffect(() => {
        if (postedSolutionId !== undefined) {
            setNewTitle("");
            setNewContent("");
            setShowPostForm(false);
            navigate("/solution/" + String(postedSolutionId));
        }
    }, [postedSolutionId]);

    useEffect(() => {
        if (postFailureReason !== undefined) {
            setDialogText(postFailureReason === "empty"
                ? t("error.emptyTitleContent")
                : postFailureReason === "invalid_session"
                    ? t("error.invalidSession")
                    : t("error.postFailed"));
            setDialogOpen(true);
        }
    }, [postFailureReason]);

    const handleSortSelect = (_ev: SelectionEvents, data: OptionOnSelectData) => {
        if (data.optionValue) {
            setSort(data.optionValue);
            setSolutionsListIndex(1);
        }
    };

    return <div style={{ margin: "0.5em 1em" }}>
        <div style={{ display: "flex", alignItems: "center", columnGap: "0.75em", marginBottom: "0.75em" }}>
            <Label>{t("sortBy")}</Label>
            <Dropdown
                style={{ minWidth: "10em" }}
                defaultValue={t("sort.newest")}
                defaultSelectedOptions={["time"]}
                onOptionSelect={handleSortSelect}>
                <Option value="time">{t("sort.newest")}</Option>
                <Option value="likes">{t("sort.mostLiked")}</Option>
            </Dropdown>
            <Button appearance="primary" onClick={() => setShowPostForm((x) => !x)} style={{ marginLeft: "auto" }}>
                {showPostForm ? t("action.cancel", { ns: "common" }) : t("writeASolution")}
            </Button>
        </div>

        {
            showPostForm &&
            <div style={{ marginBottom: "1em", padding: "0.75em", border: "1px solid #e0e0e0", borderRadius: "4px" }}>
                <Field label={t("field.title")}>
                    <Input value={newTitle} onChange={(_e, d) => setNewTitle(d.value)} />
                </Field>
                <Field label={t("field.content")} style={{ marginTop: "0.5em" }}>
                    <MentionTextarea
                        value={newContent}
                        onChange={setNewContent}
                        sendJsonMessage={sendJsonMessage}
                        lastJsonMessage={lastJsonMessage}
                        rows={8} />
                </Field>
                <div style={{ marginTop: "0.75em", display: "flex", alignItems: "center", justifyContent: "flex-end", columnGap: "0.5em" }}>
                    <Suspense fallback={<></>}>
                        <EmojiPicker onPick={(emoji) => setNewContent((c) => c + emoji)} />
                    </Suspense>
                    <Button appearance="primary" onClick={() => postSolution(problemNumber, newTitle, replaceEmojiShortcuts(newContent))}>{t("post")}</Button>
                </div>
            </div>
        }

        {
            solutionsList === undefined
                ?
                <Spinner size="small" label={t("loadingSolutions")} delay={300} />
                :
                solutionsList.length === 0
                    ?
                    <Label>{t("noSolutionsYet")}</Label>
                    :
                    <>
                        <Table size="medium">
                            <TableHeader>
                                <TableRow>
                                    <TableHeaderCell style={{ width: "45%" }}>{t("column.title")}</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "20%" }}>{t("column.author")}</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "20%" }}>{t("column.votes")}</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "15%" }}>{t("column.posted")}</TableHeaderCell>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {
                                    solutionsList.map((solution) => (
                                        <TableRow key={solution.solution_id}>
                                            <TableCell
                                                style={{ color: "#4183C4", cursor: "pointer" }}
                                                onClick={() => navigate("/solution/" + String(solution.solution_id))}>
                                                {solution.is_official && <StarFilled style={{ color: "#E3B341", marginRight: "0.35em", verticalAlign: "middle" }} />}
                                                {solution.title}
                                                {solution.is_official && <Tag size="extra-small" appearance="outline" style={{ marginLeft: "0.5em" }}>{t("official")}</Tag>}
                                            </TableCell>
                                            <TableCell>{solution.username}</TableCell>
                                            <TableCell>
                                                <ThumbLikeRegular style={{ verticalAlign: "middle", color: "#3AAF00" }} />&nbsp;{solution.likes}
                                                &nbsp;&nbsp;
                                                <ThumbDislikeRegular style={{ verticalAlign: "middle", color: "#DA3737" }} />&nbsp;{solution.dislikes}
                                            </TableCell>
                                            <TableCell style={{ fontSize: "0.85em" }}>{formatTimestamp(solution.created_at)}</TableCell>
                                        </TableRow>
                                    ))
                                }
                            </TableBody>
                        </Table>
                        <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em", marginTop: "0.75em" }}>
                            <Label>{t("page", { current: solutionsListIndex, total: Math.max(totalSolutionsListIndex, 1) })}</Label>
                            <Button appearance="secondary" onClick={() => setSolutionsListIndex((c) => Math.max(1, c - 1))}>{t("previous")}</Button>
                            <Button appearance="secondary" onClick={() => setSolutionsListIndex((c) => Math.max(1, Math.min(totalSolutionsListIndex, c + 1)))}>{t("next")}</Button>
                        </div>
                    </>
        }

        <Suspense fallback={<></>}>
            <PopupDialog
                open={dialogOpen}
                setPopupDialogOpenState={setDialogOpen}
                text={dialogText}
                onClose={() => setDialogOpen(false)} />
        </Suspense>
    </div>;
}
