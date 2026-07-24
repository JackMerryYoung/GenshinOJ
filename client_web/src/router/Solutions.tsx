import { useEffect, useState, lazy, Suspense } from "react";
import { useNavigate } from "react-router-dom";

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

import { nanoid } from "nanoid";

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

interface SolutionsListFromFetch {
    type: string;
    content: {
        solutions_list: SolutionListItem[];
        request_key: string;
    };
}

function isSolutionsListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'solutions_list' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useSolutionsList(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [solutionsList, setSolutionsList] = useState<SolutionListItem[] | undefined>(undefined);

    const loadSolutionsList = (problemNumber: number, index: number, sort: string) => {
        const _requestKey = nanoid();
        setSolutionsList(undefined);
        sendJsonMessage({
            type: "solutions_list",
            content: {
                problem_number: problemNumber,
                index: index,
                sort: sort,
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
            if (_message && isSolutionsListFromFetch(_message)) {
                const message = _message as SolutionsListFromFetch;
                if (message.content.request_key === requestKey) {
                    setSolutionsList(message.content.solutions_list);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { solutionsList, loadSolutionsList };
}

interface TotalSolutionsListIndexFromFetch {
    type: string;
    content: {
        total_solutions_list_index: number;
        request_key: string;
    };
}

function isTotalSolutionsListIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_solutions_list_index' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useTotalSolutionsListIndex(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalSolutionsListIndex, setTotalSolutionsListIndex] = useState(1);

    const loadTotalSolutionsListIndex = (problemNumber: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "total_solutions_list_index",
            content: {
                problem_number: problemNumber,
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
            if (_message && isTotalSolutionsListIndexFromFetch(_message)) {
                const message = _message as TotalSolutionsListIndexFromFetch;
                if (message.content.request_key === requestKey) {
                    setTotalSolutionsListIndex(message.content.total_solutions_list_index);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { totalSolutionsListIndex, loadTotalSolutionsListIndex };
}

interface PostSolutionResult {
    type: string;
    content: {
        solution_id?: number;
        reason?: string;
        request_key: string;
    };
}

function isPostSolutionResult(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return (x as { type: string }).type === "solution_id" || (x as { type: string }).type === "solution_post_failure";
    }
    return false;
}

function usePostSolution(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [postedSolutionId, setPostedSolutionId] = useState<number | undefined>(undefined);
    const [postFailureReason, setPostFailureReason] = useState<string | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const postSolution = (problemNumber: number, title: string, content: string) => {
        const _requestKey = nanoid();
        setPostedSolutionId(undefined);
        setPostFailureReason(undefined);
        sendJsonMessage({
            type: "solution_post",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                problem_number: problemNumber,
                title: title,
                content: content.split('\n'),
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
            if (_message && isPostSolutionResult(_message)) {
                const message = _message as PostSolutionResult;
                if (message.content.request_key === requestKey) {
                    if (message.type === "solution_id" && message.content.solution_id !== undefined)
                        setPostedSolutionId(message.content.solution_id);
                    else
                        setPostFailureReason(message.content.reason ?? "unknown");
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { postedSolutionId, postFailureReason, postSolution };
}

export default function ProblemSolutions({ problemNumber, sendJsonMessage, lastJsonMessage }: {
    problemNumber: number;
    sendJsonMessage: globals.SendJsonMessage;
    lastJsonMessage: unknown;
}) {
    const navigate = useNavigate();
    const [solutionsListIndex, setSolutionsListIndex] = useState(1);
    const [sort, setSort] = useState("time");
    const [newTitle, setNewTitle] = useState("");
    const [newContent, setNewContent] = useState("");
    const [showPostForm, setShowPostForm] = useState(false);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [dialogText, setDialogText] = useState("");

    const { solutionsList, loadSolutionsList } = useSolutionsList(sendJsonMessage, lastJsonMessage);
    const { totalSolutionsListIndex, loadTotalSolutionsListIndex } = useTotalSolutionsListIndex(sendJsonMessage, lastJsonMessage);
    const { postedSolutionId, postFailureReason, postSolution } = usePostSolution(sendJsonMessage, lastJsonMessage);

    useEffect(() => {
        loadTotalSolutionsListIndex(problemNumber);
    }, [problemNumber, postedSolutionId]);

    useEffect(() => {
        loadSolutionsList(problemNumber, solutionsListIndex, sort);
    }, [problemNumber, solutionsListIndex, sort, postedSolutionId]);

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
                ? "Title and content can't be empty."
                : postFailureReason === "invalid_session"
                    ? "Your session is invalid. Please login again."
                    : "Failed to post the solution.");
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
            <Label>Sort by</Label>
            <Dropdown
                style={{ minWidth: "10em" }}
                defaultValue="Newest"
                defaultSelectedOptions={["time"]}
                onOptionSelect={handleSortSelect}>
                <Option value="time">Newest</Option>
                <Option value="likes">Most liked</Option>
            </Dropdown>
            <Button appearance="primary" onClick={() => setShowPostForm((x) => !x)} style={{ marginLeft: "auto" }}>
                {showPostForm ? "Cancel" : "Write a solution"}
            </Button>
        </div>

        {
            showPostForm &&
            <div style={{ marginBottom: "1em", padding: "0.75em", border: "1px solid #e0e0e0", borderRadius: "4px" }}>
                <Field label="Title">
                    <Input value={newTitle} onChange={(_e, d) => setNewTitle(d.value)} />
                </Field>
                <Field label="Content (Markdown, @mention)" style={{ marginTop: "0.5em" }}>
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
                    <Button appearance="primary" onClick={() => postSolution(problemNumber, newTitle, replaceEmojiShortcuts(newContent))}>Post</Button>
                </div>
            </div>
        }

        {
            solutionsList === undefined
                ?
                <Spinner size="small" label="Loading solutions..." delay={300} />
                :
                solutionsList.length === 0
                    ?
                    <Label>No solutions yet. Be the first to write one!</Label>
                    :
                    <>
                        <Table size="medium">
                            <TableHeader>
                                <TableRow>
                                    <TableHeaderCell style={{ width: "45%" }}>Title</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "20%" }}>Author</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "20%" }}>Votes</TableHeaderCell>
                                    <TableHeaderCell style={{ width: "15%" }}>Posted</TableHeaderCell>
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
                                                {solution.is_official && <Tag size="extra-small" appearance="outline" style={{ marginLeft: "0.5em" }}>Official</Tag>}
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
                            <Label>Page {solutionsListIndex} / {Math.max(totalSolutionsListIndex, 1)}</Label>
                            <Button appearance="secondary" onClick={() => setSolutionsListIndex((c) => Math.max(1, c - 1))}>Previous</Button>
                            <Button appearance="secondary" onClick={() => setSolutionsListIndex((c) => Math.max(1, Math.min(totalSolutionsListIndex, c + 1)))}>Next</Button>
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
