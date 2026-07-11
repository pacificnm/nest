import { useEffect, useMemo, useState } from "react";
import { RemoteImage } from "./components/RemoteImage";
import { ComponentsApp } from "./components/ComponentsApp";
import {
  AppShell,
  ConfirmDialog,
  DatePicker,
  Ribbon,
  RibbonButton,
  RibbonGroup,
  todayIsoDate,
  useStatusBar,
  useToast,
  type RibbonTabDef,
} from "./shell";
import {
  faCircleInfo,
  faGear,
  faPlus,
  faTrash,
} from "./lib/fontawesome";
import {
  applyThemeRootBlock,
  fetchAppMetadata,
  fetchThemeCss,
  type AppMetadata,
} from "./lib/nest";

const DEMO_IMAGE =
  "https://upload.wikimedia.org/wikipedia/commons/thumb/4/47/PNG_transparency_demonstration_1.png/280px-PNG_transparency_demonstration_1.png";

const TABS: RibbonTabDef[] = [
  { id: "home", label: "Home" },
  { id: "components", label: "Components" },
  { id: "view", label: "View" },
  { id: "help", label: "Help" },
];

export function App() {
  const [metadata, setMetadata] = useState<AppMetadata | null>(null);
  const [activeTab, setActiveTab] = useState("home");
  const [date, setDate] = useState(todayIsoDate());
  const [confirmOpen, setConfirmOpen] = useState(false);
  const toast = useToast();
  const { setStatus } = useStatusBar();

  // Show Components app when Components tab is active
  if (activeTab === "components") {
    return <ComponentsApp />;
  }

  useEffect(() => {
    void (async () => {
      try {
        const [meta, theme] = await Promise.all([
          fetchAppMetadata(),
          fetchThemeCss(),
        ]);
        applyThemeRootBlock(theme.root_block);
        setMetadata(meta);
        setStatus(`Loaded ${meta.title}`, { variant: "success", timeoutMs: 3000 });
      } catch {
        setMetadata({ name: "nest-desktop-template", title: "Nest Desktop Template" });
      }
    })();
  }, [setStatus]);

  const ribbon = useMemo(
    () => (
      <Ribbon tabs={TABS} activeTab={activeTab} onTabChange={setActiveTab}>
        <div className="flex h-full items-stretch">
          <RibbonGroup label="Actions">
            <RibbonButton
              label="New"
              icon={faPlus}
              large
              onClick={() => toast.success("Created a new item")}
            />
            <RibbonButton
              label="Delete"
              icon={faTrash}
              iconTint="warning"
              onClick={() => setConfirmOpen(true)}
            />
          </RibbonGroup>
          <RibbonGroup label="App">
            <RibbonButton
              label="About"
              icon={faCircleInfo}
              iconTint="info"
              onClick={() => toast.info(metadata?.title ?? "Nest Desktop Template")}
            />
            <RibbonButton label="Settings" icon={faGear} iconTint="neutral" />
          </RibbonGroup>
        </div>
      </Ribbon>
    ),
    [activeTab, metadata, toast],
  );

  return (
    <>
      <AppShell
        ribbon={ribbon}
        statusLeft={<span>Ready</span>}
        statusRight={<span>{metadata?.name ?? "…"}</span>}
      >
        <div className="mx-auto flex h-full max-w-3xl flex-col gap-6 overflow-auto p-8">
          <header>
            <h1 className="text-2xl font-semibold">
              {metadata?.title ?? "Nest Desktop Template"}
            </h1>
            <p className="text-sm text-nest-muted">
              Tauri + React + Tailwind · shared Nest shell (cbre-light theme)
            </p>
          </header>

          <section className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
            <h2 className="mb-3 text-lg font-medium">Date picker</h2>
            <DatePicker value={date} onChange={setDate} variant="default" placement="below" />
          </section>

          <section className="rounded-nest-lg border border-nest-border bg-nest-surface p-6">
            <h2 className="mb-4 text-lg font-medium">Remote image (nest_image_fetch)</h2>
            <RemoteImage
              url={DEMO_IMAGE}
              alt="PNG transparency demo"
              tags={["demo"]}
              className="h-48 w-full rounded-nest-md object-contain"
            />
          </section>
        </div>
      </AppShell>

      <ConfirmDialog
        open={confirmOpen}
        title="Delete item"
        message="This is a demo of the shared confirm dialog. Delete this item?"
        confirmLabel="Delete"
        danger
        icon={faTrash}
        onConfirm={() => {
          setConfirmOpen(false);
          toast.error("Item deleted");
        }}
        onCancel={() => setConfirmOpen(false)}
      />
    </>
  );
}
