import { useMemo } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";

type EmbeddedAppProps = {
  url?: string;
  message?: string;
  title: string;
};

export function EmbeddedApp({ url, message, title }: EmbeddedAppProps) {
  const src = useMemo(() => {
    if (!url) {
      return null;
    }
    if (url.startsWith("http://") || url.startsWith("https://")) {
      return url;
    }
    return convertFileSrc(url);
  }, [url]);

  if (!src) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center text-sm text-nest-muted">
        <p>{message ?? "Embedded app is not available."}</p>
        {message ? (
          <p className="text-xs">
            In development, run <code className="text-nest-foreground">npm run tauri:dev</code>{" "}
            from <code className="text-nest-foreground">ui/</code> to start embed dev servers.
          </p>
        ) : null}
      </div>
    );
  }

  return (
    <iframe
      className="h-full w-full border-0 bg-nest-background"
      src={src}
      title={title}
      sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
    />
  );
}
