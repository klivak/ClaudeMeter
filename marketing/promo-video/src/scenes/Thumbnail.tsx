import React from 'react';
import { AbsoluteFill, Img, staticFile } from 'remotion';
import { BRAND, COLOR, FONT } from '../theme';
import { MockDashboard } from '../components/Mock';

/** Chunky SVG arrow — a glyph like "→" vanishes at 210x118 sidebar size. */
const Arrow: React.FC<{ width: number; color: string }> = ({ width, color }) => (
  <svg width={width} height={width * 0.5} viewBox="0 0 100 50">
    <path d="M2 25 H70" stroke={color} strokeWidth="14" strokeLinecap="round" />
    <path d="M50 4 L84 25 L50 46" stroke={color} strokeWidth="14" strokeLinecap="round" strokeLinejoin="round" fill="none" />
  </svg>
);

const Base: React.FC<{ children: React.ReactNode; bg?: string }> = ({ children, bg }) => (
  <AbsoluteFill
    style={{
      background: bg ?? `radial-gradient(ellipse at 30% 20%, ${COLOR.accent}22, ${COLOR.bg} 60%)`,
    }}
  >
    {children}
  </AbsoluteFill>
);

/** A — problem (red bar) -> arrow -> answer (green dashboard), 3-word headline. */
export const ThumbnailA: React.FC = () => (
  <Base>
    <div style={{ position: 'absolute', left: 60, top: 90, display: 'flex', flexDirection: 'column', gap: 18 }}>
      <div
        style={{
          fontFamily: FONT.sans,
          fontWeight: 800,
          fontSize: 122,
          lineHeight: 0.95,
          color: COLOR.ink,
          letterSpacing: -3,
        }}
      >
        DON'T
        <br />
        HIT THE
        <br />
        <span style={{ color: COLOR.red }}>WALL</span>
      </div>
    </div>
    <div
      style={{
        position: 'absolute',
        right: -10,
        top: 175,
        width: 700,
        height: 278,
        overflow: 'hidden',
        borderRadius: 24,
        transform: 'rotate(-3deg)',
        boxShadow: `0 10px 40px #000000aa`,
      }}
    >
      <MockDashboard theme="dark" width={700} showCodex={false} showChart={false} drift={false} />
    </div>
    <div style={{ position: 'absolute', left: 500, top: 440 }}>
      <Arrow width={120} color={COLOR.accent} />
    </div>
  </Base>
);

/** B — the hero number filling the frame with a stamped annotation. */
export const ThumbnailB: React.FC = () => (
  <Base bg={`linear-gradient(120deg, ${COLOR.bg} 0%, ${COLOR.surfaceAlt} 100%)`}>
    <div
      style={{
        position: 'absolute',
        inset: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <div
        style={{
          fontFamily: FONT.sans,
          fontWeight: 800,
          fontSize: 460,
          color: COLOR.red,
          letterSpacing: -12,
          lineHeight: 1,
        }}
      >
        94%
      </div>
    </div>
    <div
      style={{
        position: 'absolute',
        left: 70,
        top: 70,
        fontFamily: FONT.sans,
        fontWeight: 800,
        fontSize: 96,
        color: COLOR.ink,
      }}
    >
      SEE IT
      <br />
      <span style={{ color: COLOR.accent }}>COMING</span>
    </div>
    <div
      style={{
        position: 'absolute',
        right: 60,
        bottom: 50,
        transform: 'rotate(-8deg)',
        border: `6px solid ${COLOR.green}`,
        borderRadius: 16,
        padding: '10px 28px',
        color: COLOR.green,
        fontFamily: FONT.mono,
        fontWeight: 700,
        fontSize: 44,
      }}
    >
      LIVE IN TRAY
    </div>
  </Base>
);

/** C — abstract symbol (giant question mark) + promise. */
export const ThumbnailC: React.FC = () => (
  <Base bg={`radial-gradient(ellipse at 70% 50%, ${COLOR.accent2}2a, ${COLOR.bg} 65%)`}>
    <div
      style={{
        position: 'absolute',
        right: 40,
        top: -40,
        fontFamily: FONT.sans,
        fontWeight: 800,
        fontSize: 620,
        color: `${COLOR.accent2}55`,
        lineHeight: 1,
      }}
    >
      ?
    </div>
    <div
      style={{
        position: 'absolute',
        left: 60,
        top: 110,
        fontFamily: FONT.sans,
        fontWeight: 800,
        fontSize: 130,
        lineHeight: 1,
        color: COLOR.ink,
      }}
    >
      HOW MUCH
      <br />
      IS <span style={{ color: COLOR.accent }}>LEFT?</span>
    </div>
    <div style={{ position: 'absolute', left: 66, bottom: 60, display: 'flex', alignItems: 'center', gap: 20 }}>
      <Img src={staticFile('brand/icon.png')} style={{ width: 90, height: 90 }} />
      <div style={{ fontFamily: FONT.sans, fontWeight: 700, fontSize: 52, color: COLOR.inkDim }}>{BRAND.name}</div>
    </div>
  </Base>
);
