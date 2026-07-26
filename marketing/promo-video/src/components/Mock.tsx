import React from 'react';
import { COLOR, FONT } from '../theme';
import { WindowFrame } from './AppWindow';

/**
 * Vector recreations of the real app UI (dashboard popup, tooltip, toast,
 * settings, Task Manager row) — colours and copy taken from screenshots/, but
 * drawn as text/divs instead of a rasterized PNG so they stay crisp at 4K.
 * The source screenshots are only ~380px wide; upscaled 5x for a 4K render
 * they turn to mush, while vector text and shapes scale for free.
 */

export type MockTheme = 'dark' | 'light' | 'midnight' | 'sunset';

const PALETTE: Record<MockTheme, {
  bg: string; bgAlt: string; header: string; text: string; dim: string; track: string; border: string; refreshBorder: string;
}> = {
  dark: { bg: '#1e1e2e', bgAlt: '#242438', header: '#eef0fa', text: '#eef0fa', dim: '#9aa0c0', track: '#3a3a52', border: '#33334a', refreshBorder: '#5b8def' },
  light: { bg: '#eef0f5', bgAlt: '#e4e7f0', header: '#20212b', text: '#20212b', dim: '#5b6172', track: '#d4d8e2', border: '#d2d6e0', refreshBorder: '#3a6fd8' },
  midnight: { bg: '#161c30', bgAlt: '#1b2240', header: '#e8ecff', text: '#e8ecff', dim: '#8d96c4', track: '#2a3358', border: '#2c3560', refreshBorder: '#5b8def' },
  sunset: { bg: '#241a1a', bgAlt: '#2b201f', header: '#f7ece6', text: '#f7ece6', dim: '#c9a190', track: '#453230', border: '#453230', refreshBorder: '#f08a6a' },
};

const BADGE_PURPLE = '#8a63f5';
const BADGE_TEAL = '#12b3a0';

/** k = pixels-per-reference-unit; the whole card is authored at a 377px reference width. */
const REF_W = 377;
export const MOCK_DASHBOARD_RATIO = 677 / 377;

const Divider: React.FC<{ color: string; k: number }> = ({ color, k }) => (
  <div style={{ height: 1, background: color, margin: `${9 * k}px 0` }} />
);

const Bar: React.FC<{ pct: number; track: string; k: number; kind: 'gradient' | 'green' | 'teal' }> = ({ pct, track, k, kind }) => {
  const fill =
    kind === 'gradient'
      ? `linear-gradient(90deg, ${COLOR.green}, ${COLOR.amber} 55%, ${COLOR.red})`
      : kind === 'teal'
      ? COLOR.codex
      : COLOR.green;
  return (
    <div style={{ height: 9 * k, borderRadius: 6 * k, background: track, overflow: 'hidden', marginTop: 6 * k }}>
      <div style={{ width: `${pct}%`, height: '100%', background: fill, borderRadius: 6 * k }} />
    </div>
  );
};

const Badge: React.FC<{ children: React.ReactNode; color: string; k: number }> = ({ children, color, k }) => (
  <div
    style={{
      background: color,
      color: '#fff',
      fontFamily: FONT.sans,
      fontWeight: 700,
      fontSize: 10.5 * k,
      padding: `${2.5 * k}px ${7 * k}px`,
      borderRadius: 5 * k,
      lineHeight: 1.4,
    }}
  >
    {children}
  </div>
);

