export type EditorTab = {
  relPath: string;
  name: string;
  content: string;
  loading: boolean;
  error: string | null;
  dirty: boolean;
};
