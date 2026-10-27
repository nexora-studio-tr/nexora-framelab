import { Check, X } from 'lucide-react';
import type { CaptureComparison, Lang } from './types';
import { metric } from './types';
import { text } from './i18n';

const labels: Record<string, [string, string]> = {
  SAME_FRAME_METRIC: ['Aynı kare süresi metriği', 'Same frame-time metric'],
  DISTINCT_CAPTURES: ['Farklı kaynak kayıtları', 'Distinct source captures'],
  ENOUGH_FRAMES: ['Her kayıtta en az 1.000 kare', 'At least 1,000 frames in each capture'],
  SIMILAR_DURATION: ['Süre farkı en fazla %10', 'Duration difference within 10%'],
  NO_CAPTURE_QUALITY_WARNING: [
    'Yükleme veya çoklu akış uyarısı yok',
    'No loading or secondary-stream warning',
  ],
};

export function ComparisonEvidence({
  value,
  lang,
  synthetic = false,
}: {
  value: CaptureComparison;
  lang: Lang;
  synthetic?: boolean;
}) {
  const t = text[lang];
  const d = value.comparable ? value.deltas : null;
  const changes = [
    { label: t.fps, value: d?.fpsPct, unit: '%' },
    { label: t.low, value: d?.lowOnePct, unit: '%' },
    { label: t.tail, value: d?.p99MsPct, unit: '%' },
    { label: t.rate, value: d?.stuttersPerMinute, unit: ' / min' },
    { label: t.displayed, value: d?.displayedFpsPct, unit: '%' },
  ];
  const signed = (v: number | null | undefined, suffix: string) =>
    v == null || !Number.isFinite(v) ? '—' : `${v > 0 ? '+' : ''}${metric(v)}${suffix}`;
  return (
    <section className="comparison-evidence" aria-label={t.captureComparison}>
      <div className="comparison-heading">
        <h3>{t.captureComparison}</h3>
        <strong className="quiet-badge">
          {value.comparable ? t.observed : t.comparisonBlocked}
        </strong>
      </div>
      <p>{t.comparisonNote}</p>
      <dl className="comparison-deltas">
        {changes.map(({ label, value: v, unit }) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{signed(v, unit)}</dd>
          </div>
        ))}
      </dl>
      <small>{t.deltaHint}</small>
      <ul aria-label={t.comparisonChecks} className="comparison-checks">
        {value.checks.map((c) => (
          <li key={c.code} data-passed={c.passed}>
            {c.passed ? <Check size={15} aria-hidden="true" /> : <X size={15} aria-hidden="true" />}
            <span>{labels[c.code]?.[lang === 'tr' ? 0 : 1] ?? c.code}</span>
            <strong>{c.passed ? t.passed : t.blocked}</strong>
          </li>
        ))}
      </ul>
      <details>
        <summary>{t.provenance}</summary>
        {[
          { label: t.baseline, capture: value.baseline, frameMetric: value.baselineMetric },
          { label: t.candidate, capture: value.candidate, frameMetric: value.candidateMetric },
        ].map(({ label, capture, frameMetric }) => {
          return (
            <div className="comparison-source" key={label}>
              <strong>{label}</strong>
              <span>
                {capture.frames.toLocaleString(lang)} {t.frames} · {metric(capture.seconds)} s ·{' '}
                {frameMetric}
              </span>
              <code>
                {synthetic ? t.synthetic : 'SHA-256'}: {capture.sourceHash}
              </code>
            </div>
          );
        })}
      </details>
    </section>
  );
}
