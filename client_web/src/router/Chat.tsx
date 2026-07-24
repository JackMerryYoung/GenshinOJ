import { useEffect, useState, lazy } from "react";
import { useNavigate, Outlet, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import {
    makeStyles,
    Table,
    TableHeader,
    TableRow,
    TableHeaderCell,
    TableCell,
    TableBody,
    Divider,
    Spinner,
    Label,
    Input,
} from "@fluentui/react-components";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";

import "../css/style.css";

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "row",
        rowGap: "4px",
        columnGap: "0.3em",
        marginLeft: "0.3em"
    },
    chat_list: {
        width: "calc((100vw - 30.2px) * 0.15)",
    },
    divider: {
        width: "calc((100vw - 30.2px) * 0.01)",
    },
    chat_main_outlet: {
        width: "calc((100vw - 30.2px) * 0.84)",
    }
});

interface FriendsListMessage {
    type: string;
    content: {
        friends: string[],
        request_key: string,
    };
}

function isFriendsListMessage(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'friends' in (x.content as object) &&
            'request_key' in (x.content as object);
    }

    return false;
}

function useFriendsList(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    loginUsername: string
) {
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [friendsList, setFriendsList] = useState<string[] | undefined>(undefined);
    const [requestKey, setRequestKey] = useState("");

    const fetchFriendsList = () => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "friends_list",
            content: {
                username: loginUsername,
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
        let changed = false;
        let newFriendsList: string[] = [];
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, index) => {
            if (_message && isFriendsListMessage(_message)) {
                const message = _message as FriendsListMessage;
                if (message.type === 'friends_list' && message.content.request_key === requestKey) {
                    changed = true;
                    newFriendsList = message.content.friends;
                    delete _websocketMessageHistory[index];
                }
            }
        });

        if (changed) setFriendsList(newFriendsList);
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { friendsList, fetchFriendsList };
}

interface SearchUsersResultMessage {
    type: string;
    content: {
        users: string[],
        request_key: string,
    };
}

function isSearchUsersResultMessage(x: object) {
    if ('type' in x && 'content' in x && typeof x.content === 'object') {
        return 'users' in (x.content as object) &&
            'request_key' in (x.content as object);
    }

    return false;
}

function useSearchUsers(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    loginUsername: string
) {
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [searchResults, setSearchResults] = useState<string[] | undefined>(undefined);
    const [requestKey, setRequestKey] = useState("");

    const searchUsers = (query: string) => {
        if (query.trim() === "") {
            setSearchResults(undefined);
            return;
        }
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "search_users",
            content: {
                query,
                exclude_username: loginUsername,
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
        let changed = false;
        let newSearchResults: string[] = [];
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, index) => {
            if (_message && isSearchUsersResultMessage(_message)) {
                const message = _message as SearchUsersResultMessage;
                if (message.type === 'search_users_result' && message.content.request_key === requestKey) {
                    changed = true;
                    newSearchResults = message.content.users;
                    delete _websocketMessageHistory[index];
                }
            }
        });

        if (changed) setSearchResults(newSearchResults);
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { searchResults, searchUsers };
}

function UserRow({ username }: { username: string }) {
    const navigate = useNavigate();
    return <TableRow key={username}>
        <TableCell onClick={() => navigate("/chat/user/" + username)}>{username}</TableCell>
    </TableRow>;
}

export function ChatList({ sendJsonMessage, lastJsonMessage }: {
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
}) {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const { friendsList, fetchFriendsList } = useFriendsList(sendJsonMessage, lastJsonMessage, loginUsername.value);
    const { searchResults, searchUsers } = useSearchUsers(sendJsonMessage, lastJsonMessage, loginUsername.value);
    const [searchQuery, setSearchQuery] = useState("");
    const { t } = useTranslation("chat");

    useEffect(() => {
        if (loginStatus.value === true) fetchFriendsList();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [loginStatus]);

    useEffect(() => {
        searchUsers(searchQuery);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [searchQuery]);

    return <div className="scroll-bar-wrap">
        <div className="scroll-box" style={{ display: "block", overflowY: "auto", maxHeight: "60vh", marginTop: "0.5em" }}>
            <div style={{ margin: "0.4em" }}>
                <Input
                    placeholder={t("searchUsersPlaceholder")}
                    value={searchQuery}
                    onChange={(_ev, data) => setSearchQuery(data.value)} />
            </div>
            {
                searchQuery.trim() !== "" &&
                <Table size="medium">
                    <TableHeader style={{ position: "sticky", top: 0, zIndex: 1, background: "#fff" }}>
                        <TableRow>
                            <TableHeaderCell>{t("searchResults")}</TableHeaderCell>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {
                            searchResults === undefined
                                ?
                                <TableRow>
                                    <TableCell><Spinner size="tiny" label={t("searching")} delay={300} /></TableCell>
                                </TableRow>
                                :
                                searchResults.length === 0
                                    ?
                                    <TableRow>
                                        <TableCell><Label>{t("noUsersFound")}</Label></TableCell>
                                    </TableRow>
                                    :
                                    searchResults.map((username: string) => <UserRow key={username} username={username} />)
                        }
                    </TableBody>
                </Table>
            }
            <Table size="medium">
                <TableHeader style={{ position: "sticky", top: 0, zIndex: 1, background: "#fff" }}>
                    <TableRow>
                        <TableHeaderCell>{t("friends")}</TableHeaderCell>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {
                        friendsList === undefined
                            ?
                            <TableRow>
                                <TableCell><Spinner size="large" label={t("waiting")} delay={500} /></TableCell>
                            </TableRow>
                            :
                            friendsList.length === 0
                                ?
                                <TableRow>
                                    <TableCell><Label>{t("noFriendsYet")}</Label></TableCell>
                                </TableRow>
                                :
                                friendsList.map((username: string) => <UserRow key={username} username={username} />)
                    }
                </TableBody>
            </Table>
        </div>
    </div>;
}

export default function Chat() {
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const navigate = useNavigate();
    const style = useStyles();
    const { t } = useTranslation("chat");

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    return <>
        {
            loginStatus.value && (
                <div className={style.root}>
                    <div className={style.chat_list}>
                        <ChatList
                            sendJsonMessage={sendJsonMessage}
                            lastJsonMessage={lastJsonMessage} />
                    </div>
                    <div className={style.divider}>
                        <Divider vertical style={{ height: "calc(100vh - 8.8em)" }} />
                    </div>
                    <div className={style.chat_main_outlet}>
                        <Outlet context={{ sendJsonMessage, lastJsonMessage }} />
                    </div>
                </div>
            )
        }

        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text={t("pleaseLoginFirst")}
            onClose={() => navigate("/login")} />
    </>;
}
