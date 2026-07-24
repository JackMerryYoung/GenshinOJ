import { useEffect, useRef, useState } from "react";
import type { CSSProperties, KeyboardEvent, MouseEvent } from "react";

import { Textarea } from "@fluentui/react-components";

import { nanoid } from "nanoid";

import * as globals from "../Globals.ts";

interface MentionTextareaProps {
    value: string;
    onChange: (value: string) => void;
    sendJsonMessage: globals.SendJsonMessage;
    lastJsonMessage: unknown;
    placeholder?: string;
    rows?: number;
    style?: CSSProperties;
}

// The `@token` immediately to the left of the caret (without the `@`), or null if the caret isn't
// currently inside a mention. A mention starts at the beginning of the text or after whitespace.
function detectMentionPrefix(text: string, caret: number): string | null {
    const before = text.slice(0, caret);
    const match = before.match(/(?:^|\s)@([A-Za-z0-9_]*)$/);
    return match ? match[1] : null;
}

// A Textarea with @mention autocomplete: typing `@` opens a username suggestion list (backed by the
// judge `user_search` endpoint); picking one inserts `@username `. Drop-in replacement for the plain
// Fluent Textarea used in the post / comment / reply forms.
export default function MentionTextarea({
    value,
    onChange,
    sendJsonMessage,
    lastJsonMessage,
    placeholder,
    rows,
    style,
}: MentionTextareaProps) {
    const textareaRef = useRef<HTMLTextAreaElement | null>(null);
    const [caret, setCaret] = useState(0);
    const [mentionActive, setMentionActive] = useState(false);
    const [suggestions, setSuggestions] = useState<string[]>([]);
    const [highlight, setHighlight] = useState(0);
    const [requestKey, setRequestKey] = useState("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);

    const queryUsers = (prefix: string) => {
        const _requestKey = nanoid();
        sendJsonMessage({ type: "user_search", content: { prefix, request_key: _requestKey } });
        setRequestKey(_requestKey);
    };

    const refresh = (nextValue: string, caretPos: number) => {
        setCaret(caretPos);
        const prefix = detectMentionPrefix(nextValue, caretPos);
        if (prefix === null) {
            setMentionActive(false);
            setSuggestions([]);
        } else {
            setMentionActive(true);
            setHighlight(0);
            queryUsers(prefix);
        }
    };

    useEffect(() => {
        if (lastJsonMessage !== null)
            setWebsocketMessageHistory((previousMessageHistory) => previousMessageHistory.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, _index) => {
            if (_message && typeof _message === 'object' && 'type' in _message && (_message as { type: string }).type === "user_search") {
                const message = _message as { content: { usernames: string[]; request_key: string } };
                if (message.content.request_key === requestKey) {
                    setSuggestions(message.content.usernames);
                    delete _websocketMessageHistory[_index];
                }
            }
        });
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    const applySuggestion = (username: string) => {
        const before = value.slice(0, caret).replace(/(^|\s)@([A-Za-z0-9_]*)$/, (_match, pre) => `${pre}@${username} `);
        const after = value.slice(caret);
        onChange(before + after);
        setMentionActive(false);
        setSuggestions([]);
        const nextCaret = before.length;
        requestAnimationFrame(() => {
            const element = textareaRef.current;
            if (element) {
                element.focus();
                element.setSelectionRange(nextCaret, nextCaret);
            }
            setCaret(nextCaret);
        });
    };

    const open = mentionActive && suggestions.length > 0;

    const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
        if (!open) return;
        if (event.key === "ArrowDown") {
            event.preventDefault();
            setHighlight((current) => Math.min(suggestions.length - 1, current + 1));
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            setHighlight((current) => Math.max(0, current - 1));
        } else if (event.key === "Enter") {
            event.preventDefault();
            applySuggestion(suggestions[highlight]);
        } else if (event.key === "Escape") {
            event.preventDefault();
            setMentionActive(false);
            setSuggestions([]);
        }
    };

    const syncCaret = (event: MouseEvent<HTMLTextAreaElement> | KeyboardEvent<HTMLTextAreaElement>) => {
        const element = event.target as HTMLTextAreaElement;
        setCaret(element.selectionStart ?? 0);
    };

    // Full width: this wrapper div sits between Fluent's <Field> and the <Textarea>, so Field's
    // automatic full-width styling lands on the div and never reaches the nested Textarea. Stretch
    // both explicitly so the control fills the field like the plain Input above it.
    return <div style={{ position: "relative", width: "100%" }}>
        <Textarea
            style={{ width: "100%", ...style }}
            placeholder={placeholder}
            value={value}
            onChange={(event, data) => {
                const element = event.target as HTMLTextAreaElement;
                onChange(data.value);
                refresh(data.value, element.selectionStart ?? data.value.length);
            }}
            resize="vertical"
            rows={rows}
            textarea={{ ref: textareaRef, onKeyDown, onClick: syncCaret, onKeyUp: syncCaret }} />
        {
            open &&
            <div style={{
                position: "absolute",
                top: "100%",
                left: 0,
                zIndex: 1000,
                background: "#fff",
                border: "1px solid #ccc",
                borderRadius: "4px",
                boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
                minWidth: "12em",
                maxHeight: "12em",
                overflowY: "auto",
            }}>
                {
                    suggestions.map((username, index) => (
                        <div
                            key={username}
                            onMouseDown={(event) => { event.preventDefault(); applySuggestion(username); }}
                            onMouseEnter={() => setHighlight(index)}
                            style={{
                                padding: "0.35em 0.75em",
                                cursor: "pointer",
                                background: index === highlight ? "#f0f0f0" : "transparent",
                            }}>
                            @{username}
                        </div>
                    ))
                }
            </div>
        }
    </div>;
}
