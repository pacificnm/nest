import { useCallback, useRef } from "react";
import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import { defineNestTheme, languageForFilename, NEST_MONACO_THEME, setupMonaco } from "../../../lib/monaco";
import type { EditorTab } from "./types";

// Point @monaco-editor/react at the bundled Monaco + local workers (no CDN).
setupMonaco();

type Props = {
  tab: EditorTab;
  onChange: (relPath: string, content: string) => void;
  onSave: (relPath: string) => void;
};

/**
 * Monaco-backed code editor for the active tab. Remounts per tab (`key={tab.relPath}`)
 * since Nest Files only ever shows one active editor at a time — no cross-tab
 * model reuse needed the way Kiwi's IDE workbench does.
 */
export function CodeEditor({ tab, onChange, onSave }: Props) {
  const relRef = useRef(tab.relPath);
  relRef.current = tab.relPath;
  const saveRef = useRef(onSave);
  saveRef.current = onSave;

  const beforeMount: BeforeMount = useCallback((monaco) => {
    defineNestTheme(monaco);
  }, []);

  const onMount: OnMount = useCallback((editorInstance, monaco) => {
    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      saveRef.current(relRef.current);
    });
  }, []);

  return (
    <Editor
      key={tab.relPath}
      className="h-full w-full"
      theme={NEST_MONACO_THEME}
      path={tab.relPath}
      language={languageForFilename(tab.name)}
      defaultValue={tab.content}
      beforeMount={beforeMount}
      onMount={onMount}
      onChange={(value) => onChange(relRef.current, value ?? "")}
      loading={<div className="p-3 text-xs text-nest-muted">Loading editor…</div>}
      options={{
        automaticLayout: true,
        fontSize: 13,
        fontFamily: "JetBrains Mono, Consolas, monospace",
        minimap: { enabled: true },
        scrollBeyondLastLine: false,
        smoothScrolling: true,
        tabSize: 2,
        renderWhitespace: "selection",
        fixedOverflowWidgets: true,
      }}
    />
  );
}
