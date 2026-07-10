type PlaceholderAppProps = {
  title: string;
  description: string;
  path?: string;
};

export function PlaceholderApp({ title, description, path }: PlaceholderAppProps) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-8 text-center">
      <h2 className="text-lg font-semibold text-nest-foreground">{title}</h2>
      <p className="max-w-md text-sm text-nest-muted">{description}</p>
      {path ? (
        <p className="font-mono text-xs text-nest-muted">{path}</p>
      ) : null}
      <p className="max-w-md text-xs text-nest-muted">
        Add <code className="text-nest-foreground">mode = &quot;embed&quot;</code> under{" "}
        <code className="text-nest-foreground">[shell.launch]</code> in{" "}
        <code className="text-nest-foreground">nest-app.toml</code> to load this app&apos;s Vite
        dev server inside the shell.
      </p>
    </div>
  );
}
