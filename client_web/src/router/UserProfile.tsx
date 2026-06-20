import { useEffect, useState, lazy, useRef } from "react";

import { useNavigate, useOutletContext, useParams } from "react-router-dom";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { Button, Label, Avatar } from "@fluentui/react-components";

const MAX_AVATAR_SIZE_BYTES = 1024 * 1024;

function AvatarUploader({ username, sessionToken, onUploaded }: {
    username: string;
    sessionToken: string;
    onUploaded: () => void;
}) {
    const [error, setError] = useState<string | undefined>(undefined);
    const [uploading, setUploading] = useState(false);
    const fileInputRef = useRef<HTMLInputElement | null>(null);

    const handleFileChange = async (ev: React.ChangeEvent<HTMLInputElement>) => {
        const file = ev.target.files?.[0];
        ev.target.value = "";
        if (!file) return;

        if (file.size > MAX_AVATAR_SIZE_BYTES) {
            setError("File exceeds the 1MB limit.");
            return;
        }

        setUploading(true);
        setError(undefined);

        const formData = new FormData();
        formData.append("username", username);
        formData.append("session_token", sessionToken);
        formData.append("file", file);

        try {
            const response = await fetch("/avatar/upload", { method: "POST", body: formData });
            if (!response.ok) {
                setError(await response.text());
            } else {
                onUploaded();
            }
        } catch {
            setError("Failed to upload avatar.");
        } finally {
            setUploading(false);
        }
    };

    return <div style={{ display: "flex", flexDirection: "column", margin: "0 0.4em" }}>
        <input
            ref={fileInputRef}
            type="file"
            accept="image/png,image/jpeg,image/gif,image/webp"
            style={{ display: "none" }}
            onChange={handleFileChange} />
        <Button
            appearance="secondary"
            disabled={uploading}
            onClick={() => fileInputRef.current?.click()}>
            {uploading ? "Uploading..." : "Upload Avatar"}
        </Button>
        {error && <Label style={{ color: "#DA3737" }}>{error}</Label>}
    </div>;
}

interface UserProfileFromFetcher {
    username: string;
    accepted: number;
    test_accepted: number;
    general: number;
    is_following?: boolean;
    is_followed_by?: boolean;
}