const IconBtn: React.FC<{ k: number; border: string }> = ({ k, border }) => (
  <div style={{ width: 22 * k, height: 22 * k, borderRadius: 5 * k, border: `1px solid ${border}`, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
    <svg width={12 * k} height={12 * k} viewBox="0 0 12 12">
      <rect x="1" y="7" width="2.2" height="4" fill={COLOR.accent} />
      <rect x="4.9" y="4" width="2.2" height="7" fill={COLOR.accent} />
      <rect x="8.8" y="1.5" width="2.2" height="9.5" fill={COLOR.accent} />
    </svg>
  </div>
);

const StatusDot: React.FC<{ k: number; color?: string }> = ({ k, color = COLOR.green }) => (
  <div style={{ width: 22 * k, height: 22 * k, borderRadius: 5 * k, background: `${color}22`, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
    <div style={{ width: 8 * k, height: 8 * k, borderRadius: 999, background: color }} />
  </div>
);

const MiniChart: React.FC<{ k: number; p: typeof PALETTE.dark; accent: string }> = ({ k, p, accent }) => {
  const bars = [
    { h: 0, color: COLOR.amber },
    { h: 0, color: COLOR.amber },
    { h: 42, color: COLOR.amber },
    { h: 62, color: COLOR.amber },
    { h: 78, color: COLOR.red },
    { h: 92, color: COLOR.red },
  ];
  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 6 * k }}>
        <div style={{ fontFamily: FONT.sans, fontWeight: 700, fontSize: 13.5 * k, color: p.text }}>Usage History</div>
        <div style={{ fontFamily: FONT.sans, fontSize: 11 * k, color: p.dim, display: 'flex', gap: 5 * k }}>
          <span style={{ color: accent, fontWeight: 700 }}>24h</span>
          <span>|</span>
          <span>7d</span>
          <span>|</span>
          <span>30d</span>
        </div>
      </div>
      <div style={{ display: 'flex', border: `1px solid ${p.border}`, borderRadius: 6 * k, background: p.bgAlt, padding: `${8 * k}px ${8 * k}px ${5 * k}px` }}>
        <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between', fontFamily: FONT.sans, fontSize: 8 * k, color: p.dim, paddingRight: 5 * k, height: 62 * k }}>
          {['100%', '75%', '50%', '25%', '0%'].map((t) => (
            <div key={t}>{t}</div>
          ))}
        </div>
        <div style={{ flex: 1 }}>
          <div style={{ position: 'relative', height: 62 * k, display: 'flex', alignItems: 'flex-end', gap: 3 * k, borderLeft: `1px solid ${p.border}` }}>
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} style={{ position: 'absolute', left: `${(i + 1) * 16.5}%`, top: 0, bottom: 0, borderLeft: `1px dashed ${p.border}` }} />
            ))}
            {bars.map((b, i) => (
              <div key={i} style={{ width: 5 * k, marginLeft: i === 0 ? 8 * k : 2 * k, height: `${b.h}%`, background: b.color, borderRadius: 1 }} />
            ))}
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontFamily: FONT.sans, fontSize: 8 * k, color: p.dim, marginTop: 3 * k }}>
            {['24h', '18h', '12h', '6h', 'now'].map((t) => (
              <span key={t}>{t}</span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

/** The full dashboard popup — header, Claude section, Codex section, chart, footer. */
export const MockDashboard: React.FC<{
  theme?: MockTheme;
  width: number;
  showCodex?: boolean;
  showChart?: boolean;
  drift?: boolean;
}> = ({ theme = 'dark', width, showCodex = true, showChart = true, drift }) => {
  const p = PALETTE[theme];
  const k = width / REF_W;
  return (
    <WindowFrame width={width} ratio={MOCK_DASHBOARD_RATIO} drift={drift} glow={COLOR.accent}>
      <div style={{ background: p.bg, width: '100%', height: '100%', fontFamily: FONT.sans, display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `${13 * k}px ${16 * k}px`, borderBottom: `1px solid ${p.border}` }}>
          <div style={{ fontWeight: 700, fontSize: 17 * k, color: p.header }}>ClaudeMeter</div>
          <div style={{ display: 'flex', gap: 10 * k, color: p.dim, fontSize: 15 * k }}>
            <span>⚙</span>
            <span>✕</span>
          </div>
        </div>
        <div style={{ padding: `${13 * k}px ${16 * k}px`, flex: 1 }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 * k, fontSize: 13.5 * k, color: p.text, fontWeight: 600 }}>
              <span style={{ opacity: 0.7 }}>☁</span> CLAUDE · Plan <Badge color={BADGE_PURPLE} k={k}>Max 5X</Badge>
            </div>
            <div style={{ display: 'flex', gap: 6 * k }}>
              <IconBtn k={k} border={p.border} />
              <StatusDot k={k} />
            </div>
          </div>

          <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 12 * k, fontSize: 13.5 * k, color: p.text }}>
            <span>5-hour session</span>
            <span style={{ color: COLOR.red, fontWeight: 700 }}>94%</span>
          </div>
          <Bar pct={94} track={p.track} k={k} kind="gradient" />
          <div style={{ fontSize: 10.5 * k, color: p.dim, marginTop: 5 * k }}>resets in 1h 51m (01:00 PM)</div>

          <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 14 * k, fontSize: 13.5 * k, color: p.text }}>
            <span>Weekly (7-day)</span>
            <span style={{ color: COLOR.green, fontWeight: 700 }}>9%</span>
          </div>
          <Bar pct={9} track={p.track} k={k} kind="green" />
          <div style={{ fontSize: 10.5 * k, color: p.dim, marginTop: 5 * k }}>resets in 6d 15h (Wed 03:00 AM)</div>

          {showCodex && (
            <>
              <Divider color={p.border} k={k} />
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 * k, fontSize: 13.5 * k, color: p.text, fontWeight: 600 }}>
                  <span style={{ opacity: 0.7 }}>ⓘ</span> CODEX · Plan <Badge color={BADGE_TEAL} k={k}>Plus</Badge>
                </div>
                <div style={{ display: 'flex', gap: 6 * k }}>
                  <IconBtn k={k} border={p.border} />
                  <StatusDot k={k} />
                </div>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 12 * k, fontSize: 13.5 * k, color: p.text }}>
                <span>Weekly (7-day)</span>
                <span style={{ color: COLOR.codex, fontWeight: 700 }}>16%</span>
              </div>
              <Bar pct={16} track={p.track} k={k} kind="teal" />
              <div style={{ fontSize: 10.5 * k, color: p.dim, marginTop: 5 * k }}>resets in 6d 21h (Wed 08:44 AM)</div>
            </>
          )}

          {showChart && (
            <>
              <Divider color={p.border} k={k} />
              <MiniChart k={k} p={p} accent={theme === 'light' ? '#3a6fd8' : COLOR.accent2} />
            </>
          )}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `${11 * k}px ${16 * k}px`, borderTop: `1px solid ${p.border}` }}>
          <div style={{ fontSize: 10.5 * k, color: p.dim }}>Last updated: 11:08:14</div>
          <div style={{ border: `1px solid ${p.refreshBorder}`, color: p.refreshBorder, borderRadius: 6 * k, padding: `${4 * k}px ${11 * k}px`, fontSize: 11 * k, fontWeight: 600 }}>
            Refresh
          </div>
        </div>
      </div>
    </WindowFrame>
  );
};

