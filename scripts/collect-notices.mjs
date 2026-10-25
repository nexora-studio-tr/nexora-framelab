import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { dirname, join } from 'node:path';
const sections = [
  'Third-party license texts bundled with Nexora FrameLab.\nNew project code: see LICENSE.',
];
async function collect(label, directory, license, repository) {
  const entries = await readdir(directory, { withFileTypes: true });
  const texts = [];
  for (const entry of entries) {
    if (!/^(licen[cs]e|copying|copyright|notice)([.\-_ ]|$)/i.test(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isFile()) texts.push(`${entry.name}\n${await readFile(path, 'utf8')}`);
    else if (entry.isDirectory()) {
      for (const file of await readdir(path, { withFileTypes: true })) {
        if (file.isFile())
          texts.push(
            `${entry.name}/${file.name}\n${await readFile(join(path, file.name), 'utf8')}`,
          );
      }
    }
  }
  sections.push(
    `\n===== ${label} =====\nLicense: ${license || 'See upstream'}\n${repository || ''}\n${texts.join('\n\n') || 'License declaration retained above; see the linked upstream package.'}`,
  );
}
const lock = JSON.parse(await readFile('package-lock.json', 'utf8'));
for (const [path, pkg] of Object.entries(lock.packages)) {
  if (!path || pkg.dev) continue;
  const manifest = JSON.parse(await readFile(join(path, 'package.json'), 'utf8'));
  await collect(
    `${manifest.name} ${manifest.version}`,
    path,
    manifest.license,
    typeof manifest.repository === 'string' ? manifest.repository : manifest.repository?.url,
  );
}
const cargo = JSON.parse(
  execFileSync(
    'cargo',
    [
      'metadata',
      '--locked',
      '--format-version',
      '1',
      '--filter-platform',
      'x86_64-pc-windows-msvc',
    ],
    { encoding: 'utf8', windowsHide: true, maxBuffer: 20 * 1024 * 1024 },
  ),
);
for (const pkg of cargo.packages) {
  if (!pkg.source) continue;
  await collect(
    `${pkg.name} ${pkg.version}`,
    dirname(pkg.manifest_path),
    pkg.license,
    pkg.repository,
  );
}
await mkdir('tools/notices', { recursive: true });
await writeFile('tools/notices/THIRD-PARTY-LICENSES.txt', sections.join('\n\n'));
console.log(`Collected ${sections.length - 1} dependency license records.`);