function useUserProfile(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    targetUsername: string,
    viewerUsername: string | undefined
) {
    const [userProfile, setUserProfile] = useState<UserProfileFromFetcher | undefined>(undefined);
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [requestKey, setRequestKey] = useState("");

    const fetchUserProfile = () => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "user_profile",
            content: {
                username: targetUsername,
                viewer_username: viewerUsername,
                request_key: _requestKey
            }
        });

        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((previousMessage) => previousMessage.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, _index) => {
            interface UserProfile {
                type: string;
                content: {
                    username: string;
                    accepted: number;
                    test_accepted: number;
                    general: number;
                    is_following?: boolean;
                    is_followed_by?: boolean;
                    request_key: string;
                }
            }

            function isUserProfile(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'username' in (x.content as object) &&
                        'accepted' in (x.content as object) &&
                        'test_accepted' in (x.content as object) &&
                        'general' in (x.content as object) &&
                        'request_key' in (x.content as object);
                }

                return false;
            }

            if (_message && isUserProfile(_message)) {
                const message = _message as UserProfile;
                console.log(message);
                if (message.content.request_key == requestKey) {
                    setUserProfile({
                        username: message.content.username,
                        accepted: message.content.accepted as number,
                        test_accepted: message.content.test_accepted as number,
                        general: message.content.general as number,
                        is_following: message.content.is_following,
                        is_followed_by: message.content.is_followed_by
                    });

                    delete _websocketMessageHistory[_index];
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { userProfile, fetchUserProfile };
}

function useFollowAction(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    onChanged: () => void
) {
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const [requestKey, setRequestKey] = useState("");
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const follow = (targetUsername: string) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "follow",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                target_username: targetUsername,
                request_key: _requestKey
            }
        });
        setRequestKey(_requestKey);
    };

    const unfollow = (targetUsername: string) => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "unfollow",
            content: {
                username: loginUsername.value,
                session_token: sessionToken.value,
                target_username: targetUsername,
                request_key: _requestKey
            }
        });
        setRequestKey(_requestKey);
    };

    useEffect(() => {
        if (lastJsonMessage !== null) setWebsocketMessageHistory((previousMessage) => previousMessage.concat(lastJsonMessage as []));
    }, [lastJsonMessage]);

    useEffect(() => {
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, _index) => {
            function isFollowOrUnfollowResult(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return (x.type === "follow_result" || x.type === "unfollow_result") &&
                        'success' in (x.content as object) &&
                        'request_key' in (x.content as object);
                }
                return false;
            }

            if (_message && isFollowOrUnfollowResult(_message)) {
                const message = _message as { content: { success: boolean; request_key: string } };
                if (message.content.request_key === requestKey) {
                    if (message.content.success) onChanged();
                    delete _websocketMessageHistory[_index];
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [websocketMessageHistory, requestKey]);

    return { follow, unfollow };
}

function FollowButton({ userProfile, onClickFollow, onClickUnfollow }: {
    userProfile: UserProfileFromFetcher;
    onClickFollow: () => void;
    onClickUnfollow: () => void;
}) {
    if (userProfile.is_following === undefined) return <></>;

    if (!userProfile.is_following)
        return <Button appearance="primary" onClick={onClickFollow}>Follow</Button>;

    if (userProfile.is_followed_by)
        return <Button appearance="secondary" onClick={onClickUnfollow}>Mutual Followed</Button>;

    return <Button appearance="secondary" onClick={onClickUnfollow}>Followed</Button>;
}

export default function UserProfile() {
    const { username: usernameFromParams } = useParams<{ username?: string }>();
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);
    const targetUsername = usernameFromParams ?? loginUsername.value;
    const [avatarCacheBuster, setAvatarCacheBuster] = useState(0);
    const { userProfile, fetchUserProfile } = useUserProfile(
        sendJsonMessage,
        lastJsonMessage,
        targetUsername,
        loginStatus.value ? loginUsername.value : undefined
    );
    const { follow, unfollow } = useFollowAction(sendJsonMessage, lastJsonMessage, fetchUserProfile);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const navigate = useNavigate();

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);

    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true && targetUsername)
            fetchUserProfile();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [loginStatus, targetUsername]);

    const isOwnProfile = loginStatus.value === true && targetUsername === loginUsername.value;

    return <>
        <>
            {
                userProfile ?
                    <div style={{ display: "block", marginLeft: "0.5em", marginTop: "0.4em" }}>
                        <div style={{ display: "flex", alignItems: "center" }}>
                            <Avatar
                                size={64}
                                name={userProfile.username}
                                image={{ src: `/avatar/${userProfile.username}?v=${avatarCacheBuster}` }} />
                            <Label style={{ margin: "0 0.4em" }}>Username: {userProfile.username}</Label>
                        </div>
                        <Label style={{ margin: "0 0.4em" }}>Accepted: {userProfile.accepted}</Label>
                        <Label style={{ margin: "0 0.4em" }}>Test Accepted: {userProfile.test_accepted}</Label>
                        <Label style={{ margin: "0 0.4em" }}>General: {userProfile.general}</Label>
                        {
                            !isOwnProfile &&
                            <span style={{ margin: "0 0.4em" }}>
                                <FollowButton
                                    userProfile={userProfile}
                                    onClickFollow={() => follow(userProfile.username)}
                                    onClickUnfollow={() => unfollow(userProfile.username)} />
                            </span>
                        }
                        {
                            isOwnProfile &&
                            <AvatarUploader
                                username={loginUsername.value}
                                sessionToken={sessionToken.value}
                                onUploaded={() => setAvatarCacheBuster((x) => x + 1)} />
                        }
                    </div>
                    :
                    <></>
            }
        </>
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text="Please login first."
            onClose={() => navigate("/login")} />
    </>;
}
