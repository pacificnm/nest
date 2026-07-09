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
        Shell embedding for registered apps is coming next — this window proves dynamic app
        discovery from <code className="text-nest-foreground">nest-app.toml</code>.
      </p>
    </div>
  );
}
