import { useEffect, useState, useRef, useCallback } from "react";
import { useLoaderData, useNavigate, useOutletContext } from "react-router-dom";

import { Input, Button, Field, Popover, PopoverTrigger, PopoverSurface, Link, Subtitle1 } from "@fluentui/react-components";
import { EmojiRegular } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { EMOJI_SHORTCUTS, replaceEmojiShortcuts } from "../EmojiShortcuts.ts";

import "../css/style.css";

import "../css/chatBubble.css"

interface ChatMessage {
    id?: number;
    message: string;
    fromMe: boolean;
    createdAt?: number;
}

interface ChatMessageFromFetch {
    type: string;
    from: string;
    content: string;
    created_at: number;
}

function isChatMessageFromFetch(x: object) {
    return 'type' in x &&
        'from' in x &&
        'content' in x;
}

interface ChatMessageFromEchoSuccess {
    type: string;
    content: {
        status: number,
        messages: string,
        created_at: number,
    };
}

function isChatMessageFromEchoSuccess(x: object) {
    return "type" in x &&
        "content" in x &&
        typeof x.content === "object" &&
        x.content !== null &&
        "status" in x.content &&
        "messages" in x.content;
}

interface ChatMessageFromEchoFailure {
    type: string;
    content: {
        status: number,
        reason: string,
    };
}

function isChatMessageFromEchoFailure(x: object) {
    return "type" in x &&
        "content" in x &&
        typeof x.content === "object" &&
        x.content !== null &&
        "status" in x.content &&
        "reason" in x.content;
}

interface ChatHistoryResultMessage {
    type: string;
    content: {
        with_username: string;
        messages: { id: number; from: string; content: string; created_at: number }[];
        has_more: boolean;
    };
}

function isChatHistoryResultMessage(x: object) {
    return "type" in x &&
        "content" in x &&
        typeof x.content === "object" &&
        x.content !== null &&
        "with_username" in x.content &&
        "messages" in x.content &&
        "has_more" in x.content;
}

