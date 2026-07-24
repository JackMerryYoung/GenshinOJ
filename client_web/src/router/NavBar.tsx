import { useEffect, useState } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { Divider, Avatar, Tab, CounterBadge, Dropdown, Option } from "@fluentui/react-components";
import { ChartMultipleFilled, ChatFilled, ClipboardTaskListLtrFilled, CommentMultipleFilled, AlertFilled, PersonFilled, PersonAddFilled, SignOutFilled, ArrowEnterFilled } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

import { RootState } from "../store";
import * as globals from "../Globals.ts";

import "../css/style.css";

const LANGUAGE_OPTIONS = [
    { value: "en", labelKey: "language.en" },
    { value: "zh-CN", labelKey: "language.zhCN" },
] as const;

// Track the logged-in user's unread notification count for the navbar badge. Notifications are
// pull-based (the judge knows the actor's ws_id, not the recipient's), so this polls on a timer and
// on navigation, and also refreshes immediately whenever a `notification_mark_read_result` flows by
// (e.g. the Info Center just marked something read on this same shared socket).
function useUnreadCount(sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown, loggedIn: boolean) {
    const [unreadCount, setUnreadCount] = useState(0);
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);

    const refresh = () => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "notifications_unread_count", content: { request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((p) => p.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const h = websocketMessageHistory;
        let sawMarkRead = false;
        h.map((_message, i) => {
            if (_message && typeof _message === 'object' && 'type' in _message) {
                const type = (_message as { type: string }).type;
                if (type === "notifications_unread_count") {
                    const message = _message as { content: { unread_count: number; request_key: string } };
                    if (message.content.request_key === requestKey) { setUnreadCount(message.content.unread_count); delete h[i]; }
                } else if (type === "notification_mark_read_result") {
                    sawMarkRead = true;
                    delete h[i];
                }
            }
        });
        if (!globals.compareArray(h, websocketMessageHistory)) setWebsocketMessageHistory(h);
        if (sawMarkRead && loggedIn) refresh();
    }, [websocketMessageHistory, requestKey]);

    useEffect(() => {
        if (!loggedIn) { setUnreadCount(0); return; }
        refresh();
        const id = setInterval(refresh, 20000);
        return () => clearInterval(id);
    }, [loggedIn]);

    return { unreadCount, refresh };
}

export default function NavBar({ sendJsonMessage, lastJsonMessage }: {
    sendJsonMessage: globals.SendJsonMessage;
    lastJsonMessage: unknown;
}) {
    const { t, i18n } = useTranslation(["navBar", "common"]);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const navigate = useNavigate();
    const location = useLocation();
    const loggedIn = loginStatus.value === true;

    const { unreadCount, refresh } = useUnreadCount(sendJsonMessage, lastJsonMessage, loggedIn);

    // Re-check the unread count whenever the route changes (cheap, and keeps the badge fresh right
    // after the user reads notifications without waiting for the next poll tick).
    useEffect(() => {
        if (loggedIn) refresh();
    }, [location.pathname]);

    const onTabSelect = (value: string) => {
        if (value === "home") navigate("/home");
        if (value === "problem") navigate("/problem");
        if (value === "submission") navigate("/submission");
        if (value === "discussion") navigate("/discussion");
        if (value === "chat") navigate("/chat");
        if (value === "notification") navigate("/notification");
        if (value === "user") navigate("/user");
        if (value === "login") navigate("/login");
        if (value === "register") navigate("/register");
        if (value === "logout") navigate("/logout");
    };

    return (
        <div style={{ padding: "0.35em 0" }}>
            <div style={{ display: "inline" }}>
                <Tab onClick={() => onTabSelect("home")} style={{ float: "left" }} value="home" icon={<Avatar size={24} image={{ src: "https://img.atcoder.jp/icons/373e4eb93e4b8e5f441eeeea55e5ac84.jpg" }} />}>
                    RsOJ
                </Tab>
                {
                    loggedIn &&
                    <>
                        <Tab onClick={() => onTabSelect("problem")} style={{ float: "left" }} value="problem" icon={<ClipboardTaskListLtrFilled />}>{t("tab.problem")}</Tab>
                        <Tab onClick={() => onTabSelect("submission")} style={{ float: "left" }} value="submission" icon={<ChartMultipleFilled />}>{t("tab.submission")}</Tab>
                        <Tab onClick={() => onTabSelect("discussion")} style={{ float: "left" }} value="discussion" icon={<CommentMultipleFilled />}>{t("tab.discuss")}</Tab>
                        <Tab onClick={() => onTabSelect("notification")} style={{ float: "left" }} value="notification" icon={<AlertFilled />}>
                            {t("tab.notification")} {unreadCount > 0 && <CounterBadge count={unreadCount} size="small" color="danger" />}
                        </Tab>
                    </>
                }
                {
                    loggedIn
                        ?
                        // float:right stacks right-to-left in DOM order, so this renders as: Chat | User | Sign out.
                        <>
                            <Tab onClick={() => onTabSelect("logout")} style={{ float: "right" }} value="logout" icon={<SignOutFilled />}>{t("tab.signOut")}</Tab>
                            <Tab onClick={() => onTabSelect("user")} style={{ float: "right" }} value="user" icon={<PersonFilled />}>{t("tab.user")}</Tab>
                            <Tab onClick={() => onTabSelect("chat")} style={{ float: "right" }} value="chat" icon={<ChatFilled />}>{t("tab.chat")}</Tab>
                        </>
                        :
                        <>
                            <Tab onClick={() => onTabSelect("login")} style={{ float: "right" }} value="login" icon={<ArrowEnterFilled />}>{t("tab.signIn")}</Tab>
                            <Tab onClick={() => onTabSelect("register")} style={{ float: "right" }} value="register" icon={<PersonAddFilled />}>{t("tab.signUp")}</Tab>
                        </>
                }
                <Dropdown
                    style={{ float: "right", minWidth: "120px" }}
                    value={LANGUAGE_OPTIONS.find((option) => option.value === i18n.resolvedLanguage)?.value ?? "en"}
                    selectedOptions={[i18n.resolvedLanguage ?? "en"]}
                    onOptionSelect={(_event, data) => {
                        if (data.optionValue) i18n.changeLanguage(data.optionValue);
                    }}
                >
                    {LANGUAGE_OPTIONS.map((option) => (
                        <Option key={option.value} value={option.value}>
                            {t(option.labelKey, { ns: "common" })}
                        </Option>
                    ))}
                </Dropdown>

                <Divider />
            </div>
        </div>
    );
}
