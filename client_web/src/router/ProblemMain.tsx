import { Suspense, useCallback, useEffect, useState, lazy } from "react";
import { useLoaderData, useNavigate, useOutletContext } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { Button, Dropdown, Spinner, Subtitle1, Switch, Tag, Option, Title3, DropdownProps, SelectionEvents, OptionOnSelectData, TabList, Tab, SelectTabEvent, SelectTabData } from "@fluentui/react-components";
import { AlignBottomFilled, AlignBottomRegular, AlignStretchHorizontalFilled, AlignStretchHorizontalRegular, AppGenericFilled, AppGenericRegular, AppsAddInRegular, ArrowRoutingRectangleMultipleRegular } from "@fluentui/react-icons";

const Editor = lazy(() => import("@monaco-editor/react"));
const Markdown = lazy(() => import("react-markdown"));
const rehypeKatex = (await import("rehype-katex")).default;
const remarkMath = (await import("remark-math")).default;

import { useSelector } from "react-redux";

const PopupDialog = lazy(() => import("./PopupDialog.tsx"));
const ProblemSolutions = lazy(() => import("./Solutions.tsx"));

import * as globals from "../Globals.ts";
import { RootState } from "../store.ts";

import 'katex/dist/katex.min.css';
import "../css/style.css";

interface ProblemInfoFromLoader {
    problemNumber: number;
}

interface ProblemInfoFromFetcher {
    problem_number: number;
    difficulty: number;
    problem_name: string;
    problem_statement: string[];
}

const options = [
    {
        description: "C",
        codeLanguage: "c",
        submissionCodeLanguage: "c"
    },
    {
        description: "C++",
        codeLanguage: "cpp",
        submissionCodeLanguage: "cpp"
    },
    {
        description: "Java",
        codeLanguage: "java",
        submissionCodeLanguage: "java"
    },
    {
        description: "Python",
        codeLanguage: "python",
        submissionCodeLanguage: "py"
    }
];

function useProblemInfo(request: globals.WebSocketRequest) {
    const [problemInfo, setProblemInfo] = useState<ProblemInfoFromFetcher | undefined>(undefined);

    const fetchProblemInfo = useCallback(async (problemNumber: number, signal?: AbortSignal) => {
        setProblemInfo(undefined);
        try {
            const response = await request<{ type: string; content: ProblemInfoFromFetcher & { request_key: string } }>(
                "problem_statement",
                { problem_number: problemNumber },
                { signal },
            );
            setProblemInfo(response.content);
        } catch (error) {
            if (error instanceof Error && error.name === "AbortError") return;
        }
    }, [request]);

    return { problemInfo, fetchProblemInfo };
}

function useSubmission(request: globals.WebSocketRequest) {
    const [submissionId, setSubmissionId] = useState<number | undefined>(undefined);
    const loginUsername = useSelector((state: RootState) => state.loginUsername);
    const sessionToken = useSelector((state: RootState) => state.sessionToken);

    const submit = useCallback(async (problemNumber: number, submissionCodeLanguage: string, submissionCode: string, isTestSubmissionMode: boolean) => {
        setSubmissionId(undefined);
        try {
            const response = await request<{ type: string; content: { request_key: string; submission_id?: number } }>("submission", {
                username: loginUsername.value,
                session_token: sessionToken.value,
                problem_number: problemNumber,
                language: submissionCodeLanguage,
                code: submissionCode.split('\n'),
                is_test_submission_mode: isTestSubmissionMode,
            });
            if (response.content.submission_id !== undefined) setSubmissionId(response.content.submission_id);
        } catch {
            // Keep the success dialog closed when the broker reports a failed submission request.
        }
    }, [loginUsername.value, request, sessionToken.value]);

    return { submissionId, submit };
}

