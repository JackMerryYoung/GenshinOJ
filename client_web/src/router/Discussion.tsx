import { useEffect, useState, lazy, Suspense } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";

import {
    makeStyles,
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
    SelectionEvents,
    OptionOnSelectData,
} from "@fluentui/react-components";
import { ThumbLikeRegular, ThumbDislikeRegular } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

import MentionTextarea from "./MentionTextarea.tsx";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));
const EmojiPicker = lazy(() => import("./EmojiPicker.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { replaceEmojiShortcuts } from "../EmojiShortcuts.ts";

export interface DiscussionListItem {
    discussion_id: number;
    username: string;
    title: string;
    likes: number;
    dislikes: number;
    created_at: number;
    my_vote: number;
}

export function formatTimestamp(createdAt: number) {
    return new Date(createdAt * 1000).toLocaleString();
}

const useStyles = makeStyles({
    root: {
        margin: "0.9em 1em 0 1em",
    },
});

interface DiscussionsListFromFetch {
    type: string;
    content: { discussions_list: DiscussionListItem[]; request_key: string };
}

function isDiscussionsListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'discussions_list' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useDiscussionsList(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [discussionsList, setDiscussionsList] = useState<DiscussionListItem[] | undefined>(undefined);

    const loadDiscussionsList = (index: number, sort: string) => {
        const _requestKey = nanoid();
        setDiscussionsList(undefined);
        sendJsonMessage({ type: "discussions_list", content: { index, sort, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null)
            setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isDiscussionsListFromFetch(_message)) {
                const message = _message as DiscussionsListFromFetch;
                if (message.content.request_key === requestKey) {
                    setDiscussionsList(message.content.discussions_list);
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { discussionsList, loadDiscussionsList };
}

interface TotalDiscussionsListIndexFromFetch {
    type: string;
    content: { total_discussions_list_index: number; request_key: string };
}

function isTotalDiscussionsListIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_discussions_list_index' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useTotalDiscussionsListIndex(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalDiscussionsListIndex, setTotalDiscussionsListIndex] = useState(1);

    const loadTotalDiscussionsListIndex = () => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "total_discussions_list_index", content: { request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null)
            setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isTotalDiscussionsListIndexFromFetch(_message)) {
                const message = _message as TotalDiscussionsListIndexFromFetch;
                if (message.content.request_key === requestKey) {
                    setTotalDiscussionsListIndex(message.content.total_discussions_list_index);
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { totalDiscussionsListIndex, loadTotalDiscussionsListIndex };
}

interface PostDiscussionResult {
    type: string;
    content: { discussion_id?: number; reason?: string; request_key: string };
}

function isPostDiscussionResult(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return (x as { type: string }).type === "discussion_id" || (x as { type: string }).type === "discussion_post_failure";
    }
    return false;
}

function usePostDiscussion(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [postedDiscussionId, setPostedDiscussionId] = useState<number | undefined>(undefined);
    const [postFailureReason, setPostFailureReason] = useState<string | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const postDiscussion = (title: string, content: string) => {
        const _requestKey = nanoid();
        setPostedDiscussionId(undefined);
        setPostFailureReason(undefined);
        sendJsonMessage({
            type: "discussion_post",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                title,
                content: content.split('\n'),
                request_key: _requestKey,
            },
        });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null)
            setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isPostDiscussionResult(_message)) {
                const message = _message as PostDiscussionResult;
                if (message.content.request_key === requestKey) {
                    if (message.type === "discussion_id" && message.content.discussion_id !== undefined)
                        setPostedDiscussionId(message.content.discussion_id);
                    else
                        setPostFailureReason(message.content.reason ?? "unknown");
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { postedDiscussionId, postFailureReason, postDiscussion };
}

export default function Discussion() {
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const rootStyle = useStyles().root;

    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [discussionsListIndex, setDiscussionsListIndex] = useState(1);
    const [sort, setSort] = useState("time");
    const [newTitle, setNewTitle] = useState("");
    const [newContent, setNewContent] = useState("");
    const [showPostForm, setShowPostForm] = useState(false);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [dialogText, setDialogText] = useState("");

    const { discussionsList, loadDiscussionsList } = useDiscussionsList(sendJsonMessage, lastJsonMessage);
    const { totalDiscussionsListIndex, loadTotalDiscussionsListIndex } = useTotalDiscussionsListIndex(sendJsonMessage, lastJsonMessage);
    const { postedDiscussionId, postFailureReason, postDiscussion } = usePostDiscussion(sendJsonMessage, lastJsonMessage);

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true) loadTotalDiscussionsListIndex();
    }, [loginStatus, postedDiscussionId]);

    useEffect(() => {
        if (loginStatus.value === true) loadDiscussionsList(discussionsListIndex, sort);
    }, [loginStatus, discussionsListIndex, sort, postedDiscussionId]);

    useEffect(() => {
        if (postedDiscussionId !== undefined) {
            setNewTitle("");
            setNewContent("");
            setShowPostForm(false);
            navigate("/discussion/" + String(postedDiscussionId));
        }
    }, [postedDiscussionId]);

    useEffect(() => {
        if (postFailureReason !== undefined) {
            setDialogText(postFailureReason === "empty"
                ? "Title and content can't be empty."
                : postFailureReason === "invalid_session"
                    ? "Your session is invalid. Please login again."
                    : "Failed to post the discussion.");
            setDialogOpen(true);
        }
    }, [postFailureReason]);

    const handleSortSelect = (_ev: SelectionEvents, data: OptionOnSelectData) => {
        if (data.optionValue) { setSort(data.optionValue); setDiscussionsListIndex(1); }
    };

    return <>
        {
            loginStatus.value &&
            <div className={rootStyle}>
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
                        {showPostForm ? "Cancel" : "New discussion"}
                    </Button>
                </div>

                {
                    showPostForm &&
                    <div style={{ marginBottom: "1em", padding: "0.75em", border: "1px solid #e0e0e0", borderRadius: "4px", maxWidth: "60em" }}>
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
                            <Button appearance="primary" onClick={() => postDiscussion(newTitle, replaceEmojiShortcuts(newContent))}>Post</Button>
                        </div>
                    </div>
                }

                {
                    discussionsList === undefined
                        ?
                        <Spinner size="small" label="Loading discussions..." delay={300} />
                        :
                        discussionsList.length === 0
                            ?
                            <Label>No discussions yet. Start one!</Label>
                            :
                            <>
                                <Table size="medium">
                                    <TableHeader>
                                        <TableRow>
                                            <TableHeaderCell style={{ width: "50%" }}>Title</TableHeaderCell>
                                            <TableHeaderCell style={{ width: "20%" }}>Author</TableHeaderCell>
                                            <TableHeaderCell style={{ width: "15%" }}>Votes</TableHeaderCell>
                                            <TableHeaderCell style={{ width: "15%" }}>Posted</TableHeaderCell>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {
                                            discussionsList.map((d) => (
                                                <TableRow key={d.discussion_id}>
                                                    <TableCell
                                                        style={{ color: "#4183C4", cursor: "pointer" }}
                                                        onClick={() => navigate("/discussion/" + String(d.discussion_id))}>
                                                        {d.title}
                                                    </TableCell>
                                                    <TableCell>{d.username}</TableCell>
                                                    <TableCell>
                                                        <ThumbLikeRegular style={{ verticalAlign: "middle", color: "#3AAF00" }} />&nbsp;{d.likes}
                                                        &nbsp;&nbsp;
                                                        <ThumbDislikeRegular style={{ verticalAlign: "middle", color: "#DA3737" }} />&nbsp;{d.dislikes}
                                                    </TableCell>
                                                    <TableCell style={{ fontSize: "0.85em" }}>{formatTimestamp(d.created_at)}</TableCell>
                                                </TableRow>
                                            ))
                                        }
                                    </TableBody>
                                </Table>
                                <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em", marginTop: "0.75em" }}>
                                    <Label>Page {discussionsListIndex} / {Math.max(totalDiscussionsListIndex, 1)}</Label>
                                    <Button appearance="secondary" onClick={() => setDiscussionsListIndex((c) => Math.max(1, c - 1))}>Previous</Button>
                                    <Button appearance="secondary" onClick={() => setDiscussionsListIndex((c) => Math.max(1, Math.min(totalDiscussionsListIndex, c + 1)))}>Next</Button>
                                </div>
                            </>
                }
            </div>
        }

        <Suspense fallback={<></>}>
            <PopupDialog
                open={dialogOpen}
                setPopupDialogOpenState={setDialogOpen}
                text={dialogText}
                onClose={() => setDialogOpen(false)} />
            <PopupDialog
                open={dialogRequireLoginOpenState}
                setPopupDialogOpenState={setDialogRequireLoginOpenState}
                text="Please login first."
                onClose={() => navigate("/login")} />
        </Suspense>
    </>;
}