/** Standalone Codex panel — same section as inside the dashboard, its own small card. */
export const MockCodexPanel: React.FC<{ width: number; theme?: MockTheme }> = ({ width, theme = 'dark' }) => {
  const p = PALETTE[theme];
  const k = width / (REF_W * 0.86);
  return (
    <div
      style={{
        width,
        background: p.bg,
        border: `1px solid ${p.border}`,
        borderRadius: 10 * k,
        padding: 18 * k,
        boxShadow: `0 ${k * 3}px ${k * 10}px #00000066`,
        fontFamily: FONT.sans,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 * k, fontSize: 17 * k, color: p.text, fontWeight: 600 }}>
          <span style={{ opacity: 0.7 }}>ⓘ</span> CODEX · Plan <Badge color={BADGE_TEAL} k={k * 1.3}>Plus</Badge>
        </div>
        <div style={{ display: 'flex', gap: 8 * k }}>
          <IconBtn k={k * 1.3} border={p.border} />
          <StatusDot k={k * 1.3} />
        </div>
      </div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 20 * k, fontSize: 17 * k, color: p.text }}>
        <span>Weekly (7-day)</span>
        <span style={{ color: COLOR.codex, fontWeight: 700 }}>16%</span>
      </div>
      <Bar pct={16} track={p.track} k={k * 1.3} kind="teal" />
      <div style={{ fontSize: 13 * k, color: p.dim, marginTop: 8 * k }}>resets in 6d 21h (Wed 08:44 AM)</div>
    </div>
  );
};

