import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import enCommon from "./locales/en/common.json";
import enRoot from "./locales/en/root.json";
import enRoutes from "./locales/en/routes.json";
import enNavBar from "./locales/en/navBar.json";
import enFooter from "./locales/en/footer.json";
import enHome from "./locales/en/home.json";
import enLogin from "./locales/en/login.json";
import enRegister from "./locales/en/register.json";
import enLogout from "./locales/en/logout.json";
import enChat from "./locales/en/chat.json";
import enChatMainUser from "./locales/en/chatMainUser.json";
import enProblem from "./locales/en/problem.json";
import enProblemMain from "./locales/en/problemMain.json";
import enProblemEditor from "./locales/en/problemEditor.json";
import enSubmissionsList from "./locales/en/submissionsList.json";
import enSubmissionShower from "./locales/en/submissionShower.json";
import enSolutions from "./locales/en/solutions.json";
import enSolutionShower from "./locales/en/solutionShower.json";
import enDiscussion from "./locales/en/discussion.json";
import enDiscussionShower from "./locales/en/discussionShower.json";
import enInfoCenter from "./locales/en/infoCenter.json";
import enUserProfile from "./locales/en/userProfile.json";
import enControlPanel from "./locales/en/controlPanel.json";
import enPopupDialog from "./locales/en/popupDialog.json";
import enMarkdownView from "./locales/en/markdownView.json";
import enMentionTextarea from "./locales/en/mentionTextarea.json";
import enEmojiPicker from "./locales/en/emojiPicker.json";
import enBadgeButton from "./locales/en/badgeButton.json";

import zhCommon from "./locales/zh-CN/common.json";
import zhRoot from "./locales/zh-CN/root.json";
import zhRoutes from "./locales/zh-CN/routes.json";
import zhNavBar from "./locales/zh-CN/navBar.json";
import zhFooter from "./locales/zh-CN/footer.json";
import zhHome from "./locales/zh-CN/home.json";
import zhLogin from "./locales/zh-CN/login.json";
import zhRegister from "./locales/zh-CN/register.json";
import zhLogout from "./locales/zh-CN/logout.json";
import zhChat from "./locales/zh-CN/chat.json";
import zhChatMainUser from "./locales/zh-CN/chatMainUser.json";
import zhProblem from "./locales/zh-CN/problem.json";
import zhProblemMain from "./locales/zh-CN/problemMain.json";
import zhProblemEditor from "./locales/zh-CN/problemEditor.json";
import zhSubmissionsList from "./locales/zh-CN/submissionsList.json";
import zhSubmissionShower from "./locales/zh-CN/submissionShower.json";
import zhSolutions from "./locales/zh-CN/solutions.json";
import zhSolutionShower from "./locales/zh-CN/solutionShower.json";
import zhDiscussion from "./locales/zh-CN/discussion.json";
import zhDiscussionShower from "./locales/zh-CN/discussionShower.json";
import zhInfoCenter from "./locales/zh-CN/infoCenter.json";
import zhUserProfile from "./locales/zh-CN/userProfile.json";
import zhControlPanel from "./locales/zh-CN/controlPanel.json";
import zhPopupDialog from "./locales/zh-CN/popupDialog.json";
import zhMarkdownView from "./locales/zh-CN/markdownView.json";
import zhMentionTextarea from "./locales/zh-CN/mentionTextarea.json";
import zhEmojiPicker from "./locales/zh-CN/emojiPicker.json";
import zhBadgeButton from "./locales/zh-CN/badgeButton.json";

export const defaultNS = "common";

export const resources = {
    en: {
        common: enCommon,
        root: enRoot,
        routes: enRoutes,
        navBar: enNavBar,
        footer: enFooter,
        home: enHome,
        login: enLogin,
        register: enRegister,
        logout: enLogout,
        chat: enChat,
        chatMainUser: enChatMainUser,
        problem: enProblem,
        problemMain: enProblemMain,
        problemEditor: enProblemEditor,
        submissionsList: enSubmissionsList,
        submissionShower: enSubmissionShower,
        solutions: enSolutions,
        solutionShower: enSolutionShower,
        discussion: enDiscussion,
        discussionShower: enDiscussionShower,
        infoCenter: enInfoCenter,
        userProfile: enUserProfile,
        controlPanel: enControlPanel,
        popupDialog: enPopupDialog,
        markdownView: enMarkdownView,
        mentionTextarea: enMentionTextarea,
        emojiPicker: enEmojiPicker,
        badgeButton: enBadgeButton,
    },
    "zh-CN": {
        common: zhCommon,
        root: zhRoot,
        routes: zhRoutes,
        navBar: zhNavBar,
        footer: zhFooter,
        home: zhHome,
        login: zhLogin,
        register: zhRegister,
        logout: zhLogout,
        chat: zhChat,
        chatMainUser: zhChatMainUser,
        problem: zhProblem,
        problemMain: zhProblemMain,
        problemEditor: zhProblemEditor,
        submissionsList: zhSubmissionsList,
        submissionShower: zhSubmissionShower,
        solutions: zhSolutions,
        solutionShower: zhSolutionShower,
        discussion: zhDiscussion,
        discussionShower: zhDiscussionShower,
        infoCenter: zhInfoCenter,
        userProfile: zhUserProfile,
        controlPanel: zhControlPanel,
        popupDialog: zhPopupDialog,
        markdownView: zhMarkdownView,
        mentionTextarea: zhMentionTextarea,
        emojiPicker: zhEmojiPicker,
        badgeButton: zhBadgeButton,
    },
} as const;

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources,
        fallbackLng: "en",
        defaultNS,
        ns: Object.keys(resources.en),
        interpolation: {
            escapeValue: false,
        },
        detection: {
            order: ["localStorage", "navigator"],
            caches: ["localStorage"],
            lookupLocalStorage: "language",
        },
    });

export default i18n;
