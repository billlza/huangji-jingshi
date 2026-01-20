import { spawn } from 'node:child_process';

/**
 * Run a command and fail if output contains warnings.
 *
 * Usage:
 *   node scripts/strict-run.mjs "<command>"
 */
const cmd = process.argv.slice(2).join(' ').trim();
if (!cmd) {
  console.error('Usage: node scripts/strict-run.mjs "<command>"');
  process.exit(2);
}

const warningPatterns = [
  /\bwarn(ing)?\b/i,
  /\bdeprecationwarning\b/i,
  /\bdeprecated\b/i,
  /\bexperimentalwarning\b/i,
];

let combined = '';
const child = spawn(cmd, {
  shell: true,
  stdio: ['inherit', 'pipe', 'pipe'],
  env: process.env,
});

child.stdout.on('data', (d) => {
  const s = d.toString();
  process.stdout.write(s);
  combined += s;
});

child.stderr.on('data', (d) => {
  const s = d.toString();
  process.stderr.write(s);
  combined += s;
});

child.on('close', (code) => {
  if (code !== 0) process.exit(code ?? 1);
  const hit = warningPatterns.find((re) => re.test(combined));
  if (hit) {
    console.error(`\n[strict-run] Found warning-like output matching ${hit}. Failing.`);
    process.exit(1);
  }
  process.exit(0);
});


