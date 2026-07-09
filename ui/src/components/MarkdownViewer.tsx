import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

type MarkdownViewerProps = {
  markdown: string;
  className?: string;
};

export function MarkdownViewer({ markdown, className = "" }: MarkdownViewerProps) {
  if (!markdown.trim()) {
    return <p className="text-sm text-nest-muted">No content.</p>;
  }

  return (
    <article className={["nest-markdown-viewer", className].join(" ")}>
      <ReactMarkdown remarkPlugins={[remarkGfm]}>{markdown}</ReactMarkdown>
    </article>
  );
}
