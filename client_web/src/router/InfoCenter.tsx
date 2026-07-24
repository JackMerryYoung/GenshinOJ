import { useEffect, useState, lazy, Suspense } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";

import {
    Button,
    Label,
    Spinner,
    Badge,
} from "@fluentui/react-components";
import { PersonFilled, CommentMultipleFilled, MentionFilled } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { formatTimestamp } from "./Discussion.tsx";

import "../css/style.css";

interface NotificationItem {
    notification_id: number;
    actor: string;
    kind: string;
    source_type: string;
    target_url: string;
    excerpt: string;
    is_read: boolean;
    created_at: number;
}

// Human-readable summary of why this notification arrived.
function describe(item: NotificationItem) {
    if (item.kind === "mention") {
        if (item.source_type === "solution") return "mentioned you in a solution";
        if (item.source_type === "solution_comment") return "mentioned you in a comment";
        if (item.source_type === "discussion") return "mentioned you in a discussion";
        if (item.source_type === "discussion_reply") return "mentioned you in a reply";
        return "mentioned you";
    }
    // kind === "reply"
    if (item.source_type === "solution_comment") return "commented on your solution";
    if (item.source_type === "discussion_reply") return "replied to your discussion";
    return "replied to you";
}

function isNotificationsListFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'notifications_list' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useNotificationsList(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [notificationsList, setNotificationsList] = useState<NotificationItem[] | undefined>(undefined);

    const loadNotificationsList = (index: number) => {
        const _requestKey = nanoid();
        setNotificationsList(undefined);
        sendJsonMessage({ type: "notifications_list", content: { index, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isNotificationsListFromFetch(_message)) {
                const message = _message as { content: { notifications_list: NotificationItem[]; request_key: string } };
                if (message.content.request_key === requestKey) {
                    setNotificationsList(message.content.notifications_list);
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { notificationsList, setNotificationsList, loadNotificationsList };
}

function isTotalIndexFromFetch(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'total_notifications_list_index' in (x.content as object) && 'request_key' in (x.content as object);
    }
    return false;
}

function useTotalIndex(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown) {
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [totalIndex, setTotalIndex] = useState(1);

    const loadTotalIndex = () => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "total_notifications_list_index", content: { request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        h.map((_message, i) => {
            if (_message && isTotalIndexFromFetch(_message)) {
                const message = _message as { content: { total_notifications_list_index: number; request_key: string } };
                if (message.content.request_key === requestKey) { setTotalIndex(message.content.total_notifications_list_index); delete h[i]; }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
    }, [websocketMessageHistory, requestKey]);

    return { totalIndex, loadTotalIndex };
}

function useMarkRead(sendJsonMessage: globals.SendJsonMessage) {
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    // notificationId 0 marks every unread notification read.
    const markRead = (notificationId: number) => {
        sendJsonMessage({
            type: "notification_mark_read",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                notification_id: notificationId,
                request_key: nanoid(),
            },
        });
    };

    return { markRead };
}

export default function InfoCenter() {
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);

    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [index, setIndex] = useState(1);

    const { notificationsList, setNotificationsList, loadNotificationsList } = useNotificationsList(sendJsonMessage, lastJsonMessage);
    const { totalIndex, loadTotalIndex } = useTotalIndex(sendJsonMessage, lastJsonMessage);
    const { markRead } = useMarkRead(sendJsonMessage);

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true) {
            loadNotificationsList(index);
            loadTotalIndex();
        }
    }, [loginStatus, index]);

    const openNotification = (item: NotificationItem) => {
        if (!item.is_read) {
            markRead(item.notification_id);
            setNotificationsList((prev) => prev?.map((n) => n.notification_id === item.notification_id ? { ...n, is_read: true } : n));
        }
        navigate(item.target_url);
    };

    const markAllRead = () => {
        markRead(0);
        setNotificationsList((prev) => prev?.map((n) => ({ ...n, is_read: true })));
    };

    return <>
        {
            loginStatus.value &&
            <div style={{ display: "flex", flexDirection: "column", height: "100%", padding: "0.9em 1em 0 1em", boxSizing: "border-box" }}>
                <div style={{ display: "flex", alignItems: "center", columnGap: "0.6em", flexWrap: "wrap", paddingBottom: "0.6em", borderBottom: "1px solid #e0e0e0" }}>
                    <Label>Page {index} / {Math.max(totalIndex, 1)}</Label>
                    <Button appearance="secondary" onClick={() => setIndex((c) => Math.max(1, c - 1))}>Previous</Button>
                    <Button appearance="secondary" onClick={() => setIndex((c) => Math.max(1, Math.min(totalIndex, c + 1)))}>Next</Button>
                    <Button appearance="secondary" onClick={markAllRead} style={{ marginLeft: "auto" }}>Mark all read</Button>
                </div>

                <div className="scroll-box" style={{ flex: 1, minHeight: 0, marginTop: "0.5em" }}>
                {
                    notificationsList === undefined
                        ?
                        <Spinner size="small" label="Loading notifications..." delay={300} />
                        :
                        notificationsList.length === 0
                            ?
                            <Label>No notifications yet.</Label>
                            :
                            <>
                                {
                                    notificationsList.map((item) => (
                                        <div
                                            key={item.notification_id}
                                            onClick={() => openNotification(item)}
                                            style={{
                                                display: "flex",
                                                alignItems: "flex-start",
                                                columnGap: "0.6em",
                                                padding: "0.6em 0.75em",
                                                borderBottom: "1px solid #eee",
                                                cursor: "pointer",
                                                background: item.is_read ? "transparent" : "#EFF6FF",
                                            }}>
                                            <div style={{ marginTop: "0.15em", color: "#4183C4" }}>
                                                {
                                                    item.kind === "mention"
                                                        ? <MentionFilled />
                                                        : item.source_type === "discussion_reply"
                                                            ? <CommentMultipleFilled />
                                                            : <PersonFilled />
                                                }
                                            </div>
                                            <div style={{ flex: 1 }}>
                                                <div style={{ display: "flex", alignItems: "center", columnGap: "0.5em" }}>
                                                    <Label weight="semibold">{item.actor}</Label>
                                                    <Label>{describe(item)}</Label>
                                                    {!item.is_read && <Badge size="extra-small" color="danger" />}
                                                </div>
                                                {
                                                    item.excerpt !== "" &&
                                                    <div style={{ color: "#666", fontSize: "0.9em", marginTop: "0.2em", whiteSpace: "pre-wrap" }}>
                                                        {item.excerpt}
                                                    </div>
                                                }
                                                <div style={{ color: "#999", fontSize: "0.8em", marginTop: "0.2em" }}>
                                                    {formatTimestamp(item.created_at)}
                                                </div>
                                            </div>
                                        </div>
                                    ))
                                }
                            </>
                }
                </div>
            </div>
        }

        <Suspense fallback={<></>}>
            <PopupDialog
                open={dialogRequireLoginOpenState}
                setPopupDialogOpenState={setDialogRequireLoginOpenState}
                text="Please login first."
                onClose={() => navigate("/login")} />
        </Suspense>
    </>;
}
