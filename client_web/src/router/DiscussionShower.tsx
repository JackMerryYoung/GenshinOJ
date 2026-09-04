import { useState, useEffect, lazy, Suspense } from "react";
import { useLoaderData, useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import {
    Button,
    Dropdown,
    Option,
    Label,
    Title3,
    Subtitle1,
    Spinner,
    Divider,
    SelectionEvents,
    OptionOnSelectData,
} from "@fluentui/react-components";
import {
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
import { formatTimestamp } from "./Discussion.tsx";
import { replaceEmojiShortcuts } from "../EmojiShortcuts.ts";

import "../css/style.css";

interface DiscussionInfoFromLoader {
    discussionId: number;
}

interface Discussion {
    discussion_id: number;
    username: string;
    title: string;
    content: string[];
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

interface Reply {
    reply_id: number;
    username: string;
    content: string[];
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

function useDiscussion(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [discussion, setDiscussion] = useState<Discussion | undefined>(undefined);
    const [notFound, setNotFound] = useState(false);

    const loadDiscussion = (discussionId: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "discussion", content: { discussion_id: discussionId, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && typeof _message === 'object' && 'type' in _message && (_message as { type: string }).type === "discussion") {
                const message = _message as { type: string; content: Discussion & { result?: string; request_key: string } };
                if (message.content.request_key === requestKey) {
                    if (message.content.result === "DNF") setNotFound(true);
                    else setDiscussion(message.content as Discussion);
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { discussion, setDiscussion, notFound, loadDiscussion };
}

interface VoteResult {
    type: string;
    content: { discussion_id?: number; reply_id?: number; likes?: number; dislikes?: number; my_vote?: number; reason?: string; request_key: string };
}

function useVote(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    resultType: string,
    requestType: string,
    idField: "discussion_id" | "reply_id"
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
                vote,
                request_key: _requestKey,
            },
        });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && typeof _message === 'object' && 'type' in _message && (_message as { type: string }).type === resultType) {
                const message = _message as VoteResult;
                if (message.content.request_key === requestKey) { setVoteResult(message.content); delete h[i]; }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { voteResult, castVote };
}

interface RepliesListFromFetch {
    type: string;
    content: { replies_list: Reply[]; request_key: string };
}

function isRepliesListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'replies_list' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useRepliesList(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [repliesList, setRepliesList] = useState<Reply[] | undefined>(undefined);

    const loadRepliesList = (discussionId: number, index: number, sort: string) => {
        const _requestKey = nanoid();
        setRepliesList(undefined);
        sendJsonMessage({ type: "discussion_replies_list", content: { discussion_id: discussionId, index, sort, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isRepliesListFromFetch(_message)) {
                const message = _message as RepliesListFromFetch;
                if (message.content.request_key === requestKey) { setRepliesList(message.content.replies_list); delete h[i]; }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { repliesList, setRepliesList, loadRepliesList };
}

interface TotalRepliesListIndexFromFetch {
    type: string;
    content: { total_discussion_replies_list_index: number; request_key: string };
}

function isTotalRepliesListIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_discussion_replies_list_index' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useTotalRepliesListIndex(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalRepliesListIndex, setTotalRepliesListIndex] = useState(1);

    const loadTotalRepliesListIndex = (discussionId: number) => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "total_discussion_replies_list_index", content: { discussion_id: discussionId, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isTotalRepliesListIndexFromFetch(_message)) {
                const message = _message as TotalRepliesListIndexFromFetch;
                if (message.content.request_key === requestKey) { setTotalRepliesListIndex(message.content.total_discussion_replies_list_index); delete h[i]; }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { totalRepliesListIndex, loadTotalRepliesListIndex };
}

interface PostReplyResult {
    type: string;
    content: { reply_id?: number; reason?: string; request_key: string };
}

function usePostReply(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [postedReplyId, setPostedReplyId] = useState<number | undefined>(undefined);
    const [postFailureReason, setPostFailureReason] = useState<string | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const postReply = (discussionId: number, content: string) => {
        const _requestKey = nanoid();
        setPostedReplyId(undefined);
        setPostFailureReason(undefined);
        sendJsonMessage({
            type: "discussion_reply_post",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                discussion_id: discussionId,
                content: content.split('\n'),
                request_key: _requestKey,
            },
        });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && typeof _message === 'object' && 'type' in _message &&
                ((_message as { type: string }).type === "reply_id" || (_message as { type: string }).type === "discussion_reply_post_failure")) {
                const message = _message as PostReplyResult;
                if (message.content.request_key === requestKey) {
                    if (message.type === "reply_id" && message.content.reply_id !== undefined) setPostedReplyId(message.content.reply_id);
                    else setPostFailureReason(message.content.reason ?? "unknown");
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { postedReplyId, postFailureReason, postReply };
}

function VoteButtons({ likes, dislikes, myVote, onVote }: {
    likes: number; dislikes: number; myVote: number; onVote: (vote: number) => void;
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

export default function DiscussionShower() {
    const { t } = useTranslation(["discussionShower", "common"]);
    const { discussionId } = (useLoaderData() as DiscussionInfoFromLoader);
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);

    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [dialogNotFoundOpenState, setDialogNotFoundOpenState] = useState(false);
    const [repliesListIndex, setRepliesListIndex] = useState(1);
    const [replySort, setReplySort] = useState("time");
    const [newReply, setNewReply] = useState("");
    const [dialogReplyFailureOpen, setDialogReplyFailureOpen] = useState(false);

    const { discussion, setDiscussion, notFound, loadDiscussion } = useDiscussion(sendJsonMessage, lastJsonMessage);
    const { voteResult: discussionVoteResult, castVote: castDiscussionVote } =
        useVote(sendJsonMessage, lastJsonMessage, "discussion_vote_result", "discussion_vote", "discussion_id");
    const { repliesList, setRepliesList, loadRepliesList } = useRepliesList(sendJsonMessage, lastJsonMessage);
    const { totalRepliesListIndex, loadTotalRepliesListIndex } = useTotalRepliesListIndex(sendJsonMessage, lastJsonMessage);
    const { postedReplyId, postFailureReason, postReply } = usePostReply(sendJsonMessage, lastJsonMessage);
    const { voteResult: replyVoteResult, castVote: castReplyVote } =
        useVote(sendJsonMessage, lastJsonMessage, "discussion_reply_vote_result", "discussion_reply_vote", "reply_id");

    useEffect(() => {
        setDialogRequireLoginOpenState(loginStatus.value === false);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true) loadDiscussion(discussionId);
    }, [loginStatus, discussionId]);

    useEffect(() => {
        if (notFound) setDialogNotFoundOpenState(true);
    }, [notFound]);

    useEffect(() => {
        if (loginStatus.value === true) {
            loadTotalRepliesListIndex(discussionId);
            loadRepliesList(discussionId, repliesListIndex, replySort);
        }
    }, [loginStatus, discussionId, repliesListIndex, replySort, postedReplyId]);

    useEffect(() => {
        if (discussionVoteResult !== undefined && discussionVoteResult.likes !== undefined) {
            setDiscussion((prev) => prev === undefined ? prev : {
                ...prev,
                likes: discussionVoteResult.likes as number,
                dislikes: discussionVoteResult.dislikes as number,
                my_vote: discussionVoteResult.my_vote as number,
            });
        }
    }, [discussionVoteResult]);

    useEffect(() => {
        if (replyVoteResult !== undefined && replyVoteResult.likes !== undefined && replyVoteResult.reply_id !== undefined) {
            setRepliesList((prev) => prev === undefined ? prev : prev.map((r) => r.reply_id === replyVoteResult.reply_id ? {
                ...r,
                likes: replyVoteResult.likes as number,
                dislikes: replyVoteResult.dislikes as number,
                my_vote: replyVoteResult.my_vote as number,
            } : r));
        }
    }, [replyVoteResult]);

    useEffect(() => {
        if (postFailureReason !== undefined) setDialogReplyFailureOpen(true);
    }, [postFailureReason]);

    useEffect(() => {
        if (postedReplyId !== undefined) {
            setNewReply("");
            setRepliesListIndex(1);
            loadRepliesList(discussionId, 1, replySort);
            loadTotalRepliesListIndex(discussionId);
        }
    }, [postedReplyId]);

    const handleReplySortSelect = (_ev: SelectionEvents, data: OptionOnSelectData) => {
        if (data.optionValue) { setReplySort(data.optionValue); setRepliesListIndex(1); }
    };

    return <>
        {
            loginStatus.value && discussion !== undefined &&
            <div style={{ padding: "0.5em 1em", maxWidth: "70em" }}>
                <Title3>{discussion.title}</Title3>
                <div style={{ display: "flex", alignItems: "center", columnGap: "1em", margin: "0.5em 0" }}>
                    <Label>{t("byUsername", { username: discussion.username })}</Label>
                    <Label style={{ fontSize: "0.85em", color: "#666" }}>{formatTimestamp(discussion.created_at)}</Label>
                    <VoteButtons
                        likes={discussion.likes}
                        dislikes={discussion.dislikes}
                        myVote={discussion.my_vote}
                        onVote={(vote) => castDiscussionVote(discussion.discussion_id, vote)} />
                </div>

                <div style={{ margin: "1em 0" }}>
                    <MarkdownView lines={discussion.content} />
                </div>

                <Divider style={{ margin: "1.5em 0 1em 0" }} />

                <div style={{ display: "flex", alignItems: "center", columnGap: "0.75em", marginBottom: "0.75em" }}>
                    <Subtitle1>{t("replies")}</Subtitle1>
                    <Dropdown
                        style={{ minWidth: "9em" }}
                        defaultValue={t("sort.newest")}
                        defaultSelectedOptions={["time"]}
                        onOptionSelect={handleReplySortSelect}>
                        <Option value="time">{t("sort.newest")}</Option>
                        <Option value="likes">{t("sort.mostLiked")}</Option>
                    </Dropdown>
                </div>

                <div style={{ marginBottom: "1em" }}>
                    <MentionTextarea
                        style={{ width: "100%" }}
                        placeholder={t("replyPlaceholder")}
                        value={newReply}
                        onChange={setNewReply}
                        sendJsonMessage={sendJsonMessage}
                        lastJsonMessage={lastJsonMessage}
                        rows={3} />
                    <div style={{ marginTop: "0.5em", display: "flex", alignItems: "center", justifyContent: "flex-end", columnGap: "0.5em" }}>
                        <Suspense fallback={<></>}>
                            <EmojiPicker onPick={(emoji) => setNewReply((r) => r + emoji)} />
                        </Suspense>
                        <Button appearance="primary" onClick={() => postReply(discussionId, replaceEmojiShortcuts(newReply))}>{t("action.reply", { ns: "common" })}</Button>
                    </div>
                </div>

                {
                    repliesList === undefined
                        ?
                        <Spinner size="small" label={t("loadingReplies")} delay={300} />
                        :
                        repliesList.length === 0
                            ?
                            <Label>{t("noRepliesYet")}</Label>
                            :
                            <>
                                {
                                    repliesList.map((reply) => (
                                        <div key={reply.reply_id} style={{ padding: "0.6em 0", borderBottom: "1px solid #eee" }}>
                                            <div style={{ display: "flex", alignItems: "center", columnGap: "0.75em", marginBottom: "0.35em" }}>
                                                <Label weight="semibold">{reply.username}</Label>
                                                <Label style={{ fontSize: "0.8em", color: "#666" }}>{formatTimestamp(reply.created_at)}</Label>
                                            </div>
                                            <div style={{ marginBottom: "0.4em" }}>
                                                <MarkdownView lines={reply.content} />
                                            </div>
                                            <VoteButtons
                                                likes={reply.likes}
                                                dislikes={reply.dislikes}
                                                myVote={reply.my_vote}
                                                onVote={(vote) => castReplyVote(reply.reply_id, vote)} />
                                        </div>
                                    ))
                                }
                                <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em", marginTop: "0.75em" }}>
                                    <Label>{t("pageOf", { current: repliesListIndex, total: Math.max(totalRepliesListIndex, 1) })}</Label>
                                    <Button appearance="secondary" onClick={() => setRepliesListIndex((c) => Math.max(1, c - 1))}>{t("previous")}</Button>
                                    <Button appearance="secondary" onClick={() => setRepliesListIndex((c) => Math.max(1, Math.min(totalRepliesListIndex, c + 1)))}>{t("next")}</Button>
                                </div>
                            </>
                }
            </div>
        }

        <Suspense fallback={<></>}>
            <PopupDialog
                open={dialogNotFoundOpenState}
                setPopupDialogOpenState={setDialogNotFoundOpenState}
                text={t("discussionNotFound", { discussionId })}
                onClose={() => navigate(-1)} />
            <PopupDialog
                open={dialogRequireLoginOpenState}
                setPopupDialogOpenState={setDialogRequireLoginOpenState}
                text={t("pleaseLoginFirst")}
                onClose={() => navigate("/login")} />
            <PopupDialog
                open={dialogReplyFailureOpen}
                setPopupDialogOpenState={setDialogReplyFailureOpen}
                text={postFailureReason === "empty"
                    ? t("replyEmpty")
                    : postFailureReason === "invalid_session"
                        ? t("sessionInvalid")
                        : t("replyPostFailed")}
                onClose={() => setDialogReplyFailureOpen(false)} />
        </Suspense>
    </>;
}