function useChatMessage(
    toUsername: string,
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
) {
    const [chatMessageList, setChatMessageList] = useState<ChatMessage[]>([]);
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [hasMoreHistory, setHasMoreHistory] = useState(true);
    const [initialHistoryLoadToken, setInitialHistoryLoadToken] = useState(0);
    const [newMessageToken, setNewMessageToken] = useState(0);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);
    const loadingHistoryRef = useRef(false);

    const fetchHistory = useCallback((beforeId?: number) => {
        if (loadingHistoryRef.current) return;
        loadingHistoryRef.current = true;
        sendJsonMessage({
            type: "chat_history",
            content: {
                username: loginUsername.value,
                with_username: toUsername,
                session_token: sessionToken.value,
                before_id: beforeId,
                request_key: nanoid(),
            }
        });
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [toUsername, loginUsername.value, sessionToken.value, sendJsonMessage]);

    // Reset and load the most recent 10 messages whenever the conversation partner changes.
    useEffect(() => {
        setChatMessageList([]);
        setHasMoreHistory(true);
        loadingHistoryRef.current = false;
        fetchHistory(undefined);
    }, [toUsername, fetchHistory]);

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((previousMessage) => previousMessage.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        let changed = false;
        const newChatMessageList: ChatMessage[] = [];
        let historyPage: ChatMessage[] | null = null;
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, index) => {
            if (_message) {
                if (isChatHistoryResultMessage(_message)) {
                    const message = _message as ChatHistoryResultMessage;
                    if (message.type === "chat_history_result" && message.content.with_username === toUsername) {
                        loadingHistoryRef.current = false;
                        setHasMoreHistory(message.content.has_more);
                        historyPage = message.content.messages.map((m) => ({
                            id: m.id,
                            message: m.content,
                            fromMe: m.from === loginUsername.value,
                            createdAt: m.created_at,
                        }));
                        delete _websocketMessageHistory[index];
                    }
                }

                if (isChatMessageFromFetch(_message)) {
                    const message = _message as ChatMessageFromFetch;
                    if (message.type === "chat_message" && message.from === toUsername) {
                        changed = true;
                        newChatMessageList.push(({ message: (message.content as string), fromMe: false, createdAt: message.created_at } as ChatMessage));
                        delete _websocketMessageHistory[index];
                    }
                }

                if (isChatMessageFromEchoFailure(_message)) {
                    const message = _message as ChatMessageFromEchoFailure;
                    if (message.type === "chat_echo" && message.content.status === 0) {
                        changed = true;
                        delete _websocketMessageHistory[index];
                    }
                }

                if (isChatMessageFromEchoSuccess(_message)) {
                    const message = _message as ChatMessageFromEchoSuccess;
                    if (message.type === "chat_echo" && message.content.status === 1) {
                        changed = true;
                        newChatMessageList.push(({ message: (message.content.messages as string), fromMe: true, createdAt: message.content.created_at } as ChatMessage));
                        delete _websocketMessageHistory[index];
                    }
                }
            }
        });

        if (historyPage !== null) {
            const isInitialPage = chatMessageList.length === 0;
            setChatMessageList((previous) => (historyPage as ChatMessage[]).concat(previous));
            if (isInitialPage) setInitialHistoryLoadToken((x) => x + 1);
        } else if (changed) {
            setChatMessageList((previous) => previous.concat(newChatMessageList));
            // A live message (sent or received), not a history page — signal so the view can
            // jump to the bottom to follow it.
            setNewMessageToken((x) => x + 1);
        }
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, toUsername, loginUsername.value, chatMessageList.length]);

    const loadMoreHistory = () => {
        if (!hasMoreHistory || loadingHistoryRef.current) return;
        const earliest = chatMessageList.find((m) => typeof m.id === "number");
        if (earliest === undefined) return;
        fetchHistory(earliest.id);
    };

    const sendChatMessage = (chatMessageToSend: string) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "chat_user",
            content: {
                from: loginUsername.value,
                to: toUsername,
                messages: chatMessageToSend as string,
                session_token: sessionToken.value,
                request_key: _requestKey,
            }
        });
    };

    return {
        chatMessageList,
        hasMoreHistory,
        initialHistoryLoadToken,
        newMessageToken,
        loadMoreHistory,
        sendChatMessage
    };
}

interface ChatInfo {
    toUsername: string;
}

