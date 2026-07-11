/**
 * Nest Settings - launched from the Main Menu's "System Settings" button.
 *
 * Left menu of settings categories, main window shows the selected
 * category's settings — same layout as Help / Component Library / Files.
 */

import { useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRobot } from "@fortawesome/free-solid-svg-icons";
import { AgentSettingsPanel } from "./settings/AgentSettingsPanel";

type SettingsCategory = { id: string; label: string; icon: typeof faRobot };

const CATEGORIES: SettingsCategory[] = [{ id: "agent", label: "Agent", icon: faRobot }];

export function SettingsApp() {
  const [selected, setSelected] = useState<string>(CATEGORIES[0].id);

  return (
    <div className="settings-app">
      <aside className="settings-app-sidebar">
        <div className="settings-app-sidebar-header">
          <h2 className="text-sm font-semibold text-nest-foreground">Settings</h2>
          <p className="text-xs text-nest-muted">Nest Desktop preferences</p>
        </div>
        <nav className="settings-app-toc">
          {CATEGORIES.map((category) => (
            <button
              key={category.id}
              type="button"
              className={["settings-app-toc-item", selected === category.id ? "settings-app-toc-item-active" : ""].join(" ")}
              onClick={() => setSelected(category.id)}
            >
              <FontAwesomeIcon icon={category.icon} className="size-3.5" />
              {category.label}
            </button>
          ))}
        </nav>
      </aside>
      <main className="settings-app-content">{selected === "agent" ? <AgentSettingsPanel /> : null}</main>
    </div>
  );
}