/** The tray hover tooltip. */
export const MockTooltip: React.FC<{ width: number }> = ({ width }) => {
  const k = width / 260;
  const line = (t: string) => (
    <div style={{ fontSize: 14 * k, color: '#dfe2f2', marginBottom: 10 * k }}>{t}</div>
  );
  return (
    <div
      style={{
        width,
        background: '#1a1a28',
        border: '1px solid #34344a',
        borderRadius: 6 * k,
        padding: 14 * k,
        boxShadow: `0 ${k * 4}px ${k * 14}px #000000aa`,
        fontFamily: FONT.sans,
      }}
    >
      <div style={{ fontSize: 15 * k, fontWeight: 700, color: '#fff', marginBottom: 4 * k }}>ClaudeMeter</div>
      {line('Claude (Max 5X)')}
      <div style={{ height: 6 * k }} />
      {line('5-hour session: 70% | 38m')}
      <div style={{ height: 6 * k }} />
      {line('Weekly (7-day): 40% | 3d 8h')}
      <div style={{ height: 6 * k }} />
      <div style={{ fontSize: 14 * k, color: '#dfe2f2' }}>Sonnet (7-day): 3% | 3d 9h</div>
    </div>
  );
};

/** The Windows toast notification. */
export const MockNotification: React.FC<{ width: number }> = ({ width }) => {
  const k = width / 364;
  return (
    <div
      style={{
        width,
        background: '#fbfbfd',
        borderRadius: 8 * k,
        padding: 16 * k,
        boxShadow: `0 ${k * 4}px ${k * 16}px #00000055`,
        fontFamily: FONT.sans,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 10 * k }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 * k }}>
          <div style={{ width: 16 * k, height: 16 * k, borderRadius: 999, background: COLOR.amber }} />
          <div style={{ fontSize: 13 * k, color: '#3a3a44' }}>ClaudeMeter</div>
        </div>
        <div style={{ display: 'flex', gap: 8 * k, color: '#8a8a96', fontSize: 13 * k }}>
          <span>⋯</span>
          <span>✕</span>
        </div>
      </div>
      <div style={{ fontSize: 15 * k, fontWeight: 700, color: '#1c1c22', marginBottom: 5 * k }}>ClaudeMeter — Usage Alert</div>
      <div style={{ fontSize: 13.5 * k, color: '#57575f', lineHeight: 1.5 }}>
        5-hour session: 50% (exceeded 50%)
        <br />
        resets in 1h 35m (02:00 PM)
      </div>
    </div>
  );
};

/** The Settings screen list. */
export const MockSettings: React.FC<{ width: number }> = ({ width }) => {
  const p = PALETTE.light;
  const k = width / 380;
  const rows: [string, string, boolean?][] = [
    ['Theme', 'Auto'],
    ['Language', 'Auto (English)'],
    ['Compact mode', '', false],
    ['Show Codex section', '', true],
    ['Start with Windows', '', false],
    ['Show widget', '', false],
    ['Check for updates', '', true],
    ['Accessibility patterns', '', true],
    ['Icon style', 'Number'],
    ['Dashboard layout', 'Standard'],
    ['Show extra usage', '', false],
    ['Show startup notification', '', true],
    ['Show login expiry warning', '', false],
    ['Usage link icons', '', true],
  ];
  return (
    <WindowFrame width={width} ratio={742 / 380} glow={COLOR.accent2}>
      <div style={{ background: '#eef0f5', width: '100%', height: '100%', fontFamily: FONT.sans, display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `${13 * k}px ${16 * k}px`, borderBottom: `1px solid ${p.border}` }}>
          <div style={{ color: '#3a6fd8', fontSize: 13 * k, fontWeight: 600 }}>← Back</div>
          <div style={{ fontWeight: 700, fontSize: 15 * k, color: '#20212b' }}>Settings</div>
          <div style={{ color: '#8a8f9e', fontSize: 14 * k }}>✕</div>
        </div>
        <div style={{ flex: 1 }}>
          {rows.map(([label, value, checked], i) => (
            <div
              key={label}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: `${8 * k}px ${16 * k}px`,
                background: i % 2 === 0 ? '#e7eaf2' : 'transparent',
              }}
            >
              <div style={{ fontSize: 12 * k, color: '#2a2b38' }}>{label}</div>
              {value ? (
                <div style={{ fontSize: 12 * k, color: '#3a6fd8', fontWeight: 600 }}>{value}</div>
              ) : (
                <div
                  style={{
                    width: 14 * k,
                    height: 14 * k,
                    borderRadius: 3 * k,
                    border: `1.5px solid ${checked ? '#3a6fd8' : '#9aa0b4'}`,
                    background: checked ? '#3a6fd8' : 'transparent',
                  }}
                />
              )}
            </div>
          ))}
        </div>
        <div style={{ padding: `${12 * k}px ${16 * k}px`, borderTop: `1px solid ${p.border}` }}>
          <div style={{ fontSize: 10.5 * k, color: '#6b7080' }}>ClaudeMeter v5.8.0 by klivak</div>
          <div style={{ fontSize: 10.5 * k, color: '#3a6fd8' }}>github.com/klivak/claudemeter</div>
        </div>
      </div>
    </WindowFrame>
  );
};

