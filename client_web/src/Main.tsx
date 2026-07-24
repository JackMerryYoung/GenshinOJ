import { StrictMode, Suspense, lazy } from "react";
import * as ReactDOM from "react-dom/client";

import {
    createBrowserRouter,
    RouterProvider,
} from "react-router-dom";

import { Provider } from "react-redux";

import { Skeleton } from "@fluentui/react-components";

const Root = lazy(() => import("./router/Root.tsx"));
const Login = lazy(() => import("./router/Login.tsx"));
const Register = lazy(() => import("./router/Register.tsx"));
const Logout = lazy(() => import("./router/Logout.tsx"));
const Home = lazy(() => import("./router/Home.tsx"));
const Chat = lazy(() => import("./router/Chat.tsx"));
const ChatMainUser = lazy(() => import("./router/ChatMainUser.tsx"));
const Problem = lazy(() => import("./router/Problem.tsx"));
const ProblemMain = lazy(() => import("./router/ProblemMain.tsx"));
const SubmissionShower = lazy(() => import("./router/SubmissionShower.tsx"));
const SolutionShower = lazy(() => import("./router/SolutionShower.tsx"));
const Discussion = lazy(() => import("./router/Discussion.tsx"));
const DiscussionShower = lazy(() => import("./router/DiscussionShower.tsx"));
const InfoCenter = lazy(() => import("./router/InfoCenter.tsx"));
const SubmissionsList = lazy(() => import("./router/SubmissionsList.tsx"));
const UserProfile = lazy(() => import("./router/UserProfile.tsx"));
const ControlPanel = lazy(() => import("./router/ControlPanel.tsx"));
const ErrorPage = lazy(() => import("./ErrorPage.tsx"));

import store from "./store.ts";
import i18n from "./i18n";

const t = (key: string, options?: Record<string, unknown>) => i18n.t(key, { ns: "routes", ...options });

const router = createBrowserRouter([
    {
        path: "/",
        element: <Suspense fallback={<Skeleton />}><Root /></Suspense>,
        loader: () => document.title = t("title.root"),
        errorElement: <ErrorPage />,
        children: [
            {
                path: "/login",
                element: <Suspense fallback={<Skeleton />}><Login /></Suspense>,
                loader: () => document.title = t("title.login")
            },
            {
                path: "/register",
                element: <Suspense fallback={<Skeleton />}><Register /></Suspense>,
                loader: () => document.title = t("title.register")
            },
            {
                path: "/home",
                element: <Suspense fallback={<Skeleton />}><Home /></Suspense>,
                loader: () => document.title = t("title.home")
            },
            {
                path: "/chat",
                element: <Suspense fallback={<Skeleton />}><Chat /></Suspense>,
                loader: () => document.title = t("title.chat"),
                children: [
                    {
                        path: "/chat/user/:username",
                        element: <Suspense fallback={<Skeleton />}><ChatMainUser /></Suspense>,
                        loader: ({ params }) => {
                            document.title = t("title.chatUser");
                            return { toUsername: params.username };
                        },
                    }
                ],
            },
            {
                path: "/problem",
                element: <Suspense fallback={<Skeleton />}><Problem /></Suspense>,
                loader: () => document.title = t("title.problem"),
                children: [
                    {
                        path: "/problem/:problem_number",
                        element: <Suspense fallback={<Skeleton />}><ProblemMain /></Suspense>,
                        loader: ({ params }) => {
                            document.title = t("title.problemNumber", { number: params.problem_number });
                            return { problemNumber: Number(params.problem_number) };
                        },
                    }
                ],
            },
            {
                path: "/submission",
                element: <Suspense fallback={<Skeleton />}><SubmissionsList /></Suspense>,
                loader: () => document.title = t("title.submissionsList"),
            },
            {
                path: "/submission/:submission_id",
                element: <Suspense fallback={<Skeleton />}><SubmissionShower /></Suspense>,
                loader: ({ params }) => {
                    document.title = t("title.submissionId", { id: params.submission_id });
                    return { submissionId: Number(params.submission_id) };
                },
            },
            {
                path: "/solution/:solution_id",
                element: <Suspense fallback={<Skeleton />}><SolutionShower /></Suspense>,
                loader: ({ params }) => {
                    document.title = t("title.solutionId", { id: params.solution_id });
                    return { solutionId: Number(params.solution_id) };
                },
            },
            {
                path: "/discussion",
                element: <Suspense fallback={<Skeleton />}><Discussion /></Suspense>,
                loader: () => document.title = t("title.discussion"),
            },
            {
                path: "/discussion/:discussion_id",
                element: <Suspense fallback={<Skeleton />}><DiscussionShower /></Suspense>,
                loader: ({ params }) => {
                    document.title = t("title.discussionId", { id: params.discussion_id });
                    return { discussionId: Number(params.discussion_id) };
                },
            },
            {
                path: "/notification",
                element: <Suspense fallback={<Skeleton />}><InfoCenter /></Suspense>,
                loader: () => document.title = t("title.notification"),
            },
            {
                path: "/logout",
                element: <Suspense fallback={<Skeleton />}><Logout /></Suspense>,
                loader: () => document.title = t("title.home")
            },
            {
                path: "/user/:username",
                element: <Suspense fallback={<Skeleton />}><UserProfile /></Suspense>,
                loader: ({ params }) => {
                    document.title = t("title.profileOfUser", { username: params.username });
                    return { username: params.username };
                },
            },
            {
                path: "/user",
                element: <Suspense fallback={<Skeleton />}><UserProfile /></Suspense>,
                loader: () => document.title = t("title.profileOfUserDefault"),
            }
        ]
    },
    {
        path: "/control-panel",
        element: <Suspense fallback={<Skeleton />}><ControlPanel /></Suspense>,
        loader: () => document.title = t("title.controlPanel"),
    }
]);

const rootElement = document.getElementById("root");

if (rootElement) {
    ReactDOM.createRoot(rootElement).render(<StrictMode><Provider store={store}><RouterProvider router={router} /></Provider></StrictMode>);
} else {
    console.error("Failed to find the root element");
}