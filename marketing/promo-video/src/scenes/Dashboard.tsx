import React from 'react';
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { COLOR, FONT, SCREENS } from '../theme';
import { useLayout } from '../layout';
import { Screen } from '../components/AppWindow';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';

/** y is a fraction of the screenshot height; side is which way the label flies out. */
const CALLOUTS = [
  { y: 0.175, side: 'left' as const, label: '5-hour session', note: 'live % + countdown', color: COLOR.red },
  { y: 0.335, side: 'right' as const, label: 'Weekly (7-day)', note: 'cap + reset time', color: COLOR.green },
  { y: 0.53, side: 'left' as const, label: 'Codex panel', note: 'from your local ~/.codex logs', color: COLOR.codex },
  { y: 0.79, side: 'right' as const, label: 'Usage history', note: '24h / 7d / 30d chart', color: COLOR.accent2 },
];

/** Step 02 — one left-click and everything is on one card. */
export const Dashboard: React.FC = () => {
  const frame = useCurrentFrame();
  const { u, fs, vertical, vh, vw, safeTop, safeBottom, pad } = useLayout();
  const s = SCREENS.dark;

  if (vertical) {
    const w = vw(62);
    return (
      <AbsoluteFill
        style={{
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: u * 3,
          paddingTop: safeTop,
          paddingBottom: safeBottom,
          paddingLeft: pad,
          paddingRight: pad,
        }}
      >
        <Reveal>
          <Kicker>Step 02 · Dashboard</Kicker>
        </Reveal>
        <Reveal delay={5} style={{ textAlign: 'center' }}>
          <Headline size={5.6}>Every limit, one click.</Headline>
        </Reveal>
        <Reveal delay={12}>
          <Screen screen="dark" width={w} />
        </Reveal>
        <Reveal delay={20} style={{ textAlign: 'center' }}>
          <Sub size={2.3}>Session · Weekly · Codex · History</Sub>
        </Reveal>
      </AbsoluteFill>
    );
  }

  const screenW = vw(20);
  const screenH = (screenW * s.h) / s.w;
  const top = (vh(100) - screenH) / 2;

  return (
    <AbsoluteFill style={{ alignItems: 'center', justifyContent: 'center' }}>
      <div style={{ position: 'absolute', left: pad, top: vh(18), width: vw(26) }}>
        <Reveal>
          <Kicker>Step 02 · Dashboard</Kicker>
        </Reveal>
        <Reveal delay={6} style={{ marginTop: u * 1.6 }}>
          <Headline size={5.4}>Every limit on one card.</Headline>
        </Reveal>
        <Reveal delay={14} style={{ marginTop: u * 1.6 }}>
          <Sub size={2.2}>Left-click the tray icon. ESC closes it, F5 refreshes it.</Sub>
        </Reveal>
      </div>

      <Screen screen="dark" width={screenW} />

      {CALLOUTS.map((c, i) => {
        const delay = 30 + i * 14;
        const o = interpolate(frame, [delay, delay + 12], [0, 1], {
          extrapolateLeft: 'clamp',
          extrapolateRight: 'clamp',
        });
        const dir = c.side === 'left' ? -1 : 1;
        return (
          <div
            key={c.label}
            style={{
              position: 'absolute',
              top: top + screenH * c.y,
              left: '50%',
              transform: `translate(${dir * (screenW / 2 + u * 2)}px, -50%) translateX(${dir === -1 ? '-100%' : '0'})`,
              display: 'flex',
              flexDirection: dir === -1 ? 'row' : 'row-reverse',
              alignItems: 'center',
              gap: u * 1.2,
              opacity: o,
            }}
          >
            <div style={{ textAlign: dir === -1 ? 'right' : 'left' }}>
              <div style={{ fontFamily: FONT.sans, fontWeight: 700, fontSize: fs(2.4), color: COLOR.ink }}>
                {c.label}
              </div>
              <div style={{ fontFamily: FONT.mono, fontSize: fs(1.5), color: COLOR.muted, marginTop: u * 0.4 }}>
                {c.note}
              </div>
            </div>
            <div style={{ width: u * 6 * o, height: 2, background: c.color, borderRadius: 2 }} />
          </div>
        );
      })}
    </AbsoluteFill>
  );
};
