import { useEffect, useRef, useState, lazy } from "react";
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

function useFriendsList(
    request: globals.WebSocketRequest,
    loginUsername: string
) {
    const [friendsList, setFriendsList] = useState<string[] | undefined>(undefined);

    const fetchFriendsList = () => {
        request<{
            type: "friends_list";
            content: { friends: string[]; request_key: string };
        }>("friends_list", { username: loginUsername }, { responseTypes: ["friends_list"] })
            .then((response) => setFriendsList(response.content.friends))
            .catch((error) => console.error("Failed to load friends", error));
    };

    return { friendsList, fetchFriendsList };
}

function useSearchUsers(
    request: globals.WebSocketRequest,
    loginUsername: string
) {
    const [searchResults, setSearchResults] = useState<string[] | undefined>(undefined);
    const searchAbortRef = useRef<AbortController | null>(null);

    const searchUsers = (query: string) => {
        searchAbortRef.current?.abort();
        if (query.trim() === "") {
            setSearchResults(undefined);
            return;
        }
        const controller = new AbortController();
        searchAbortRef.current = controller;
        request<{
            type: "search_users_result";
            content: { users: string[]; request_key: string };
        }>("search_users", {
                query,
                exclude_username: loginUsername,
            }, { responseTypes: ["search_users_result"], signal: controller.signal })
            .then((response) => setSearchResults(response.content.users))
            .catch((error) => {
                if (!controller.signal.aborted) console.error("Failed to search users", error);
            });
    };

    return { searchResults, searchUsers };
}

function UserRow({ username }: { username: string }) {
    const navigate = useNavigate();
    return <TableRow key={username}>
        <TableCell onClick={() => navigate("/chat/user/" + username)}>{username}</TableCell>
    </TableRow>;
}

export function ChatList({ request }: {
    request: globals.WebSocketRequest;
}) {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const { friendsList, fetchFriendsList } = useFriendsList(request, loginUsername.value);
    const { searchResults, searchUsers } = useSearchUsers(request, loginUsername.value);
    const [searchQuery, setSearchQuery] = useState("");
    const { t } = useTranslation("chat");

    useEffect(() => {
        if (loginStatus.value === true) fetchFriendsList();
    }, [loginStatus]);

    useEffect(() => {
        searchUsers(searchQuery);
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
    const { request, sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const navigate = useNavigate();
    const style = useStyles();
    const { t } = useTranslation("chat");

    useEffect(() => {
        setDialogRequireLoginOpenState(loginStatus.value === false);
    }, [loginStatus]);

    return <>
        {
            loginStatus.value && (
                <div className={style.root}>
                    <div className={style.chat_list}>
                        <ChatList
                            request={request} />
                    </div>
                    <div className={style.divider}>
                        <Divider vertical style={{ height: "calc(100vh - 8.8em)" }} />
                    </div>
                    <div className={style.chat_main_outlet}>
                        <Outlet context={{ request, sendJsonMessage, lastJsonMessage }} />
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
