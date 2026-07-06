import { useEffect, useState } from "react";
import { Sparkles } from "lucide-react";
import { RemoteImage } from "./components/RemoteImage";
import {
  applyThemeRootBlock,
  fetchAppMetadata,
  fetchThemeCss,
  type AppMetadata,
} from "./lib/nest";

const DEMO_IMAGE =
  "https://upload.wikimedia.org/wikipedia/commons/thumb/4/47/PNG_transparency_demonstration_1.png/280px-PNG_transparency_demonstration_1.png";

export function App() {
  const [metadata, setMetadata] = useState<AppMetadata | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        const [meta, theme] = await Promise.all([
          fetchAppMetadata(),
          fetchThemeCss(),
        ]);
        applyThemeRootBlock(theme.root_block);
        setMetadata(meta);
      } catch {
        setMetadata({ name: "nest-desktop-template", title: "Nest Desktop Template" });
      }
    })();
  }, []);

  return (
    <main className="mx-auto flex min-h-screen max-w-3xl flex-col gap-8 p-8">
      <header className="flex items-center gap-3">
        <Sparkles className="size-8 text-nest-primary" aria-hidden />
        <div>
          <h1 className="text-2xl font-semibold">
            {metadata?.title ?? "Nest Desktop Template"}
          </h1>
          <p className="text-sm text-nest-muted">
            Tauri + React + Tailwind · {metadata?.name ?? "…"}
          </p>
        </div>
      </header>

      <section className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
        <h2 className="mb-4 text-lg font-medium">Remote image (nest_image_fetch)</h2>
        <RemoteImage
          url={DEMO_IMAGE}
          alt="PNG transparency demo"
          tags={["demo"]}
          className="h-48 w-full rounded-nest-md object-contain"
        />
      </section>
    </main>
  );
}
