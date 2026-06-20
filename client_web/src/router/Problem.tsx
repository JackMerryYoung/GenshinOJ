import { useRef, useEffect, useState, lazy } from "react";
import { Outlet, useOutletContext, useNavigate } from "react-router-dom";

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
} from "@fluentui/react-components";

import { useSelector } from "react-redux";

import { nanoid } from "nanoid";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));

import * as globals from "../Globals.ts";

import { RootState } from "../store.ts";

import "../css/style.css";
import { ErrorCircle20Color } from "@fluentui/react-icons";


const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "row",
        rowGap: "4px",
        columnGap: "4px",
        height: "fill",
        margin: "0 0.3em",
    },
});

function useProblemList(
    sendJsonMessage: globals.SendJsonMessage,
    lastJsonMessage: unknown,
    setElapsedTime: React.Dispatch<React.SetStateAction<number>>
) {
    const [problemList, setProblemList] = useState<string[] | undefined>(undefined);
    const [requestKey, setRequestKey] = useState<string>("");
    const [websocketMessageHistory, setWebsocketMessageHistory] = useState([]);
    const elapsedTimerSinceLoaded = useRef(0);

    useEffect(() => {
        elapsedTimerSinceLoaded.current = setInterval(() => {
            setElapsedTime(x => x + 1);
        }, 1000);

        return () => {
            if (elapsedTimerSinceLoaded.current) {
                clearInterval(elapsedTimerSinceLoaded.current);
            }
        };
    }, []);

    const loadProblemList = () => {
        const _requestKey = nanoid();
        sendJsonMessage({
            type: "problem_set",
            content: {
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
        let newProblemList: string[] = [], changed = false;
        const _websocketMessageHistory = websocketMessageHistory;
        _websocketMessageHistory.map((_message, index) => {
            interface ProblemSet {
                type: string;
                content: {
                    problem_set: string[],
                    request_key: string
                };
            }

            function isProblemSet(x: object) {
                if ('type' in x && 'content' in x && typeof x.content === 'object') {
                    return 'problem_set' in (x.content as object) && 'request_key' in (x.content as object);
                }

                return false;
            }
            if (_message && isProblemSet(_message)) {
                console.log(_message);
                const message = _message as ProblemSet;
                if (message.content.request_key === requestKey) {
                    changed = true;
                    newProblemList = message.content.problem_set;
                    delete _websocketMessageHistory[index];
                }
            }
        });

        if (changed) setProblemList(newProblemList);
        if (!globals.compareArray(_websocketMessageHistory, websocketMessageHistory)) setWebsocketMessageHistory(_websocketMessageHistory);
    }, [websocketMessageHistory, requestKey]);

    return { problemList, loadProblemList };
}

function TableCellForProblemList({ problem_number }: {
    problem_number: string;
}) {
    const navigate = useNavigate();
    const handleClick = () => {
        navigate("/problem/" + problem_number);
    };
    return <TableRow key={problem_number}>
        <TableCell onClick={handleClick}>{problem_number}</TableCell>
    </TableRow>;
}

export function ProblemList({ sendJsonMessage, lastJsonMessage }: { sendJsonMessage: globals.SendJsonMessage, lastJsonMessage: unknown }) {
    const [elapsedTime, setElapsedTime] = useState(0);
    const { problemList, loadProblemList } = useProblemList(sendJsonMessage, lastJsonMessage, setElapsedTime);

    useEffect(() => {
        loadProblemList();
    }, []);

    return <>
        {
            problemList !== undefined
                ?
                <div style={{ overflowY: "auto", maxHeight: "60vh" }}>
                    <Table size="medium">
                        <TableHeader style={{ position: "sticky", top: 0, zIndex: 1, background: "#fff" }}>
                            <TableRow>
                                <TableHeaderCell>Problem List</TableHeaderCell>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {
                                problemList.map((problem_number: string, index: number) => (
                                    <TableCellForProblemList key={index} problem_number={problem_number} />
                                )
                                )
                            }
                        </TableBody>
                    </Table>
                </div>
                :
                <div style={{ display: "flex", blockSize: "100%" }}>
                    {
                        elapsedTime >= 10
                            ?
                            <div style={{ display: "flex", margin: "50% 10% 50% 10%" }}>
                                <div style={{ display: "flex", blockSize: "100% 80%" }}>
                                    <ErrorCircle20Color style={{ margin: "auto" }} />
                                    <Label style={{ margin: "auto", padding: "0 0 0 8px", fontSize: "14px" }}>
                                        Failed to fetch the problem list.
                                    </Label>
                                </div>
                            </div>
                            :
                            <div style={{ display: "flex", margin: "50% 0% 50% 35%" }}>
                                <Spinner size="tiny" label="Waiting..." delay={500} style={{ margin: "auto" }} />
                            </div>
                    }
                </div>
        }
    </>;
}

export default function Problem() {
    const { sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const navigate = useNavigate();
    const [dialogRequireLoginOpenState, setDialogRequireLoginOpenState] = useState(false);
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const style = useStyles();

    useEffect(() => {
        const localLoginStatus = localStorage.getItem("loginStatus");
        if (localLoginStatus === null || (loginStatus.value === false && localLoginStatus !== null && JSON.parse(localLoginStatus) === false))
            setDialogRequireLoginOpenState(true);
    }, [loginStatus]);

    const handleCloseDialogRequireLogin = () => {
        navigate("/login");
    };

    return <>
        {
            loginStatus.value && (
                <div className={style.root}>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.15)" }}>
                        <ProblemList
                            sendJsonMessage={sendJsonMessage}
                            lastJsonMessage={lastJsonMessage} />
                    </div>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.01)" }}>
                        <Divider vertical style={{ height: "calc(100vh - 8.8em)" }} />
                    </div>
                    <div style={{ width: "calc((100vw - 23.7px) * 0.84)" }}>
                        <Outlet context={{ sendJsonMessage, lastJsonMessage }} />
                    </div>
                </div>
            )
        }
        <PopupDialog
            open={dialogRequireLoginOpenState}
            setPopupDialogOpenState={setDialogRequireLoginOpenState}
            text="Please login first."
            onClose={handleCloseDialogRequireLogin} />
    </>;
}