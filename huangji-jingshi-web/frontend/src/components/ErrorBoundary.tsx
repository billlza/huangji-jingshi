import { Component } from 'react';
import type { ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-[#050508] text-white p-6">
          <div className="max-w-md w-full bg-white/5 border border-red-500/30 rounded-2xl p-8 backdrop-blur-xl">
            <h2 className="text-xl font-serif text-red-400 mb-4 tracking-widest">系统错误 (System Error)</h2>
            <p className="text-gray-400 text-sm mb-6">应用遇到意外错误，无法继续运行。</p>
            <div className="bg-black/30 p-4 rounded-lg border border-white/5 mb-6 overflow-auto max-h-40">
              <code className="text-xs text-red-300 font-mono break-all">
                {this.state.error?.message || 'Unknown error'}
              </code>
            </div>
            <button
              onClick={() => window.location.reload()}
              className="w-full py-3 bg-red-500/20 hover:bg-red-500/30 text-red-200 border border-red-500/50 rounded-lg transition-all uppercase tracking-widest text-xs font-bold"
            >
              刷新页面 (Refresh)
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

