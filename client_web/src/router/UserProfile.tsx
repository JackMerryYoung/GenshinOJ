import { useEffect, useState, lazy } from "react";

import { useNavigate, useOutletContext } from "react-router-dom";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";
import { Label } from "@fluentui/react-components";

interface UserProfileFromFetcher {
    accepted: number;
    test_accepted: number;
    general: number;
}

function useUserProfile(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown
) {
    const [userProfile, setUserProfile] = useState<UserProfileFromFetcher | undefined>(undefined);
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const [requestKey, setRequestKey] = useState("");

    const fetchUserProfile = () => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "user_profile",
            content: {
                username: loginUsername.value,
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
                    accepted: number;
                    test_accepted: number;
                    general: number;
                    request_key: string;
                }
            }

            function isUserProfile(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'accepted' in (x.content as object) &&
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
                        accepted: message.content.accepted as number,
                        test_accepted: message.content.test_accepted as number,
                        general: message.content.general as number
                    });

                    delete _websocketMessageHistory[_index];
                }
            }
        });

        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { userProfile, fetchUserProfile };
}

export default function UserProfile() {
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const { userProfile, fetchUserProfile } = useUserProfile(sendJsonMessage, lastJsonMessage);
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const navigate = useNavigate();

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);

    }, [loginStatus]);

    useEffect(() => {
        if (loginStatus.value === true)
            fetchUserProfile();

    }, [loginStatus]);

    return <>
        <>
            {
                userProfile ?
                    <div style={{ display: "block", marginLeft: "0.5em", marginTop: "0.4em" }}>
                        <Label style={{ margin: "0 0.4em" }}>Username: {loginUsername.value}</Label>
                        <Label style={{ margin: "0 0.4em" }}>Accepted: {userProfile.accepted}</Label>
                        <Label style={{ margin: "0 0.4em" }}>Test Accepted: {userProfile.test_accepted}</Label>
                        <Label style={{ margin: "0 0.4em" }}>General: {userProfile.general}</Label>
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