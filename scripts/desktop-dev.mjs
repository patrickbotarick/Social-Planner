import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

// Port 0 lets the OS allocate a free port. Tauri receives that actual URL.
const root = fileURLToPath(new URL('../', import.meta.url));
if (spawnSync('cargo', ['--version'], { stdio: 'ignore' }).status !== 0) {
  console.error('Rust/Cargo não encontrado. Siga os pré-requisitos do README.md. Para prévia da interface: npm run dev:web.');
  process.exit(1);
}
const server = await createServer({ root, server: { host: '127.0.0.1', port: 0 } });
await server.listen();
const address = server.httpServer?.address();
if (!address || typeof address === 'string') {
  await server.close();
  throw new Error('Não foi possível determinar a porta de desenvolvimento.');
}
const devUrl = `http://127.0.0.1:${address.port}`;
console.info(`Interface local: ${devUrl}`);
const child = spawn(process.execPath, [
  fileURLToPath(new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url)),
  'dev', '--config', JSON.stringify({ build: { devUrl } }),
], { cwd: root, stdio: 'inherit' });
let closing = false;
async function close(code) {
  if (closing) return;
  closing = true;
  if (child.exitCode === null) child.kill();
  await server.close();
  process.exitCode = code;
}
child.on('exit', (code) => { void close(code ?? 1); });
child.on('error', (error) => { console.error(error.message); void close(1); });
process.on('SIGINT', () => { void close(130); });
process.on('SIGTERM', () => { void close(143); });
