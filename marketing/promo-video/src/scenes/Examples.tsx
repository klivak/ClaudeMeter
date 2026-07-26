import React from 'react';
import { AbsoluteFill, Sequence, interpolate, useCurrentFrame } from 'remotion';
import { COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';
import { Plate } from '../components/AppWindow';
import { Crop } from '../components/Crop';
import { TrayIcon, IconStyle } from '../components/Tray';

type CardProps = { index: number; total: number };

const Frame: React.FC<{
  kicker: string;
  title: string;
  body: string;
  accent: string;
  visual: React.ReactNode;
  index: number;
  total: number;
}> = ({ kicker, title, body, accent, visual, index, total }) => {
  const { u, vertical, pad, safeTop, safeBottom, vw } = useLayout();
  return (
    <AbsoluteFill
      style={{
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: u * 3,
        paddingLeft: pad,
        paddingRight: pad,
        paddingTop: safeTop,
        paddingBottom: safeBottom,
      }}
    >
      <Reveal>
        <Kicker color={accent}>{kicker}</Kicker>
      </Reveal>
      <Reveal delay={4} style={{ textAlign: 'center' }}>
        <Headline size={vertical ? 5.6 : 5.2}>{title}</Headline>
      </Reveal>
      <Reveal delay={10} style={{ textAlign: 'center', maxWidth: vertical ? '100%' : vw(52) }}>
        <Sub size={2.3}>{body}</Sub>
      </Reveal>
      <Reveal delay={16} style={{ marginTop: u * 1 }}>
        {visual}
      </Reveal>
      <div style={{ display: 'flex', gap: u * 1.1, marginTop: u * 1.4 }}>
        {Array.from({ length: total }).map((_, i) => (
          <div
            key={i}
            style={{
              width: i === index ? u * 3.2 : u * 1.1,
              height: u * 1.1,
              borderRadius: u,
              background: i === index ? accent : COLOR.surfaceHi,
            }}
          />
        ))}
      </div>
    </AbsoluteFill>
  );
};

const IconStyles: React.FC<CardProps> = (p) => {
  const frame = useCurrentFrame();
  const { u, fs, vw, vertical } = useLayout();
  const value = interpolate(frame, [16, 80], [30, 92], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  const size = vertical ? vw(13) : vw(6);
  const styles: IconStyle[] = ['number', 'ring', 'bar', 'pie'];
  return (
    <Frame
      {...p}
      kicker="Tray icon · 4 styles"
      title="Pick how it looks in the tray."
      body="Number, Ring, Bar or Pie — all colour-coded by threshold, all drawn at your DPI."
      accent={COLOR.amber}
      visual={
        <div style={{ display: 'flex', gap: u * 4, alignItems: 'flex-end' }}>
          {styles.map((s) => (
            <div key={s} style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: u * 1.2 }}>
              <TrayIcon value={value} size={size} variant={s} />
              <div
                style={{
                  fontFamily: FONT.mono,
                  fontSize: fs(1.6),
                  color: COLOR.muted,
                  textTransform: 'uppercase',
                  letterSpacing: u * 0.2,
                }}
              >
                {s}
              </div>
            </div>
          ))}
        </div>
      }
    />
  );
};

const Themes: React.FC<CardProps> = (p) => {
  const { u, vw, vertical } = useLayout();
  const w = vertical ? vw(19) : vw(11);
  return (
    <Frame
      {...p}
      kicker="Themes · 4 palettes"
      title="Dark, Light, Midnight, Sunset."
      body="Auto follows your Windows theme. Surfaces, accents, gradients and the mini widget all move together."
      accent={COLOR.accent2}
      visual={
        <div style={{ display: 'flex', gap: u * 2.2 }}>
          {(['dark', 'light', 'midnight', 'sunset'] as const).map((s, i) => (
            <Plate key={s} screen={s === 'dark' ? 'dark' : s === 'light' ? 'light' : s === 'midnight' ? 'midnight' : 'sunset'} width={w} delay={18 + i * 6} />
          ))}
        </div>
      }
    />
  );
};

const Alerts: React.FC<CardProps> = (p) => {
  const { vw, vertical } = useLayout();
  return (
    <Frame
      {...p}
      kicker="Notifications"
      title="A warning before the wall."
      body="Toasts at 50%, 75% and 90% by default — for Claude limits and for the Codex windows, each tracked separately."
      accent={COLOR.red}
      visual={<Plate screen="notification" width={vertical ? vw(78) : vw(34)} delay={18} />}
    />
  );
};

const Codex: React.FC<CardProps> = (p) => {
  const { vw, vertical } = useLayout();
  return (
    <Frame
      {...p}
      kicker="Codex · optional"
      title="Your Codex usage too."
      body="Read from your local ~/.codex session logs and drawn in its own teal, so it never reads as a Claude limit."
      accent={COLOR.codex}
      visual={<Crop screen="dark" region={{ x: 6, y: 288, w: 365, h: 128 }} width={vertical ? vw(80) : vw(36)} />}
    />
  );
};

const CARDS = [IconStyles, Themes, Alerts, Codex];

/** One card at a time, held long enough to read. count=2 for the Shorts cut. */
export const Examples: React.FC<{ count?: number; duration: number }> = ({ count = 4, duration }) => {
  const cards = CARDS.slice(0, count);
  const per = Math.floor(duration / cards.length);
  return (
    <AbsoluteFill>
      {cards.map((C, i) => (
        <Sequence key={i} from={i * per} durationInFrames={per} name={`example-${i + 1}`}>
          <C index={i} total={cards.length} />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};