export default function ChatMainUser() {
    const { toUsername } = (useLoaderData() as ChatInfo);
    const navigate = useNavigate();
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const { chatMessageList, hasMoreHistory, initialHistoryLoadToken, newMessageToken, loadMoreHistory, sendChatMessage } = useChatMessage(
        toUsername,
        sendJsonMessage,
        lastJsonMessage
    );
    const [chatMessageToSend, setChatMessageToSend] = useState("");
    const scrollBoxRef = useRef<HTMLDivElement | null>(null);
    const pendingScrollHeightRef = useRef<number | null>(null);

    // After the very first page of history loads for this conversation, jump to the bottom so
    // the most recent (last ten) messages are what's visible, instead of the oldest of the page.
    useEffect(() => {
        const el = scrollBoxRef.current;
        if (el && initialHistoryLoadToken > 0) el.scrollTop = el.scrollHeight;
    }, [initialHistoryLoadToken]);

    // A new message (sent or received) just arrived — smoothly scroll down to follow it.
    useEffect(() => {
        const el = scrollBoxRef.current;
        if (el && newMessageToken > 0) el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
    }, [newMessageToken]);

    // After an older page gets prepended (from upscrolling), restore the scroll position so the
    // view doesn't visually jump to follow the newly-inserted content above it.
    useEffect(() => {
        const el = scrollBoxRef.current;
        if (el && pendingScrollHeightRef.current !== null) {
            el.scrollTop += el.scrollHeight - pendingScrollHeightRef.current;
            pendingScrollHeightRef.current = null;
        }
    }, [chatMessageList]);

    const handleScroll = () => {
        const el = scrollBoxRef.current;
        if (!el || !hasMoreHistory) return;
        if (el.scrollTop < 50) {
            pendingScrollHeightRef.current = el.scrollHeight;
            loadMoreHistory();
        }
    };

    const handleClickSendChatMessage = () => {
        sendChatMessage(replaceEmojiShortcuts(chatMessageToSend));
        setChatMessageToSend("");
    };

    const handlePickEmoji = (emoji: string) => {
        setChatMessageToSend((previous) => previous + emoji);
    };

    return <>
        <div style={{ display: "block", height: "80%" }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", margin: "0 0.5em" }}>
                <Subtitle1>{toUsername}</Subtitle1>
                <Link onClick={() => navigate("/user/" + toUsername)}>View Profile</Link>
            </div>
            <div className="scroll-bar-wrap">
                <div
                    ref={scrollBoxRef}
                    onScroll={handleScroll}
                    style={{ display: "block", height: "400px", overflowY: "auto", marginTop: "1em" }}
                    className="scroll-box">
                    <div style={{ display: "flex", flexDirection: "column", flexWrap: "wrap", width: "fill", marginRight: "1em" }}>
                        {
                            chatMessageList == null
                                ?
                                <></>
                                :
                                chatMessageList.map((message, index) => <div key={message.id ?? "live-" + index} style={{ alignItems: message.fromMe ? "flex-end" : "flex-start", display: "flex", flexDirection: "column", width: "fill", minHeight: "50px" }}>
                                    <span style={{ fontSize: "0.75em", color: "#888", margin: "0 0.3em 0.15em" }}>{formatChatTimestamp(message.createdAt)}</span>
                                    <ChatBubble text={message.message} fromMe={message.fromMe} />
                                </div>)
                        }
                    </div>
                </div>
                <div className="cover-bar" />
            </div>
            <div style={{ display: "flex", flexDirection: "row", width: "fill", alignItems: "end" }}>
                <form>
                    <Field label="Input to chat" style={{ maxWidth: "300px", flex: 3 }}>
                        <Input type="text" id="chat-input" value={chatMessageToSend} onChange={(props) => setChatMessageToSend(props.target.value)} />
                    </Field>
                    <Popover>
                        <PopoverTrigger disableButtonEnhancement>
                            <Button icon={<EmojiRegular />} style={{ flex: 1 }} />
                        </PopoverTrigger>
                        <PopoverSurface>
                            <div style={{ display: "flex", flexWrap: "wrap", maxWidth: "240px" }}>
                                {
                                    Object.entries(EMOJI_SHORTCUTS).map(([shortcut, emoji]) => (
                                        <Button
                                            key={shortcut}
                                            appearance="subtle"
                                            title={shortcut}
                                            onClick={() => handlePickEmoji(emoji)}
                                            style={{ fontSize: "1.2em", minWidth: "2em" }}>
                                            {emoji}
                                        </Button>
                                    ))
                                }
                            </div>
                        </PopoverSurface>
                    </Popover>
                    <Button onClick={handleClickSendChatMessage} style={{ flex: 1 }} appearance="primary">Send</Button>
                </form>
            </div>
        </div>
    </>;
}

function formatChatTimestamp(createdAt?: number): string {
    if (createdAt === undefined) return "";
    const date = new Date(createdAt);
    const now = new Date();
    const isToday = date.toDateString() === now.toDateString();
    const time = date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    if (isToday) return time;
    return date.toLocaleDateString([], { month: "short", day: "numeric" }) + " " + time;
}

function ChatBubble({ text, fromMe }: {
    text: string;
    fromMe: boolean;
}) {
    if (fromMe)
        return <div className="chat-bubble chat-bubble--me">
            <p style={{ fontSize: "1em", color: "black" }}>{text}</p>
        </div>;
    else
        return <div className="chat-bubble chat-bubble--not-me">
            <p style={{ fontSize: "1em", color: "white" }}>{text}</p>
        </div>;
}
