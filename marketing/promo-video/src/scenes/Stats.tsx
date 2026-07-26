import React from 'react';
import { AbsoluteFill, interpolate, useCurrentFrame } from 'remotion';
import { COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';
import { Plate } from '../components/AppWindow';

/** Numbers quoted from README.md — RAM and binary size measured in Task Manager. */
const TILES = [
  { to: 1.9, suffix: ' MB', label: 'RAM in Task Manager', decimals: 1, color: COLOR.green },
  { to: 3, suffix: ' MB', label: 'Single portable .exe', decimals: 0, color: COLOR.accent },
  { to: 40, suffix: '', label: 'Languages', decimals: 0, color: COLOR.accent2 },
  { to: 0, suffix: '', label: 'Telemetry, ads, accounts', decimals: 0, color: COLOR.amber },
];

export const Stats: React.FC = () => {
  const frame = useCurrentFrame();
  const { u, fs, vertical, pad, safeTop, safeBottom, vw } = useLayout();

  return (
    <AbsoluteFill
      style={{
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: u * 2.6,
        paddingLeft: pad,
        paddingRight: pad,
        paddingTop: safeTop,
        paddingBottom: safeBottom,
      }}
    >
      <Reveal>
        <Kicker color={COLOR.green}>Why Rust</Kicker>
      </Reveal>
      <Reveal delay={5} style={{ textAlign: 'center' }}>
        <Headline size={vertical ? 5.8 : 6}>Lighter than Notepad.</Headline>
      </Reveal>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: vertical ? '1fr 1fr' : 'repeat(4, 1fr)',
          gap: u * 2,
          width: vertical ? '100%' : vw(74),
          marginTop: u * 1,
        }}
      >
        {TILES.map((t, i) => {
          const delay = 16 + i * 8;
          const v = interpolate(frame, [delay, delay + 30], [0, t.to], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
          });
          return (
            <Reveal key={t.label} delay={delay}>
              <div
                style={{
                  border: `1px solid ${COLOR.line}`,
                  background: `${COLOR.surface}cc`,
                  borderRadius: u * 1.4,
                  padding: u * 2.2,
                  textAlign: 'center',
                }}
              >
                <div
                  style={{
                    fontFamily: FONT.sans,
                    fontWeight: 800,
                    fontSize: fs(5),
                    color: t.color,
                    letterSpacing: -fs(0.12),
                  }}
                >
                  {v.toFixed(t.decimals)}
                  {t.suffix}
                </div>
                <div style={{ fontSize: fs(1.7), color: COLOR.muted, marginTop: u * 0.8, fontFamily: FONT.sans }}>
                  {t.label}
                </div>
              </div>
            </Reveal>
          );
        })}
      </div>

      {!vertical && (
        <Reveal delay={54}>
          <Plate screen="taskManager" width={vw(40)} delay={54} />
        </Reveal>
      )}

      <Reveal delay={vertical ? 50 : 70} style={{ textAlign: 'center', maxWidth: vertical ? '100%' : vw(66) }}>
        <Sub size={1.9} color={COLOR.muted}>
          Needs Claude Code installed and logged in — ClaudeMeter reads that token locally, never
          your prompts. Codex usage comes from your own <span style={{ fontFamily: FONT.mono }}>~/.codex</span> logs, not an OpenAI API.
        </Sub>
      </Reveal>
    </AbsoluteFill>
  );
};