/** A simplified Task Manager row, cropped tight to just the ClaudeMeter/Claude process rows. */
export const MockTaskManager: React.FC<{ width: number }> = ({ width }) => {
  const k = width / 888;
  const cols = ['Name', 'Status', 'CPU', 'Memory', 'Disk', 'Network'];
  const rows = [
    { name: 'ClaudeMeter', color: '#3a6fd8', cpu: '0%', mem: '1.9 MB', disk: '0 MB/s', net: '0 Mbps' },
    { name: 'Claude', color: '#e8703a', cpu: '0%', mem: '0.9 MB', disk: '0 MB/s', net: '0 Mbps' },
  ];
  return (
    <div
      style={{
        width,
        background: '#1b1b22',
        borderRadius: 8 * k,
        border: '1px solid #333340',
        boxShadow: `0 ${k * 4}px ${k * 16}px #00000066`,
        fontFamily: FONT.sans,
        overflow: 'hidden',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 * k, padding: `${10 * k}px ${14 * k}px`, borderBottom: '1px solid #2c2c36' }}>
        <div style={{ width: 14 * k, height: 14 * k, borderRadius: 3 * k, background: '#3a6fd8' }} />
        <div style={{ fontSize: 12 * k, color: '#e8e8ee' }}>Task Manager</div>
        <div style={{ flex: 1, background: '#26262f', borderRadius: 5 * k, height: 18 * k, marginLeft: 20 * k, display: 'flex', alignItems: 'center', paddingLeft: 8 * k, fontSize: 10.5 * k, color: '#c9c9d4' }}>
          claude
        </div>
      </div>
      <div style={{ display: 'flex', gap: 12 * k, padding: `${9 * k}px ${14 * k}px`, fontSize: 10 * k, color: '#9a9aa8', borderBottom: '1px solid #2c2c36' }}>
        {cols.map((c, i) => (
          <div key={c} style={{ width: i === 0 ? 34 * k : 15 * k, flex: i === 0 ? 3 : 1 }}>{c}</div>
        ))}
      </div>
      {rows.map((r) => (
        <div key={r.name} style={{ display: 'flex', alignItems: 'center', gap: 12 * k, padding: `${8 * k}px ${14 * k}px`, background: '#213052' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 * k, flex: 3, fontSize: 11 * k, color: '#e8e8ee' }}>
            <div style={{ width: 12 * k, height: 12 * k, borderRadius: 3 * k, background: r.color }} /> {r.name}
          </div>
          <div style={{ flex: 1, fontSize: 11 * k, color: '#e8e8ee' }}>{r.cpu}</div>
          <div style={{ flex: 1, fontSize: 11 * k, color: '#e8e8ee' }}>{r.mem}</div>
          <div style={{ flex: 1, fontSize: 11 * k, color: '#e8e8ee' }}>{r.disk}</div>
          <div style={{ flex: 1, fontSize: 11 * k, color: '#e8e8ee' }}>{r.net}</div>
        </div>
      ))}
    </div>
  );
};
