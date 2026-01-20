// Strict mode for tests: any console.warn is treated as a test failure.
// This prevents dependencies from silently emitting warnings in CI.
//
// If you ever need to temporarily allow warnings for debugging,
// you can run with ALLOW_CONSOLE_WARN=1.
const allow = (import.meta as unknown as { env?: Record<string, string> }).env?.ALLOW_CONSOLE_WARN === '1'
  // Fallback for Node env
  || (typeof process !== 'undefined' && process.env && process.env.ALLOW_CONSOLE_WARN === '1');

if (!allow) {
  const originalWarn = console.warn.bind(console);
  console.warn = (...args: unknown[]) => {
    const msg = args.map(a => {
      try { return typeof a === 'string' ? a : JSON.stringify(a); } catch { return String(a); }
    }).join(' ');
    // Keep original output for easier diagnosis, then fail hard.
    originalWarn(...args);
    throw new Error(`console.warn is forbidden in tests: ${msg}`);
  };
}


