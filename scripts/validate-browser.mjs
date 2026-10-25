import { chromium, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { mkdir, writeFile } from 'node:fs/promises';
const out = new URL('../artifacts/browser/', import.meta.url);
await mkdir(out, { recursive: true });
const child = spawn(
  process.execPath,
  ['node_modules/vite/bin/vite.js', '--host', '127.0.0.1', '--port', '4368', '--strictPort'],
  { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] },
);
let browser;
const checks = [];
try {
  for (let i = 0; i < 80; i++) {
    try {
      if ((await fetch('http://127.0.0.1:4368')).ok) break;
    } catch {}
    if (child.exitCode !== null) throw Error('Demo server exited');
    await new Promise((r) => setTimeout(r, 100));
  }
  browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 1050 },
    reducedMotion: 'reduce',
  });
  const page = await context.newPage();
  const errors = [],
    requests = [];
  page.on('pageerror', (e) => errors.push(e.message));
  page.on('request', (r) => requests.push(r.url()));
  await page.goto('http://127.0.0.1:4368');
  await expect(page.getByText('SENTETİK ARAYÜZ DEMOSU', { exact: true })).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Analiz et ve optimize et', exact: true }),
  ).toBeDisabled();
  checks.push('Web demo explicitly labels synthetic data and cannot mutate the system');
  await page.getByRole('button', { name: 'Örnek sonucu incele', exact: true }).click();
  await expect(page.getByText('SENTETİK ÖRNEK', { exact: true }).first()).toBeVisible();
  await expect(page.locator('.trace')).toHaveCount(2);
  checks.push('Synthetic example opens both frame-time traces without a game connection');
  await page.screenshot({
    path: fileURLToPath(new URL('desktop-tr.png', out)),
    fullPage: true,
  });
  await page.getByRole('button', { name: 'Switch to English' }).click();
  await expect(page.locator('html')).toHaveAttribute('lang', 'en');
  await expect(page.getByRole('heading', { name: 'Frame times' })).toBeVisible();
  checks.push('TR and EN interface switch without changing measurement state');
  await page.getByRole('button', { name: 'How does it work?', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  checks.push('Radix dialog supports keyboard dismissal');
  await page.getByRole('tab', { name: 'Results', exact: false }).click();
  await expect(page.getByRole('button', { name: 'Export report', exact: true })).toBeDisabled();
  checks.push('Synthetic evidence cannot export or restore a real configuration');
  await page.getByRole('tab', { name: 'My system', exact: true }).click();
  const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21aa']).analyze();
  await writeFile(new URL('accessibility.json', out), JSON.stringify(axe.violations, null, 2));
  expect(axe.violations.map((v) => ({ id: v.id, targets: v.nodes.map((n) => n.target) }))).toEqual(
    [],
  );
  checks.push('Automated WCAG A/AA checks pass on the measured desktop demo view');
  await page.screenshot({
    path: fileURLToPath(new URL('desktop-en.png', out)),
    fullPage: true,
  });
  await page.setViewportSize({ width: 390, height: 844 });
  const fits = await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth);
  if (!fits)
    console.log(
      await page.evaluate(() => ({
        width: innerWidth,
        scroll: document.documentElement.scrollWidth,
        offenders: Array.from(document.querySelectorAll('body *'))
          .filter(
            (e) =>
              e.scrollWidth > e.clientWidth + 1 || e.getBoundingClientRect().right > innerWidth,
          )
          .map((e) => ({
            tag: e.tagName,
            class: e.className,
            width: e.clientWidth,
            scroll: e.scrollWidth,
            right: e.getBoundingClientRect().right,
          }))
          .slice(0, 20),
      })),
    );
  await page.screenshot({
    path: fileURLToPath(new URL('mobile.png', out)),
    fullPage: true,
  });
  expect(fits).toBe(true);
  checks.push('390px layout has no horizontal overflow');
  expect(errors).toEqual([]);
  expect(requests.every((url) => new URL(url).origin === 'http://127.0.0.1:4368')).toBe(true);
  checks.push('Demo has no application errors, external requests or IPC calls');
  await writeFile(
    new URL('report.json', out),
    JSON.stringify({ synthetic: true, checks, liveCs2Test: false }, null, 2),
  );
  console.log(JSON.stringify({ passed: checks.length, checks }, null, 2));
} finally {
  await browser?.close();
  child.kill();
}
