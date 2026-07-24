import { useState, useEffect, lazy, Suspense } from "react";
import { useLoaderData, useNavigate, useOutletContext } from "react-router-dom";

import {
    Button,
    Dropdown,
    Option,
    Label,
    Title3,
    Subtitle1,
    Spinner,
    Tag,
    Divider,
    SelectionEvents,
    OptionOnSelectData,
} from "@fluentui/react-components";
import {
    StarFilled,
    ThumbLikeRegular,
    ThumbLikeFilled,
    ThumbDislikeRegular,
    ThumbDislikeFilled,
} from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

import MarkdownView from "./MarkdownView.tsx";
import MentionTextarea from "./MentionTextarea.tsx";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));
const EmojiPicker = lazy(() => import("./EmojiPicker.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { formatTimestamp } from "./Solutions.tsx";
import { replaceEmojiShortcuts } from "../EmojiShortcuts.ts";

import "../css/style.css";

interface SolutionInfoFromLoader {
    solutionId: number;
}

interface Solution {
    solution_id: number;
    problem_number: number;
    username: string;
    title: string;
    content: string[];
    is_official: boolean;
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

interface Comment {
    comment_id: number;
    username: string;
    content: string[];
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

// ---------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------

function useSolution(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [solution, setSolution] = useState<Solution | undefined>(undefined);
    const [notFound, setNotFound] = useState(false);

    const loadSolution = (solutionId: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "solution",
            content: { solution_id: solutionId, request_key: _requestKey }
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
            if (_message && typeof _message === 'object' && 'type' in _message && (_message as { type: string }).type === "solution") {
                const message = _message as { type: string; content: Solution & { result?: string; request_key: string } };
                if (message.content.request_key === requestKey) {
                    if (message.content.result === "SNF") setNotFound(true);
                    else setSolution(message.content as Solution);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { solution, setSolution, notFound, loadSolution };
}

interface VoteResult {
    type: string;
    content: {
        solution_id?: number;
        comment_id?: number;
        likes?: number;
        dislikes?: number;
        my_vote?: number;
        reason?: string;
        request_key: string;
    };
}

function useVote(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    resultType: string,
    requestType: string,
    idField: "solution_id" | "comment_id"
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [voteResult, setVoteResult] = useState<VoteResult["content"] | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const castVote = (targetId: number, vote: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: requestType,
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                [idField]: targetId,
                vote: vote,
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
            if (_message && typeof _message === 'object' && 'type' in _message && (_message as { type: string }).type === resultType) {
                const message = _message as VoteResult;
                if (message.content.request_key === requestKey) {
                    setVoteResult(message.content);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { voteResult, castVote };
}

interface CommentsListFromFetch {
    type: string;
    content: {
        comments_list: Comment[];
        request_key: string;
    };
}

function isCommentsListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'comments_list' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useCommentsList(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [commentsList, setCommentsList] = useState<Comment[] | undefined>(undefined);

    const loadCommentsList = (solutionId: number, index: number, sort: string) => {
        const _requestKey = nanoid();
        setCommentsList(undefined);
        sendJsonMessage({
            type: "solution_comments_list",
            content: { solution_id: solutionId, index: index, sort: sort, request_key: _requestKey }
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
            if (_message && isCommentsListFromFetch(_message)) {
                const message = _message as CommentsListFromFetch;
                if (message.content.request_key === requestKey) {
                    setCommentsList(message.content.comments_list);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { commentsList, setCommentsList, loadCommentsList };
}

interface TotalCommentsListIndexFromFetch {
    type: string;
    content: {
        total_solution_comments_list_index: number;
        request_key: string;
    };
}

function isTotalCommentsListIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_solution_comments_list_index' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useTotalCommentsListIndex(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalCommentsListIndex, setTotalCommentsListIndex] = useState(1);

    const loadTotalCommentsListIndex = (solutionId: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "total_solution_comments_list_index",
            content: { solution_id: solutionId, request_key: _requestKey }
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
            if (_message && isTotalCommentsListIndexFromFetch(_message)) {
                const message = _message as TotalCommentsListIndexFromFetch;
                if (message.content.request_key === requestKey) {
                    setTotalCommentsListIndex(message.content.total_solution_comments_list_index);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { totalCommentsListIndex, loadTotalCommentsListIndex };
}

interface PostCommentResult {
    type: string;
    content: {
        comment_id?: number;
        reason?: string;
        request_key: string;
    };
}

function usePostComment(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [postedCommentId, setPostedCommentId] = useState<number | undefined>(undefined);
    const [postFailureReason, setPostFailureReason] = useState<string | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const postComment = (solutionId: number, content: string) => {
        const _requestKey = nanoid();
        setPostedCommentId(undefined);
        setPostFailureReason(undefined);
        sendJsonMessage({
            type: "solution_comment_post",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                solution_id: solutionId,
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
            if (_message && typeof _message === 'object' && 'type' in _message &&
                ((_message as { type: string }).type === "comment_id" || (_message as { type: string }).type === "solution_comment_post_failure")) {
                const message = _message as PostCommentResult;
                if (message.content.request_key === requestKey) {
                    if (message.type === "comment_id" && message.content.comment_id !== undefined)
                        setPostedCommentId(message.content.comment_id);
                    else
                        setPostFailureReason(message.content.reason ?? "unknown");
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { postedCommentId, postFailureReason, postComment };
}

// ---------------------------------------------------------------------------
// Vote buttons
// ---------------------------------------------------------------------------

function VoteButtons({ likes, dislikes, myVote, onVote }: {
    likes: number;
    dislikes: number;
    myVote: number;
    onVote: (vote: number) => void;
}) {
    return <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em" }}>
        <Button
            size="small"
            appearance={myVote === 1 ? "primary" : "outline"}
            icon={myVote === 1 ? <ThumbLikeFilled /> : <ThumbLikeRegular />}
            onClick={() => onVote(myVote === 1 ? 0 : 1)}>
            {String(likes)}
        </Button>
        <Button
            size="small"
            appearance={myVote === -1 ? "primary" : "outline"}
            icon={myVote === -1 ? <ThumbDislikeFilled /> : <ThumbDislikeRegular />}
            onClick={() => onVote(myVote === -1 ? 0 : -1)}>
            {String(dislikes)}
        </Button>
    </div>;
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export default function SolutionShower() {
    const { solutionId } = (useLoaderData() as SolutionInfoFromLoader);
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);

    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [dialogNotFoundOpenState, setDialogNotFoundOpenState] = useState(false);
    const [commentsListIndex, setCommentsListIndex] = useState(1);
    const [commentSort, setCommentSort] = useState("time");
    const [newComment, setNewComment] = useState("");
    const [dialogCommentFailureOpen, setDialogCommentFailureOpen] = useState(false);

    const { solution, setSolution, notFound, loadSolution } = useSolution(sendJsonMessage, lastJsonMessage);
    const { voteResult: solutionVoteResult, castVote: castSolutionVote } =
        useVote(sendJsonMessage, lastJsonMessage, "solution_vote_result", "solution_vote", "solution_id");
    const { commentsList, setCommentsList, loadCommentsList } = useCommentsList(sendJsonMessage, lastJsonMessage);
    const { totalCommentsListIndex, loadTotalCommentsListIndex } = useTotalCommentsListIndex(sendJsonMessage, lastJsonMessage);
    const { postedCommentId, postFailureReason, postComment } = usePostComment(sendJsonMessage, lastJsonMessage);
    const { voteResult: commentVoteResult, castVote: castCommentVote } =
        useVote(sendJsonMessage, lastJsonMessage, "solution_comment_vote_result", "solution_comment_vote", "comment_id");

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true) loadSolution(solutionId);
    }, [loginStatus, solutionId]);

    useEffect(() => {
        if (notFound) setDialogNotFoundOpenState(true);
    }, [notFound]);

    useEffect(() => {
        if (loginStatus.value === true) {
            loadTotalCommentsListIndex(solutionId);
            loadCommentsList(solutionId, commentsListIndex, commentSort);
        }
    }, [loginStatus, solutionId, commentsListIndex, commentSort, postedCommentId]);

    // Fold a solution vote result back into the displayed solution.
    useEffect(() => {
        if (solutionVoteResult !== undefined && solutionVoteResult.likes !== undefined) {
            setSolution((prev) => prev === undefined ? prev : {
                ...prev,
                likes: solutionVoteResult.likes as number,
                dislikes: solutionVoteResult.dislikes as number,
                my_vote: solutionVoteResult.my_vote as number,
            });
        }
    }, [solutionVoteResult]);

    // Fold a comment vote result back into the matching comment.
    useEffect(() => {
        if (commentVoteResult !== undefined && commentVoteResult.likes !== undefined && commentVoteResult.comment_id !== undefined) {
            setCommentsList((prev) => prev === undefined ? prev : prev.map((c) => c.comment_id === commentVoteResult.comment_id ? {
                ...c,
                likes: commentVoteResult.likes as number,
                dislikes: commentVoteResult.dislikes as number,
                my_vote: commentVoteResult.my_vote as number,
            } : c));
        }
    }, [commentVoteResult]);

    useEffect(() => {
        if (postFailureReason !== undefined) setDialogCommentFailureOpen(true);
    }, [postFailureReason]);

    useEffect(() => {
        if (postedCommentId !== undefined) {
            setNewComment("");
            setCommentsListIndex(1);
            loadCommentsList(solutionId, 1, commentSort);
            loadTotalCommentsListIndex(solutionId);
        }
    }, [postedCommentId]);

    const handleCommentSortSelect = (_ev: SelectionEvents, data: OptionOnSelectData) => {
        if (data.optionValue) {
            setCommentSort(data.optionValue);
            setCommentsListIndex(1);
        }
    };

    return <>
        {
            loginStatus.value && solution !== undefined &&
            <div style={{ padding: "0.5em 1em", maxWidth: "90%" }}>
                <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em" }}>
                    {solution.is_official && <StarFilled style={{ color: "#E3B341" }} />}
                    <Title3>{solution.title}</Title3>
                    {solution.is_official && <Tag size="small" appearance="outline">Official</Tag>}
                </div>
                <div style={{ display: "flex", alignItems: "center", columnGap: "1em", margin: "0.5em 0" }}>
                    <Label>By {solution.username}</Label>
                    <Label style={{ color: "#4183C4", cursor: "pointer" }}
                        onClick={() => navigate("/problem/" + String(solution.problem_number))}>
                        Problem {solution.problem_number}
                    </Label>
                    <Label style={{ fontSize: "0.85em", color: "#666" }}>{formatTimestamp(solution.created_at)}</Label>
                    <VoteButtons
                        likes={solution.likes}
                        dislikes={solution.dislikes}
                        myVote={solution.my_vote}
                        onVote={(vote) => castSolutionVote(solution.solution_id, vote)} />
                </div>

                <div style={{ margin: "1em 0" }}>
                    <MarkdownView lines={solution.content} />
                </div>

                <Divider style={{ margin: "1.5em 0 1em 0" }} />

                <div style={{ display: "flex", alignItems: "center", columnGap: "0.75em", marginBottom: "0.75em" }}>
                    <Subtitle1>Comments</Subtitle1>
                    <Dropdown
                        style={{ minWidth: "9em" }}
                        defaultValue="Newest"
                        defaultSelectedOptions={["time"]}
                        onOptionSelect={handleCommentSortSelect}>
                        <Option value="time">Newest</Option>
                        <Option value="likes">Most liked</Option>
                    </Dropdown>
                </div>

                <div style={{ marginBottom: "1em" }}>
                    <MentionTextarea
                        style={{ width: "100%" }}
                        placeholder="Write a comment... (Markdown, @mention)"
                        value={newComment}
                        onChange={setNewComment}
                        sendJsonMessage={sendJsonMessage}
                        lastJsonMessage={lastJsonMessage}
                        rows={3} />
                    <div style={{ marginTop: "0.5em", display: "flex", alignItems: "center", justifyContent: "flex-end", columnGap: "0.5em" }}>
                        <Suspense fallback={<></>}>
                            <EmojiPicker onPick={(emoji) => setNewComment((c) => c + emoji)} />
                        </Suspense>
                        <Button appearance="primary" onClick={() => postComment(solutionId, replaceEmojiShortcuts(newComment))}>Comment</Button>
                    </div>
                </div>

                {
                    commentsList === undefined
                        ?
                        <Spinner size="small" label="Loading comments..." delay={300} />
                        :
                        commentsList.length === 0
                            ?
                            <Label>No comments yet.</Label>
                            :
                            <>
                                <div className="scroll-box" style={{ maxHeight: "34em" }}>
                                {
                                    commentsList.map((comment) => (
                                        <div key={comment.comment_id} style={{ padding: "0.6em 0", borderBottom: "1px solid #eee" }}>
                                            <div style={{ display: "flex", alignItems: "center", columnGap: "0.75em", marginBottom: "0.35em" }}>
                                                <Label weight="semibold">{comment.username}</Label>
                                                <Label style={{ fontSize: "0.8em", color: "#666" }}>{formatTimestamp(comment.created_at)}</Label>
                                            </div>
                                            <div style={{ marginBottom: "0.4em" }}>
                                                <MarkdownView lines={comment.content} />
                                            </div>
                                            <VoteButtons
                                                likes={comment.likes}
                                                dislikes={comment.dislikes}
                                                myVote={comment.my_vote}
                                                onVote={(vote) => castCommentVote(comment.comment_id, vote)} />
                                        </div>
                                    ))
                                }
                                </div>
                                <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em", marginTop: "0.75em" }}>
                                    <Label>Page {commentsListIndex} / {Math.max(totalCommentsListIndex, 1)}</Label>
                                    <Button appearance="secondary" onClick={() => setCommentsListIndex((c) => Math.max(1, c - 1))}>Previous</Button>
                                    <Button appearance="secondary" onClick={() => setCommentsListIndex((c) => Math.max(1, Math.min(totalCommentsListIndex, c + 1)))}>Next</Button>
                                </div>
                            </>
                }
            </div>
        }

        <Suspense fallback={<></>}>
            <PopupDialog
                open={dialogNotFoundOpenState}
                setPopupDialogOpenState={setDialogNotFoundOpenState}
                text={`Solution ${solutionId} is not found!`}
                onClose={() => navigate(-1)} />
            <PopupDialog
                open={dialogRequireLoginOpenState}
                setPopupDialogOpenState={setDialogRequireLoginOpenState}
                text="Please login first."
                onClose={() => navigate("/login")} />
            <PopupDialog
                open={dialogCommentFailureOpen}
                setPopupDialogOpenState={setDialogCommentFailureOpen}
                text={postFailureReason === "empty"
                    ? "Comment can't be empty."
                    : postFailureReason === "invalid_session"
                        ? "Your session is invalid. Please login again."
                        : "Failed to post the comment."}
                onClose={() => setDialogCommentFailureOpen(false)} />
        </Suspense>
    </>;
}
