import { mkdir, writeFile } from 'node:fs/promises';
const root = new URL('../fixtures/', import.meta.url);
await mkdir(root, { recursive: true });
for (const [name, factor] of [
  ['baseline', 1],
  ['candidate', 0.9],
]) {
  const rows = [
    'Application,ProcessID,SwapChainAddress,CPUStartTime,FrameTime,GPUBusy,DisplayedTime',
  ];
  let start = 0;
  for (let i = 0; i < 2400; i++) {
    const ms = (10 + ((i * 17) % 19) * 0.025) * factor;
    rows.push(
      `cs2.exe,4242,0x1,${(start / 1000).toFixed(6)},${ms.toFixed(6)},${(ms * 0.88).toFixed(6)},NA`,
    );
    start += ms;
  }
  await writeFile(new URL(`synthetic-${name}.csv`, root), rows.join('\n') + '\n');
}
await writeFile(
  new URL('video-v16.synthetic.txt', root),
  '"video.cfg"\n{\n"Version" "16"\n"setting.msaa_samples" "8"\n"setting.videocfg_particle_detail" "2"\n"setting.defaultres" "1920"\n"setting.defaultresheight" "1080"\n}\n',
);
console.log('Synthetic test fixtures generated; not real CS2 recordings.');
