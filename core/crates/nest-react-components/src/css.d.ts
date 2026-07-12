// Ambient declaration so `tsc` accepts side-effect CSS imports (e.g. runtime.css).
// The bundler (Vite) handles the actual stylesheet; there is no JS export.
declare module '*.css';
