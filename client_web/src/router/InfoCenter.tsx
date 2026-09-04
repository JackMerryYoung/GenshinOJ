import { useEffect, useOptimistic, useRef, useState, startTransition, lazy, Suspense } from "react";
import { useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { TFunction } from "i18next";

import {
    Button,
    Label,
    Spinner,
    Badge,
} from "@fluentui/react-components";
import { PersonFilled, CommentMultipleFilled, MentionFilled } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

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
function describe(item: NotificationItem, t: TFunction) {
    if (item.kind === "mention") {
        if (item.source_type === "solution") return t("notification.mentionedInSolution");
        if (item.source_type === "solution_comment") return t("notification.mentionedInComment");
        if (item.source_type === "discussion") return t("notification.mentionedInDiscussion");
        if (item.source_type === "discussion_reply") return t("notification.mentionedInReply");
        return t("notification.mentionedYou");
    }
    // kind === "reply"
    if (item.source_type === "solution_comment") return t("notification.commentedOnSolution");
    if (item.source_type === "discussion_reply") return t("notification.repliedToDiscussion");
    return t("notification.repliedToYou");
}

function useNotificationsList(request: globals.WebSocketRequest) {
    const [notificationsList, setNotificationsList] = useState<NotificationItem[] | undefined>(undefined);
    const requestAbortRef = useRef<AbortController | null>(null);

    const loadNotificationsList = (index: number) => {
        requestAbortRef.current?.abort();
        const controller = new AbortController();
        requestAbortRef.current = controller;
        setNotificationsList(undefined);
        request<{
            type: "notifications_list";
            content: { notifications_list: NotificationItem[]; request_key: string };
        }>("notifications_list", { index }, { responseTypes: ["notifications_list"], signal: controller.signal })
            .then((response) => setNotificationsList(response.content.notifications_list))
            .catch((error) => {
                if (!controller.signal.aborted) console.error("Failed to load notifications", error);
            });
    };

    return { notificationsList, setNotificationsList, loadNotificationsList };
}

function useTotalIndex(request: globals.WebSocketRequest) {
    const [totalIndex, setTotalIndex] = useState(1);
    const requestAbortRef = useRef<AbortController | null>(null);

    const loadTotalIndex = () => {
        requestAbortRef.current?.abort();
        const controller = new AbortController();
        requestAbortRef.current = controller;
        request<{
            type: "total_notifications_list_index";
            content: { total_notifications_list_index: number; request_key: string };
        }>("total_notifications_list_index", {}, { responseTypes: ["total_notifications_list_index"], signal: controller.signal })
            .then((response) => setTotalIndex(response.content.total_notifications_list_index))
            .catch((error) => {
                if (!controller.signal.aborted) console.error("Failed to load notification page count", error);
            });
    };

    return { totalIndex, loadTotalIndex };
}

function useMarkRead(request: globals.WebSocketRequest) {
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    // notificationId 0 marks every unread notification read.
    const markRead = (notificationId: number) => request<{
        type: "notification_mark_read_result";
        content: { request_key: string; result: string };
    }>("notification_mark_read", {
                username: loginUsername.value,
                session_token: sessionToken.value,
                notification_id: notificationId,
            }, { responseTypes: ["notification_mark_read_result"] });

    return { markRead };
}

export default function InfoCenter() {
    const { t } = useTranslation("infoCenter");
    const { request } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);

    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const [index, setIndex] = useState(1);

    const { notificationsList, setNotificationsList, loadNotificationsList } = useNotificationsList(request);
    const { totalIndex, loadTotalIndex } = useTotalIndex(request);
    const { markRead } = useMarkRead(request);
    const [optimisticNotifications, applyOptimistic] = useOptimistic(
        notificationsList,
        (current: NotificationItem[] | undefined, update: { notificationId: number }) => {
            if (!current) return current;
            return current.map((item) => update.notificationId === 0 || item.notification_id === update.notificationId
                ? { ...item, is_read: true }
                : item);
        },
    );

    useEffect(() => {
        setDialogRequireLoginOpenState(loginStatus.value === false);
    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true) {
            loadNotificationsList(index);
            loadTotalIndex();
        }
    }, [loginStatus, index]);

    const openNotification = (item: NotificationItem) => {
        if (!item.is_read) {
            startTransition(async () => {
                applyOptimistic({ notificationId: item.notification_id });
                try {
                    const response = await markRead(item.notification_id);
                    if (response.content.result !== "ok") throw new Error(response.content.result);
                    setNotificationsList((prev) => prev?.map((n) => n.notification_id === item.notification_id ? { ...n, is_read: true } : n));
                } catch {
                    loadNotificationsList(index);
                }
            });
        }
        navigate(item.target_url);
    };

    const markAllRead = () => {
        startTransition(async () => {
            applyOptimistic({ notificationId: 0 });
            try {
                const response = await markRead(0);
                if (response.content.result !== "ok") throw new Error(response.content.result);
                setNotificationsList((prev) => prev?.map((n) => ({ ...n, is_read: true })));
            } catch {
                loadNotificationsList(index);
            }
        });
    };

    return <>
        {
            loginStatus.value &&
            <div style={{ display: "flex", flexDirection: "column", height: "100%", padding: "0.9em 1em 0 1em", boxSizing: "border-box" }}>
                <div style={{ display: "flex", alignItems: "center", columnGap: "0.6em", flexWrap: "wrap", paddingBottom: "0.6em", borderBottom: "1px solid #e0e0e0" }}>
                    <Label>{t("pagination.page", { index, total: Math.max(totalIndex, 1) })}</Label>
                    <Button appearance="secondary" onClick={() => setIndex((c) => Math.max(1, c - 1))}>{t("pagination.previous")}</Button>
                    <Button appearance="secondary" onClick={() => setIndex((c) => Math.max(1, Math.min(totalIndex, c + 1)))}>{t("pagination.next")}</Button>
                    <Button appearance="secondary" onClick={markAllRead} style={{ marginLeft: "auto" }}>{t("action.markAllRead")}</Button>
                </div>

                <div className="scroll-box" style={{ flex: 1, minHeight: 0, marginTop: "0.5em" }}>
                {
                    optimisticNotifications === undefined
                        ?
                        <Spinner size="small" label={t("loading.notifications")} delay={300} />
                        :
                        optimisticNotifications.length === 0
                            ?
                            <Label>{t("empty.noNotifications")}</Label>
                            :
                            <>
                                {
                                    optimisticNotifications.map((item) => (
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
                                                    <Label>{describe(item, t)}</Label>
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
                text={t("dialog.pleaseLoginFirst")}
                onClose={() => navigate("/login")} />
        </Suspense>
    </>;
}
