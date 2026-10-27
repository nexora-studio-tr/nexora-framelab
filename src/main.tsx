import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { Checkbox, Dialog, Progress, Select, Tabs } from 'radix-ui';
import {
  Activity,
  ArrowDownLeft,
  ArrowRight,
  Check,
  ChevronDown,
  CircleHelp,
  Cpu,
  Download,
  FlaskConical,
  FolderOpen,
  Gauge,
  HardDrive,
  Languages,
  Monitor,
  Play,
  RefreshCw,
  RotateCcw,
  ShieldCheck,
  Square,
  TriangleAlert,
  X,
} from 'lucide-react';
import { api, desktop } from './api';
import {
  active,
  estimateMinutes,
  metric,
  optimizeBlock,
  type Lang,
  type Mode,
  type Report,
  type Snapshot,
} from './types';
import { text, explain, phase } from './i18n';
import { ComparisonEvidence } from './ComparisonEvidence';
import './style.css';

function Choice({
  label,
  value,
  onChange,
  items,
  placeholder,
  disabled = false,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  items: { value: string; label: string }[];
  placeholder?: string;
  disabled?: boolean;
}) {
  return (
    <div className="field">
      <span className="field-label">{label}</span>
      <Select.Root value={value} onValueChange={onChange} disabled={disabled}>
        <Select.Trigger aria-label={label} className="select">
          <Select.Value placeholder={placeholder} />
          <Select.Icon>
            <ChevronDown size={15} />
          </Select.Icon>
        </Select.Trigger>
        <Select.Portal>
          <Select.Content position="popper" sideOffset={5} className="select-menu">
            <Select.Viewport>
              {items.map((item) => (
                <Select.Item key={item.value} value={item.value} className="select-item">
                  <Select.ItemText>{item.label}</Select.ItemText>
                  <Select.ItemIndicator>
                    <Check size={14} />
                  </Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
    </div>
  );
}
function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  children?: React.ReactNode;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="overlay" />
        <Dialog.Content className="dialog">
          <Dialog.Title>{title}</Dialog.Title>
          <Dialog.Description>{description}</Dialog.Description>
          {children}
          <Dialog.Close className="icon close" aria-label="Close / Kapat">
            <X size={19} />
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
function Chart({ report, lang }: { report: Report | null; lang: Lang }) {
  const t = text[lang],
    trial = report?.holdout ?? report?.trials[0],
    before = report?.baseline?.series ?? [],
    after = report?.comparison?.candidate.series ?? trial?.pairs.at(-1)?.candidate.series ?? [];
  const max = Math.max(16, ...before, ...after) * 1.15;
  const points = (values: number[]) =>
    values
      .map((v, i) => `${52 + (i / Math.max(values.length - 1, 1)) * 696},${229 - (v / max) * 192}`)
      .join(' ');
  return (
    <div className="chart-wrap">
      <div className="chart-caption">
        <span>{t.milliseconds}</span>
        <div>
          <i className="legend before" />
          {t.baseline}
          {after.length > 0 && (
            <>
              <i className="legend after" />
              {t.candidate}
            </>
          )}
        </div>
      </div>
      <svg viewBox="0 0 780 268" role="img" aria-label={t.chart}>
        <title>{t.chart}</title>
        {[0, 0.25, 0.5, 0.75, 1].map((f) => (
          <g key={f}>
            <line x1="52" x2="748" y1={229 - f * 192} y2={229 - f * 192} className="gridline" />
            <text x="8" y={233 - f * 192}>
              {(max * f).toFixed(0)}
            </text>
          </g>
        ))}
        {before.length > 0 && <polyline points={points(before)} className="trace before" />}
        {after.length > 0 && <polyline points={points(after)} className="trace after" />}
        <text x="52" y="258">
          0
        </text>
        <text x="748" y="258" textAnchor="end">
          {t.sampled}
        </text>
      </svg>
    </div>
  );
}

function App() {
  const [lang, setLang] = useState<Lang>(() =>
    localStorage.getItem('nfl-language') === 'en' ? 'en' : 'tr',
  );
  const t = text[lang];
  const [state, setState] = useState<Snapshot | null>(null),
    [error, setError] = useState(''),
    [success, setSuccess] = useState(''),
    [working, setWorking] = useState(false),
    [tab, setTab] = useState('system');
  const [profileId, setProfileId] = useState(''),
    [mode, setMode] = useState<Mode>('balanced'),
    [duration, setDuration] = useState('30'),
    [report, setReport] = useState<Report | null>(null);
  const [help, setHelp] = useState(false),
    [confirm, setConfirm] = useState(false),
    [consent, setConsent] = useState(false),
    [restoreTarget, setRestoreTarget] = useState<string | null>(null);
  const load = async () => {
    const next = await api.snapshot();
    setState(next);
    setProfileId(
      (previous) =>
        previous || (next.discovery.profiles.length === 1 ? next.discovery.profiles[0].id : ''),
    );
    setReport((previous) => {
      if (previous?.synthetic) return previous;
      if (previous) {
        return next.reports.find((r) => r.id === previous.id) ?? previous;
      }
      return next.reports[0] ?? null;
    });
  };
  useEffect(() => {
    void load().catch((e) => setError(String(e)));
    if (!desktop) return;
    const timer = setInterval(() => {
      void load().catch((e) => setError(String(e)));
    }, 2000);
    return () => clearInterval(timer);
  }, []);
  useEffect(() => {
    document.documentElement.lang = lang;
    localStorage.setItem('nfl-language', lang);
  }, [lang]);
  const perform = async (action: () => Promise<unknown>) => {
    setWorking(true);
    setError('');
    setSuccess('');
    try {
      await action();
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setWorking(false);
    }
  };
  const currentProfile = state?.discovery.profiles.find((p) => p.id === profileId),
    block = optimizeBlock(state, currentProfile, mode, !desktop),
    busy = active(state?.job ?? null);
  const hw = state?.hardware,
    trial = report?.holdout ?? report?.trials[0],
    before = report?.baseline,
    after = report?.comparison?.candidate ?? trial?.pairs.at(-1)?.candidate,
    decision = trial?.decision;
  const estimate = estimateMinutes(2, 5, Number(duration));
  const source = report?.synthetic
    ? t.synthetic
    : report?.source.startsWith('IMPORTED_CSV_')
      ? t.imported
      : t.actual;
  const reportLabel = (status: string) =>
    ({
      applied: t.applied,
      noGain: t.noGain,
      analysis: t.analysis,
      restored: t.restored,
      failed: t.failed,
      canceled: t.canceled,
      example: t.exampleStatus,
    })[status] ?? status;
  const showExample = () =>
    perform(async () => {
      setReport(await api.example());
    });
  const importCapture = () =>
    perform(async () => {
      const next = await api.importCapture();
      if (next) {
        setReport(next);
        setTab('results');
      }
    });
  const compareCaptures = () =>
    perform(async () => {
      const next = await api.compareCaptures();
      if (next) {
        setReport(next);
        setTab('results');
      }
    });
  const systemCard = (
    <section className="hardware" aria-label={t.scan}>
      <div className="hardware-heading">
        <span className="eyebrow">{t.scan}</span>
        <span className="quiet-badge">
          <span className="dot" />
          {desktop ? t.real : t.illustrative}
        </span>
      </div>
      <div className="hardware-grid">
        <article>
          <Cpu />
          <span className="eyebrow">{t.cpu}</span>
          <strong title={hw?.cpu}>{hw?.cpu || t.unknown}</strong>
          <small>
            {hw?.physicalCores || '—'} {t.cores} / {hw?.logicalCores || '—'} {t.threads}
          </small>
        </article>
        <article>
          <Gauge />
          <span className="eyebrow">{t.gpu}</span>
          <strong title={hw?.gpus[0]?.name}>{hw?.gpus[0]?.name || t.unknown}</strong>
          <small>
            {hw?.gpus[0]
              ? `${metric(hw.gpus[0].dedicatedBytes / 1024 ** 3)} GB ${t.dedicated}`
              : t.unknown}
          </small>
        </article>
        <article>
          <HardDrive />
          <span className="eyebrow">{t.ram}</span>
          <strong>{hw?.ramBytes ? `${metric(hw.ramBytes / 1024 ** 3)} GB` : t.unknown}</strong>
          <small>
            {hw?.ramBytes
              ? `${metric(hw.availableRamBytes / 1024 ** 3)} GB ${t.available}`
              : t.unknown}
          </small>
        </article>
        <article>
          <Monitor />
          <span className="eyebrow">{t.display}</span>
          <strong>{hw?.display || t.unknown}</strong>
          <small>{hw?.os || t.unknown}</small>
        </article>
      </div>
      {hw?.warnings.length ? (
        <div className="small-note">
          <TriangleAlert size={14} />
          {explain(hw.warnings[0], lang)}
          <code>{hw.warnings[0]}</code>
        </div>
      ) : null}
    </section>
  );
  const resultCard = (
    <section className="measurement">
      <div className="section-head">
        <div>
          <span className="eyebrow">
            {report?.synthetic ? t.dataset : report ? t.report : t.local}
          </span>
          <h2>{t.chart}</h2>
          <p>{report?.comparison ? t.comparisonChart : t.chartSub}</p>
        </div>
        <span className={`status-badge ${report?.synthetic ? 'amber' : ''}`}>
          {report?.synthetic ? t.synthetic : report ? reportLabel(report.status) : t.readOnly}
        </span>
      </div>
      {report?.comparison && (
        <ComparisonEvidence value={report.comparison} lang={lang} synthetic={report.synthetic} />
      )}
      {before ? (
        <>
          <div className="metrics">
            {[
              { name: t.fps, before: before.fps, after: after?.fps, unit: '' },
              { name: t.low, before: before.lowOne, after: after?.lowOne, unit: '' },
              { name: t.tail, before: before.p99Ms, after: after?.p99Ms, unit: 'ms' },
              report?.comparison
                ? {
                    name: t.rate,
                    before: (before.stutters * 60) / before.seconds,
                    after: after ? (after.stutters * 60) / after.seconds : undefined,
                    unit: '',
                  }
                : { name: t.stutters, before: before.stutters, after: after?.stutters, unit: '' },
            ].map((m, i) => (
              <article key={m.name}>
                <span className="eyebrow">{m.name}</span>
                <div>
                  <strong>{metric(m.after ?? m.before, i === 3 ? 0 : 1)}</strong>
                  <span>{m.unit}</span>
                </div>
                {m.after !== undefined ? (
                  <small>
                    {metric(m.before, i === 3 ? 0 : 1)} <ArrowRight size={12} />{' '}
                    {metric(m.after, i === 3 ? 0 : 1)}
                  </small>
                ) : (
                  <small>
                    {before.frames.toLocaleString(lang)} {t.frames}
                  </small>
                )}
              </article>
            ))}
          </div>
          <Chart report={report} lang={lang} />
          <div className="measurement-foot">
            <span>
              <ShieldCheck size={14} />
              {source}
            </span>
            {report?.synthetic ? (
              <span>{t.sampleWarning}</span>
            ) : (
              <span>{report?.map ?? t.captureNote}</span>
            )}
          </div>
        </>
      ) : (
        <div className="empty-chart">
          <Activity size={34} />
          <h3>{t.noMeasurement}</h3>
          <p>{t.noMeasurementNote}</p>
          <button className="button secondary" onClick={showExample} disabled={working}>
            <Play size={14} />
            {t.example}
          </button>
        </div>
      )}
    </section>
  );
  const experimentCard = (
    <aside className="experiment">
      <div className="section-head compact">
        <div>
          <span className="eyebrow">{t.game}</span>
          <h2>{t.plan}</h2>
        </div>
        <FlaskConical size={19} />
      </div>
      <Choice
        label={t.profile}
        value={profileId}
        onChange={setProfileId}
        items={
          state?.discovery.profiles.map((p) => ({
            value: p.id,
            label: `${p.label}${p.supported ? '' : ` · ${t.readOnly}`}`,
          })) ?? []
        }
        placeholder={state?.discovery.profiles.length ? t.selectProfile : t.noProfile}
        disabled={busy || !state?.discovery.profiles.length}
      />
      <Choice
        label={t.mode}
        value={mode}
        onChange={(v) => setMode(v as Mode)}
        items={[
          { value: 'balanced', label: t.balanced },
          { value: 'competitive', label: t.competitive },
          { value: 'preserve', label: t.preserve },
        ]}
        disabled={busy}
      />
      <p className="field-hint">
        {mode === 'balanced'
          ? t.balancedNote
          : mode === 'competitive'
            ? t.competitiveNote
            : t.preserveNote}
      </p>
      <button
        className={`replay-button ${state?.replaySelected ? 'selected' : ''}`}
        onClick={() => perform(() => api.chooseReplay())}
        disabled={!desktop || busy || working}
      >
        <FolderOpen size={16} />
        <span>{state?.replaySelected ? t.replayReady : t.chooseReplay}</span>
        {state?.replaySelected && <Check size={15} />}
      </button>
      <Choice
        label={t.duration}
        value={duration}
        onChange={setDuration}
        items={['15', '30', '60', '120'].map((v) => ({ value: v, label: `${v} ${t.seconds}` }))}
        disabled={busy}
      />
      <div className="experiment-rule">
        <ShieldCheck size={17} />
        <span>{t.pairs}</span>
      </div>
      {busy && state?.job ? (
        <div className="progress-area">
          <div>
            <strong>{phase(state.job.phase, lang)}</strong>
            <span>
              {state.job.completed}/{state.job.total || '—'}
            </span>
          </div>
          <Progress.Root
            className="progress"
            aria-label={t.progress}
            value={state.job.total ? (state.job.completed / state.job.total) * 100 : null}
          >
            <Progress.Indicator
              style={{
                transform: `translateX(-${100 - (state.job.total ? (state.job.completed / state.job.total) * 100 : 0)}%)`,
              }}
            />
          </Progress.Root>
          <button
            className="button danger"
            onClick={() => perform(() => api.cancel())}
            disabled={working || state.job.canceling}
          >
            <Square size={13} />
            {state.job.canceling ? t.stopping : t.stop}
          </button>
        </div>
      ) : (
        <>
          <button
            className="button primary"
            disabled={Boolean(block) || working}
            onClick={() => {
              setConsent(false);
              setConfirm(true);
            }}
          >
            <Play size={15} />
            {t.start}
            <ArrowRight size={15} />
          </button>
          {block && <p className="block-reason">{explain(block, lang)}</p>}
        </>
      )}
      <p className="commitment">
        <RotateCcw size={14} />
        {t.commitment}
      </p>
    </aside>
  );
  return (
    <Tabs.Root value={tab} onValueChange={setTab} className="app">
      <a className="skip" href="#main">
        Skip to content / İçeriğe geç
      </a>
      <aside className="rail">
        <div className="brand">
          <svg viewBox="0 0 40 40" aria-hidden="true">
            <path d="M7 31V9h7l12 17V9h7v22h-7L14 14v17z" fill="currentColor" />
          </svg>
          <div>
            <strong>nexora</strong>
            <span>FRAMELAB</span>
          </div>
        </div>
        <div className="rail-label">WORKSPACE</div>
        <Tabs.List aria-label="Workspace" className="nav">
          <Tabs.Trigger value="system">
            <Cpu size={17} />
            {t.system}
          </Tabs.Trigger>
          <Tabs.Trigger value="optimize">
            <FlaskConical size={17} />
            {t.optimize}
          </Tabs.Trigger>
          <Tabs.Trigger value="results">
            <Activity size={17} />
            {t.results}
            <span>{state?.reports.length || '0'}</span>
          </Tabs.Trigger>
        </Tabs.List>
        <div className="rail-bottom">
          <ShieldCheck size={20} />
          <p>{t.privacy}</p>
          <button onClick={() => setHelp(true)} className="help">
            <CircleHelp size={15} />
            {t.help}
          </button>
          <div className="version">
            <span>v0.1.0-alpha.2</span>
            <span>{t.alpha}</span>
          </div>
        </div>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <div>
            <span>FRAMELAB</span>
            <i>/</i>
            <span>{tab === 'system' ? t.system : tab === 'optimize' ? t.optimize : t.results}</span>
          </div>
          <button
            className="language"
            aria-label={lang === 'tr' ? 'Switch to English' : 'Türkçeye geç'}
            onClick={() => setLang(lang === 'tr' ? 'en' : 'tr')}
          >
            <Languages size={16} />
            {lang === 'tr' ? 'EN' : 'TR'}
          </button>
        </header>
        <main id="main">
          <div className="hero">
            <div>
              <span className="eyebrow">{t.local}</span>
              <h1>
                {t.title}
                <br />
                <em>{t.titleEm}</em>
              </h1>
              <p>{t.intro}</p>
            </div>
            <div className="hero-mark" aria-hidden="true">
              <span>01</span>
              <Activity />
              <small>measure → verify</small>
            </div>
          </div>
          {!desktop && (
            <div className="demo-banner">
              <FlaskConical size={17} />
              <strong>{t.demo}</strong>
              <span>{t.demoNote}</span>
            </div>
          )}
          {error && (
            <div className="notice error" role="alert">
              <TriangleAlert size={18} />
              <div>
                <strong>{explain(error, lang)}</strong>
                <details>
                  <summary>{t.details}</summary>
                  <code>{error}</code>
                </details>
              </div>
              <button className="icon" aria-label={t.close} onClick={() => setError('')}>
                <X size={16} />
              </button>
            </div>
          )}
          {success && (
            <div className="notice" role="status">
              <Check size={16} />
              {success}
            </div>
          )}
          {state?.recovery.map((r) => (
            <div className="notice error recovery" key={r.id}>
              <RotateCcw size={18} />
              <div>
                <strong>{t.recovery}</strong>
                <span>{r.keys.join(', ')}</span>
              </div>
              <button
                className="button secondary"
                disabled={busy || working}
                onClick={() => setRestoreTarget(`recovery:${r.id}`)}
              >
                {t.recover}
              </button>
            </div>
          ))}
          <Tabs.Content value="system">
            <div className="toolbar">
              <span className="quiet">{desktop ? t.real : t.illustrative}</span>
              <button
                className="button text"
                onClick={() =>
                  perform(async () => {
                    setState(await api.refresh());
                  })
                }
                disabled={working || busy}
              >
                <RefreshCw size={14} className={working ? 'spin' : ''} />
                {t.refresh}
              </button>
            </div>
            {systemCard}
            <div className="work-grid">
              {resultCard}
              {experimentCard}
            </div>
          </Tabs.Content>
          <Tabs.Content value="optimize">
            <div className="workflow">
              {[t.stage1, t.stage2, t.stage3, t.stage4].map((name, i) => (
                <div key={name}>
                  <span>{String(i + 1).padStart(2, '0')}</span>
                  <strong>{name}</strong>
                  {i < 3 && <ArrowRight size={17} />}
                </div>
              ))}
            </div>
            <div className="work-grid optimize-grid">
              <section className="explanation">
                <span className="eyebrow">{t.experimental}</span>
                <h2>{t.helpTitle}</h2>
                <p>{t.helpBody}</p>
                <div className="explanation-line">
                  <ShieldCheck />
                  <p>{t.helpFoot}</p>
                </div>
                <button
                  className="button secondary"
                  onClick={importCapture}
                  disabled={!desktop || working || busy}
                >
                  <FolderOpen size={15} />
                  {t.import}
                </button>
                <p className="field-hint">{t.captureNote}</p>
                {resultCard}
              </section>
              {experimentCard}
            </div>
          </Tabs.Content>
          <Tabs.Content value="results">
            <div className="result-toolbar">
              <div>
                <span className="eyebrow">{t.evidence}</span>
                <h2>{t.history}</h2>
              </div>
              <div className="report-actions">
                <button
                  className="button secondary"
                  disabled={working || busy}
                  onClick={compareCaptures}
                >
                  <Activity size={15} />
                  {desktop ? t.compare : t.compareExample}
                </button>
                <button
                  className="button secondary"
                  disabled={!desktop || working || busy}
                  onClick={importCapture}
                >
                  <FolderOpen size={15} />
                  {t.import}
                </button>
              </div>
            </div>
            <div className="history">
              {state?.reports.length ? (
                state.reports.map((r) => (
                  <button
                    className={`report-row ${report?.id === r.id ? 'selected' : ''}`}
                    key={r.id}
                    onClick={() => setReport(r)}
                  >
                    <span className="report-icon">
                      <Activity size={19} />
                    </span>
                    <div>
                      <strong>{reportLabel(r.status)}</strong>
                      <span>
                        {new Date(r.createdAt).toLocaleString(lang)} ·{' '}
                        {r.source.startsWith('IMPORTED_CSV_') ? t.imported : t.actual}
                      </span>
                    </div>
                    <span>{metric(r.baseline?.fps)} FPS</span>
                    <ArrowRight size={17} />
                  </button>
                ))
              ) : (
                <div className="empty-history">
                  <strong>{t.noReports}</strong>
                  <p>{t.noReportsNote}</p>
                  <button className="button text" disabled={working} onClick={showExample}>
                    {t.example}
                    <ArrowRight size={14} />
                  </button>
                </div>
              )}
            </div>
            {resultCard}
            {report && (
              <div className="evidence">
                <div>
                  <span className="eyebrow">{t.source}</span>
                  <strong>{source}</strong>
                  {report.synthetic && <p>{t.sampleWarning}</p>}
                </div>
                {decision && (
                  <div>
                    <span className="eyebrow">{t.confidence}</span>
                    <strong>
                      {decision.confidenceLowPct <= -100
                        ? '—'
                        : `${metric(decision.confidenceLowPct)}% … ${metric(decision.confidenceHighPct)}%`}
                    </strong>
                    <span>
                      {decision.pairs} {lang === 'tr' ? 'eşlenik koşu' : 'paired runs'}
                    </span>
                  </div>
                )}
                <div className="report-actions">
                  <button
                    className="button secondary"
                    onClick={() =>
                      perform(async () => {
                        if (await api.export(report.id)) setSuccess(t.reportSaved);
                      })
                    }
                    disabled={!desktop || report.synthetic || working}
                  >
                    <Download size={15} />
                    {t.export}
                  </button>
                  {report.transactionId && report.status !== 'restored' && (
                    <button
                      className="button secondary"
                      onClick={() => setRestoreTarget(report.id)}
                      disabled={busy || working}
                    >
                      <RotateCcw size={15} />
                      {t.restore}
                    </button>
                  )}
                </div>
              </div>
            )}
          </Tabs.Content>
          {report?.synthetic && (
            <button
              className="button text hide-example"
              onClick={() => setReport(state?.reports[0] ?? null)}
            >
              <X size={13} />
              {t.hideExample}
            </button>
          )}
          <footer>
            <span>
              NEXORA <b>FRAMELAB</b>
            </span>
            <span>{t.footer}</span>
            <button className="button text" onClick={() => setHelp(true)}>
              <CircleHelp size={13} />
              {t.noLive}
            </button>
          </footer>
        </main>
      </div>
      <Modal open={help} onOpenChange={setHelp} title={t.helpTitle} description={t.helpBody}>
        <div className="modal-note">
          <ShieldCheck size={20} />
          <p>{t.helpFoot}</p>
        </div>
        <Dialog.Close className="button secondary">{t.close}</Dialog.Close>
      </Modal>
      <Modal
        open={confirm}
        onOpenChange={setConfirm}
        title={t.confirmTitle}
        description={t.confirmBody}
      >
        <div className="estimate">
          <span>{t.estimated}</span>
          <strong>
            {estimate[0]}–{estimate[1]} {t.minutes}
          </strong>
        </div>
        <label className="consent">
          <Checkbox.Root
            aria-label={t.consent}
            checked={consent}
            onCheckedChange={(v) => setConsent(v === true)}
          >
            <Checkbox.Indicator>
              <Check size={15} />
            </Checkbox.Indicator>
          </Checkbox.Root>
          <span>{t.consent}</span>
        </label>
        <div className="dialog-actions">
          <Dialog.Close className="button secondary">{t.cancel}</Dialog.Close>
          <button
            className="button primary"
            disabled={!consent || working || Boolean(block)}
            onClick={() =>
              perform(async () => {
                await api.start(profileId, mode, Number(duration));
                setConfirm(false);
              })
            }
          >
            <Play size={14} />
            {t.confirm}
          </button>
        </div>
      </Modal>
      <Modal
        open={restoreTarget !== null}
        onOpenChange={(open) => {
          if (!open) setRestoreTarget(null);
        }}
        title={t.restore}
        description={
          lang === 'tr'
            ? 'Yalnızca FrameLab’in değiştirdiği alanlar geri alınır. Sonradan değişmiş bir değerle karşılaşılırsa işlem durur. CS2 kapalı olmalı.'
            : 'Only fields changed by FrameLab are restored. Conflicting later edits stop recovery. CS2 must be closed.'
        }
      >
        <div className="dialog-actions">
          <Dialog.Close className="button secondary">{t.cancel}</Dialog.Close>
          <button
            className="button primary"
            disabled={working || busy}
            onClick={() =>
              perform(async () => {
                if (restoreTarget) {
                  await api.restore(restoreTarget);
                  setRestoreTarget(null);
                  setSuccess(t.restoreDone);
                }
              })
            }
          >
            <RotateCcw size={14} />
            {t.restore}
          </button>
        </div>
      </Modal>
    </Tabs.Root>
  );
}
createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