function DifficultyShower({ difficulty }: {
    difficulty: number;
}) {
    const { t } = useTranslation("problemMain");
    if (difficulty == 0)
        return <Tag appearance="outline" icon={<AlignBottomRegular style={{ fontSize: "0.8em" }} />} style={{ color: "#999999" }}>{t("difficulty.unknown")}</Tag>;
    if (difficulty == 1)
        return <Tag appearance="outline" icon={<AlignBottomFilled style={{ fontSize: "0.8em" }} />} style={{ color: "#DA3737" }}>{t("difficulty.beginner")}</Tag>;
    if (difficulty == 2)
        return <Tag appearance="outline" icon={<AlignStretchHorizontalRegular style={{ fontSize: "0.8em" }} />} style={{ color: "#CC7700" }}>{t("difficulty.primary")}</Tag>;
    if (difficulty == 3)
        return <Tag appearance="outline" icon={<AlignStretchHorizontalFilled style={{ fontSize: "0.8em" }} />} style={{ color: "#FDDB10" }}>{t("difficulty.junior")}</Tag>;
    if (difficulty == 4)
        return <Tag appearance="outline" icon={<AppGenericRegular style={{ fontSize: "0.8em" }} />} style={{ color: "#3AAF00" }}>{t("difficulty.senior")}</Tag>;
    if (difficulty == 5)
        return <Tag appearance="outline" icon={<AppGenericFilled style={{ fontSize: "0.8em" }} />} style={{ color: "#2744C2" }}>{t("difficulty.advanced")}</Tag>;
    if (difficulty == 6)
        return <Tag appearance="outline" icon={<AppsAddInRegular style={{ fontSize: "0.8em" }} />} style={{ color: "#773388" }}>{t("difficulty.hard")}</Tag>;
    if (difficulty == 7)
        return <Tag appearance="outline" icon={<ArrowRoutingRectangleMultipleRegular style={{ fontSize: "0.8em" }} />} style={{ color: "#1C1C3C" }}>{t("difficulty.grand")}</Tag>;
}

