import { StrictMode, Suspense, lazy, type ReactNode } from "react";
import * as ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
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
const Submissions = lazy(() => import("./router/Submissions.tsx"));
const UserProfile = lazy(() => import("./router/UserProfile.tsx"));
const ControlPanel = lazy(() => import("./router/ControlPanel.tsx"));
const ErrorPage = lazy(() => import("./ErrorPage.tsx"));

import store from "./store.ts";
import "./i18n";

type RouteParams = Record<string, string | undefined>;
type RouteTitleOptions = Record<string, unknown>;

// React 19 hoists metadata rendered by a route into <head>. useTranslation keeps the title in
// sync when the language changes without imperative document.title loaders.
function RoutePage({ titleKey, titleOptions, children }: {
    titleKey: string;
    titleOptions?: RouteTitleOptions | ((params: RouteParams) => RouteTitleOptions);
    children: ReactNode;
}) {
    const { t } = useTranslation("routes");
    const params = useParams<RouteParams>() as RouteParams;
    const options = typeof titleOptions === "function" ? titleOptions(params) : titleOptions;

    return <>
        <title>{t(titleKey, options)}</title>
        <Suspense fallback={<Skeleton />}>{children}</Suspense>
    </>;
}

const router = createBrowserRouter([
    {
        path: "/",
        element: <Suspense fallback={<Skeleton />}><Root /></Suspense>,
        errorElement: <ErrorPage />,
        children: [
            {
                path: "/login",
                element: <RoutePage titleKey="title.login"><Login /></RoutePage>,
            },
            {
                path: "/register",
                element: <RoutePage titleKey="title.register"><Register /></RoutePage>,
            },
            {
                path: "/home",
                element: <RoutePage titleKey="title.home"><Home /></RoutePage>,
            },
            {
                path: "/chat",
                element: <RoutePage titleKey="title.chat"><Chat /></RoutePage>,
                children: [
                    {
                        path: "/chat/user/:username",
                        element: <RoutePage titleKey="title.chatUser"><ChatMainUser /></RoutePage>,
                        loader: ({ params }) => ({ toUsername: params.username }),
                    },
                ],
            },
            {
                path: "/problem",
                element: <RoutePage titleKey="title.problem"><Problem /></RoutePage>,
                children: [
                    {
                        path: "/problem/:problem_number",
                        element: (
                            <RoutePage
                                titleKey="title.problemNumber"
                                titleOptions={(params) => ({ number: params.problem_number })}
                            >
                                <ProblemMain />
                            </RoutePage>
                        ),
                        loader: ({ params }) => ({ problemNumber: Number(params.problem_number) }),
                    },
                ],
            },
            {
                path: "/submission",
                element: <RoutePage titleKey="title.submissionsList"><Submissions /></RoutePage>,
                children: [
                    {
                        path: ":submission_id",
                        element: (
                            <RoutePage
                                titleKey="title.submissionId"
                                titleOptions={(params) => ({ id: params.submission_id })}
                            >
                                <SubmissionShower />
                            </RoutePage>
                        ),
                        loader: ({ params }) => ({ submissionId: Number(params.submission_id) }),
                    },
                ],
            },
            {
                path: "/solution/:solution_id",
                element: (
                    <RoutePage
                        titleKey="title.solutionId"
                        titleOptions={(params) => ({ id: params.solution_id })}
                    >
                        <SolutionShower />
                    </RoutePage>
                ),
                loader: ({ params }) => ({ solutionId: Number(params.solution_id) }),
            },
            {
                path: "/discussion",
                element: <RoutePage titleKey="title.discussion"><Discussion /></RoutePage>,
            },
            {
                path: "/discussion/:discussion_id",
                element: (
                    <RoutePage
                        titleKey="title.discussionId"
                        titleOptions={(params) => ({ id: params.discussion_id })}
                    >
                        <DiscussionShower />
                    </RoutePage>
                ),
                loader: ({ params }) => ({ discussionId: Number(params.discussion_id) }),
            },
            {
                path: "/notification",
                element: <RoutePage titleKey="title.notification"><InfoCenter /></RoutePage>,
            },
            {
                path: "/logout",
                element: <RoutePage titleKey="title.home"><Logout /></RoutePage>,
            },
            {
                path: "/user/:username",
                element: (
                    <RoutePage
                        titleKey="title.profileOfUser"
                        titleOptions={(params) => ({ username: params.username })}
                    >
                        <UserProfile />
                    </RoutePage>
                ),
                loader: ({ params }) => ({ username: params.username }),
            },
            {
                path: "/user",
                element: <RoutePage titleKey="title.profileOfUserDefault"><UserProfile /></RoutePage>,
            },
        ],
    },
    {
        path: "/control-panel",
        element: <RoutePage titleKey="title.controlPanel"><ControlPanel /></RoutePage>,
    },
]);

const rootElement = document.getElementById("root");

if (rootElement) {
    ReactDOM.createRoot(rootElement).render(
        <StrictMode>
            <Provider store={store}>
                <RouterProvider router={router} />
            </Provider>
        </StrictMode>,
    );
} else {
    console.error("Failed to find the root element");
}
