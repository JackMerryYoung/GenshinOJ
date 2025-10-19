import { useNavigate } from "react-router-dom";

import { Divider, Avatar, Tab } from "@fluentui/react-components";
import { ChartMultipleFilled, ChatFilled, ClipboardTaskListLtrFilled } from "@fluentui/react-icons";

import { useSelector } from "react-redux";

import { RootState } from "../store";

import "../css/style.css";

export default function NavBar() {
    const loginStatus = useSelector((state: RootState) => state.loginStatus);
    const navigate = useNavigate();
    const onTabSelect = (value: string) => {
        if (value === "home") navigate("/home");
        if (value === "problem") navigate("/problem");
        if (value === "submission") navigate("/submission");
        if (value === "chat") navigate("/chat");
        if (value === "user") navigate("/user");
        if (value === "login") navigate("/login");
        if (value === "register") navigate("/register");
        if (value === "logout") navigate("/logout");
    };

    return (
        <div style={{ padding: "0.35em 0" }}>
            <div style={{ display: "inline" }}>
                <Tab onClick={() => onTabSelect("home")} style={{ float: "left" }} value="home" icon={<Avatar size={24} image={{ src: "https://img.atcoder.jp/icons/373e4eb93e4b8e5f441eeeea55e5ac84.jpg" }} />}>
                    Genshin OJ
                </Tab>
                <Tab onClick={() => onTabSelect("problem")} style={{ float: "left" }} value="problem" icon={<ClipboardTaskListLtrFilled />}>Problem</Tab>
                <Tab onClick={() => onTabSelect("submission")} style={{ float: "left" }} value="submission" icon={<ChartMultipleFilled />}>Submission</Tab>
                <Tab onClick={() => onTabSelect("chat")} style={{ float: "left" }} value="chat" icon={<ChatFilled />}>Chat</Tab>
                {
                    loginStatus.value === true
                        ?
                        <Tab onClick={() => onTabSelect("logout")} style={{ float: "right" }} value="logout">Sign out</Tab>
                        :
                        <>
                            <Tab onClick={() => onTabSelect("login")} style={{ float: "right" }} value="login">Sign in</Tab>
                            <Tab onClick={() => onTabSelect("register")} style={{ float: "right" }} value="register">Sign up</Tab>
                        </>
                }

                <Tab onClick={() => onTabSelect("user")} style={{ float: "right" }} value="user">User</Tab>
                <Divider />
            </div>
        </div>
    );
}