export default function ProblemMain() {
    const { t } = useTranslation(["problemMain", "common"]);
    const { problemNumber } = (useLoaderData() as ProblemInfoFromLoader);
    const [submissionCode, setSubmissionCode] = useState("");
    const [submissionCodeLanguage, setSubmissionCodeLanguage] = useState("cpp");
    const [codeLanguage, setCodeLanguage] = useState("cpp");
    const [isTestSubmissionMode, setIsTestSubmissionMode] = useState(false);
    const { request, sendJsonMessage, lastJsonMessage } = useOutletContext<globals.WebSocketHook>();
    const [dialogSubmitSuccessOpenState, setDialogSubmitSuccessOpenState] = useState(false);
    const [convertedMarkdownRenderString, setConvertedMarkdownRenderString] = useState("");
    const [selectedTab, setSelectedTab] = useState<string>("statement");
    const { submissionId, submit } = useSubmission(request);
    const { problemInfo, fetchProblemInfo } = useProblemInfo(request);

    const navigate = useNavigate();

    const handleSubmitCode = () => {
        submit(problemNumber, submissionCodeLanguage, submissionCode, isTestSubmissionMode)
    };

    const handleClickSubmitCode = () => {
        handleSubmitCode();
    };

    const convertStringListToMarkdownRenderString = (_problemInfo: ProblemInfoFromFetcher) => {
        let markdownRenderString: string = "";
        _problemInfo.problem_statement.map((statement) => {
            markdownRenderString = markdownRenderString +
                `
${statement}
`
        });

        return markdownRenderString;
    };

    const handleSubmissionNavigate = () => {
        navigate("/submission/" + submissionId);
    };

    const handleEditorContentChange = (code: string | undefined,) => {
        setSubmissionCode((code === undefined) ? "" : code);
    };

    useEffect(() => {
        if (problemInfo !== undefined)
            setConvertedMarkdownRenderString(convertStringListToMarkdownRenderString(problemInfo));
    }, [problemInfo]);

    useEffect(() => {
        const controller = new AbortController();
        void fetchProblemInfo(problemNumber, controller.signal);
        return () => controller.abort();
    }, [fetchProblemInfo, problemNumber]);

    useEffect(() => {
        if (submissionId !== undefined) setDialogSubmitSuccessOpenState(true);
    }, [submissionId]);

    return <>
        <div style={{ display: "block", marginLeft: "0.5em", marginTop: "0.5em" }}>
            {
                problemInfo && (problemInfo as ProblemInfoFromFetcher).problem_statement
                    ?
                    <>
                        <Title3>
                            {t("problemTitle", { number: (problemInfo as ProblemInfoFromFetcher).problem_number, name: (problemInfo as ProblemInfoFromFetcher).problem_name })}
                        </Title3>
                        &nbsp;&nbsp;&nbsp;&nbsp;
                        <DifficultyShower difficulty={(problemInfo as ProblemInfoFromFetcher).difficulty} />
                        <TabList
                            selectedValue={selectedTab}
                            onTabSelect={(_ev: SelectTabEvent, data: SelectTabData) => setSelectedTab(data.value as string)}
                            style={{ marginTop: "0.5em" }}>
                            <Tab value="statement">{t("tab.statement")}</Tab>
                            <Tab value="solutions">{t("tab.solutions")}</Tab>
                        </TabList>
                        {
                            selectedTab === "statement"
                                ?
                                <div style={{ display: "block", marginLeft: "1em" }}>
                                    <Subtitle1>{t("problemStatement")}</Subtitle1>
                                    <div style={{ textIndent: "1em", marginTop: "1em" }}>
                                        <Suspense fallback={<Spinner size="tiny" delay={500} />}>
                                            <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                                                {convertedMarkdownRenderString}
                                            </Markdown>
                                        </Suspense>
                                    </div>
                                    <Subtitle1>{t("submitCode")}</Subtitle1>
                                    <CodeLanguageChooser setCodeLanguage={setCodeLanguage} setSubmissionCodeLanguage={setSubmissionCodeLanguage} style={{ marginTop: "0.5em", marginBottom: "1em" }} />
                                    <Suspense fallback={<Spinner size="tiny" delay={500} />}>
                                        <Editor
                                            height="500px"
                                            language={codeLanguage}
                                            onChange={handleEditorContentChange}
                                            loading={<Spinner delay={200} />} />
                                    </Suspense>

                                    <div style={{ alignItems: "center", justifyContent: "center", display: "flex" }}>
                                        <Switch
                                            label={t("testSubmissionMode")}
                                            checked={isTestSubmissionMode}
                                            onChange={(_ev, data) => setIsTestSubmissionMode(data.checked)}
                                            style={{ marginRight: "1em" }}
                                            labelPosition="before" />
                                        <Button appearance="primary" onClick={handleClickSubmitCode}>{t("action.submit", { ns: "common" })}</Button>
                                    </div>
                                </div>
                                :
                                <Suspense fallback={<Spinner size="tiny" delay={500} />}>
                                    <ProblemSolutions
                                        problemNumber={problemNumber}
                                        request={request}
                                        sendJsonMessage={sendJsonMessage}
                                        lastJsonMessage={lastJsonMessage} />
                                </Suspense>
                        }
                    </>
                    :
                    <></>
            }
        </div>
        <PopupDialog
            open={dialogSubmitSuccessOpenState}
            setPopupDialogOpenState={setDialogSubmitSuccessOpenState}
            text={t("submitSuccessNavigating")}
            onClose={handleSubmissionNavigate} />
    </>;
}

function CodeLanguageChooser({ setCodeLanguage, setSubmissionCodeLanguage, style, ...props }: {
    setCodeLanguage: React.Dispatch<React.SetStateAction<string>>;
    setSubmissionCodeLanguage: React.Dispatch<React.SetStateAction<string>>;
    style?: React.CSSProperties;
} & Partial<{
    props: DropdownProps;
}>) {
    const { t } = useTranslation("problemMain");
    const handleOptionSelect = (_ev: SelectionEvents, data: OptionOnSelectData) => {
        if (data.optionValue) {
            setCodeLanguage(data.optionValue.split(' ')[1]);
            setSubmissionCodeLanguage(data.optionValue.split(' ')[0]);
        }
    };

    return (
        <div style={style}>
            <Dropdown placeholder={t("codeLanguage")} onOptionSelect={handleOptionSelect} defaultValue="C++" defaultSelectedOptions={["cpp cpp"]} {...props}>
                {
                    options.map((option) => (
                        <Option value={option.submissionCodeLanguage + ' ' + option.codeLanguage} key={option.submissionCodeLanguage + ' ' + option.codeLanguage}>{option.description}</Option>
                    ))
                }
            </Dropdown>
        </div>
    );
}
