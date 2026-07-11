import { Component, type ErrorInfo, type ReactNode } from "react";
import { faTriangleExclamation, faPowerOff, faBug, faCopy } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  children: ReactNode;
  onExit?: () => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
    this.setState({ errorInfo });
  }

  private handleExit = async () => {
    try {
      // Use Tauri's exit command to properly close the application
      await invoke("exit_app");
    } catch {
      // Fallback: try to close the window
      window.close();
    }
  };

  private handleCopyError = async () => {
    const errorText = this.formatErrorReport();
    try {
      await navigator.clipboard.writeText(errorText);
      alert("Error details copied to clipboard");
    } catch {
      // Fallback for older browsers
      const textarea = document.createElement("textarea");
      textarea.value = errorText;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      alert("Error details copied to clipboard");
    }
  };

  private formatErrorReport(): string {
    const { error, errorInfo } = this.state;
    const lines = [
      "=== Nest Shell Error Report ===",
      `Time: ${new Date().toISOString()}`,
      "",
      "--- Error ---",
      error?.name ?? "Error",
      error?.message ?? "Unknown error",
      "",
      "--- Stack Trace ---",
      error?.stack ?? "No stack trace available",
      "",
      "--- Component Stack ---",
      errorInfo?.componentStack ?? "No component stack available",
      "",
      "--- Environment ---",
      `User Agent: ${navigator.userAgent}`,
      `Platform: ${navigator.platform}`,
      `Language: ${navigator.language}`,
    ];
    return lines.join("\n");
  }

  render() {
    if (this.state.hasError && this.state.error) {
      return (
        <div className="error-boundary-root">
          <div className="error-boundary-container">
            <div className="error-boundary-header">
              <FontAwesomeIcon
                icon={faTriangleExclamation}
                className="error-boundary-icon text-5xl text-nest-error"
              />
              <h1 className="error-boundary-title">Nest Shell Encountered an Error</h1>
              <p className="error-boundary-subtitle">
                The application has crashed. Please review the error details below.
              </p>
            </div>

            <div className="error-boundary-content">
              <div className="error-boundary-error-box">
                <div className="error-boundary-error-header">
                  <FontAwesomeIcon icon={faBug} className="mr-2" />
                  <span className="font-semibold">Error Details</span>
                </div>
                <div className="error-boundary-error-name">
                  {this.state.error.name || "Error"}
                </div>
                <div className="error-boundary-error-message">
                  {this.state.error.message}
                </div>
                {this.state.error.stack && (
                  <details className="error-boundary-stack-details">
                    <summary className="error-boundary-stack-summary">
                      Show Stack Trace
                    </summary>
                    <pre className="error-boundary-stack-trace">
                      {this.state.error.stack}
                    </pre>
                  </details>
                )}
                {this.state.errorInfo?.componentStack && (
                  <details className="error-boundary-stack-details">
                    <summary className="error-boundary-stack-summary">
                      Show Component Stack
                    </summary>
                    <pre className="error-boundary-stack-trace">
                      {this.state.errorInfo.componentStack}
                    </pre>
                  </details>
                )}
              </div>

              <div className="error-boundary-actions">
                <button
                  type="button"
                  className="error-boundary-btn error-boundary-btn-copy"
                  onClick={this.handleCopyError}
                >
                  <FontAwesomeIcon icon={faCopy} className="mr-2" />
                  Copy Error Report
                </button>
                <button
                  type="button"
                  className="error-boundary-btn error-boundary-btn-exit"
                  onClick={this.handleExit}
                >
                  <FontAwesomeIcon icon={faPowerOff} className="mr-2" />
                  Exit Application
                </button>
              </div>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
