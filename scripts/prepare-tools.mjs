import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
const lock = JSON.parse(
  await readFile(new URL('../tools/presentmon.lock.json', import.meta.url), 'utf8'),
);
const dir = new URL('../tools/presentmon/', import.meta.url);
await mkdir(dir, { recursive: true });
const target = new URL('PresentMon.exe', dir);
let bytes = await readFile(target).catch(() => null);
const valid = (b) =>
  b && b.length === lock.bytes && createHash('sha256').update(b).digest('hex') === lock.sha256;
if (!valid(bytes)) {
  const response = await fetch(lock.url, { signal: AbortSignal.timeout(60000) });
  if (!response.ok) throw Error('PresentMon download failed');
  bytes = Buffer.from(await response.arrayBuffer());
  if (!valid(bytes)) throw Error('PresentMon integrity mismatch');
  await writeFile(target, bytes);
}
const response = await fetch(lock.licenseUrl, { signal: AbortSignal.timeout(30000) });
if (!response.ok) throw Error('License download failed');
const license = await response.text();
if (createHash('sha256').update(license).digest('hex') !== lock.licenseSha256)
  throw Error('License integrity mismatch');
await writeFile(new URL('LICENSE.txt', dir), license);
console.log(`PresentMon ${lock.version}: SHA-256 verified; license retained.`);
