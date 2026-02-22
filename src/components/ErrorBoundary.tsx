import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[STG ErrorBoundary]", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            height: "100vh",
            background: "#0d1117",
            color: "#f85149",
            fontFamily: "monospace",
            gap: 12,
            padding: 32,
          }}
        >
          <h2 style={{ margin: 0 }}>Runtime Error</h2>
          <pre
            style={{
              background: "#161b22",
              border: "1px solid #30363d",
              borderRadius: 6,
              padding: "12px 16px",
              maxWidth: 640,
              overflow: "auto",
              color: "#e6edf3",
              fontSize: 12,
            }}
          >
            {this.state.error.message}
            {"\n\n"}
            {this.state.error.stack}
          </pre>
          <button
            onClick={() => this.setState({ error: null })}
            style={{
              background: "#21262d",
              border: "1px solid #30363d",
              borderRadius: 6,
              color: "#e6edf3",
              padding: "8px 16px",
              cursor: "pointer",
              fontFamily: "inherit",
            }}
          >
            Retry
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}
