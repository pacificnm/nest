import type { ShellApp } from "../shell/types";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faBars,
  faFolder,
  faClockRotateLeft,
  faStar,
  faGear,
  faPowerOff,
} from "@fortawesome/free-solid-svg-icons";

type StartMenuProps = {
  apps: ShellApp[];
  isOpen: boolean;
  onClose: () => void;
  onLaunchApp: (appId: string) => void;
  onExit: () => void;
};

export function StartMenu({ apps, isOpen, onClose, onLaunchApp, onExit }: StartMenuProps) {
  if (!isOpen) return null;

  const categories = Array.from(new Set(apps.map((app) => app.category)));

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} />
      <div className="absolute bottom-12 left-0 z-50 flex h-[500px] w-[800px] border border-nest-border bg-nest-surface shadow-2xl">
        <div className="flex w-64 flex-col border-r border-nest-border bg-nest-background/50">
          <div className="border-b border-nest-border p-4">
            <div className="flex items-center gap-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-nest-primary">
                <FontAwesomeIcon icon={faBars} className="text-white" />
              </div>
              <div>
                <h2 className="text-sm font-semibold text-nest-foreground">Nest Shell</h2>
                <p className="text-xs text-nest-muted">Applications</p>
              </div>
            </div>
          </div>

          <nav className="flex-1 overflow-y-auto p-2">
            <div className="mb-2 px-3 py-2 text-xs font-semibold uppercase text-nest-muted">
              Categories
            </div>
            {categories.map((category) => (
              <button
                key={category}
                className="flex w-full items-center gap-3 rounded-nest-md px-3 py-2.5 text-left text-sm text-nest-foreground hover:bg-nest-primary/20"
              >
                <FontAwesomeIcon icon={faFolder} className="text-nest-muted" />
                {category}
              </button>
            ))}

            <div className="mt-4 border-t border-nest-border pt-2">
              <button className="flex w-full items-center gap-3 rounded-nest-md px-3 py-2.5 text-left text-sm text-nest-foreground hover:bg-nest-primary/20">
                <FontAwesomeIcon icon={faClockRotateLeft} className="text-nest-muted" />
                Recent
              </button>
              <button className="flex w-full items-center gap-3 rounded-nest-md px-3 py-2.5 text-left text-sm text-nest-foreground hover:bg-nest-primary/20">
                <FontAwesomeIcon icon={faStar} className="text-nest-muted" />
                Favorites
              </button>
            </div>
          </nav>

          <div className="border-t border-nest-border p-2">
            <button
              onClick={() => onLaunchApp("settings")}
              className="flex w-full items-center gap-3 rounded-nest-md px-3 py-2.5 text-left text-sm text-nest-foreground hover:bg-nest-primary/20"
            >
              <FontAwesomeIcon icon={faGear} className="text-nest-muted" />
              System Settings
            </button>
          </div>
        </div>

        <div className="flex flex-1 flex-col">
          <div className="border-b border-nest-border p-4">
            <input
              type="text"
              placeholder="Type to search..."
              className="w-full rounded-nest-md border border-nest-border bg-nest-background px-4 py-2 text-sm text-nest-foreground placeholder-nest-muted focus:border-nest-primary focus:outline-none"
            />
          </div>

          <div className="flex-1 overflow-y-auto p-4">
            {categories.map((category) => (
              <div key={category} className="mb-6">
                <h3 className="mb-3 text-sm font-semibold text-nest-muted">{category}</h3>
                <div className="grid grid-cols-3 gap-2">
                  {apps
                    .filter((app) => app.category === category)
                    .map((app) => (
                      <button
                        key={app.id}
                        className="flex flex-col items-center gap-2 rounded-nest-md p-3 hover:bg-nest-primary/20"
                        onClick={() => onLaunchApp(app.id)}
                      >
                        <div className="flex h-12 w-12 items-center justify-center rounded-nest-lg bg-nest-primary/20">
                          <FontAwesomeIcon
                            icon={app.icon as never}
                            className="text-xl text-nest-primary"
                          />
                        </div>
                        <span className="text-xs text-nest-foreground">{app.name}</span>
                      </button>
                    ))}
                </div>
              </div>
            ))}
          </div>

          <div className="border-t border-nest-border bg-nest-background/30 p-3">
            <button
              onClick={onExit}
              className="flex w-full items-center justify-center gap-2 rounded-nest-md bg-nest-error/20 px-4 py-2.5 text-sm font-medium text-nest-error hover:bg-nest-error/30"
            >
              <FontAwesomeIcon icon={faPowerOff} />
              Exit Nest Shell
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
