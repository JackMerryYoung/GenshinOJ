import { lazy, Suspense } from "react";
import { useNavigate } from "react-router-dom";

import { Spinner } from "@fluentui/react-components";

const Markdown = lazy(() => import("react-markdown"));
const rehypeKatex = (await import("rehype-katex")).default;
const remarkMath = (await import("remark-math")).default;

import 'katex/dist/katex.min.css';

// Turn `@name` (at the start of the text or after whitespace) into a markdown link to that user's
// profile page. Matches the same `[A-Za-z0-9_]` username shape the backend uses when parsing
// mentions for notifications, so what gets linked here is exactly what triggers a notification.
function linkifyMentions(text: string) {
    return text.replace(/(^|\s)@([A-Za-z0-9_]+)/g, (_match, pre, name) => `${pre}[@${name}](/user/${name})`);
}

// Shared markdown renderer for solution / discussion bodies, comments and replies. Renders math
// (remark-math + KaTeX) like the rest of the app, linkifies @mentions, and keeps internal links
// (`/user/...`) inside the SPA instead of triggering a full page reload.
export default function MarkdownView({ lines }: { lines: string[] }) {
    const navigate = useNavigate();
    const source = linkifyMentions(lines.join('\n'));

    return <Suspense fallback={<Spinner size="tiny" delay={500} />}>
        <Markdown
            remarkPlugins={[remarkMath]}
            rehypePlugins={[rehypeKatex]}
            components={{
                a: ({ href, children }) => {
                    if (href !== undefined && href.startsWith("/")) {
                        return <a
                            href={href}
                            onClick={(e) => { e.preventDefault(); navigate(href); }}
                            style={{ color: "#4183C4", cursor: "pointer" }}>
                            {children}
                        </a>;
                    }
                    return <a href={href} target="_blank" rel="noopener noreferrer">{children}</a>;
                },
            }}>
            {source}
        </Markdown>
    </Suspense>;
}
