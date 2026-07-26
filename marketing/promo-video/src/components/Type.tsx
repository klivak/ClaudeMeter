import React from 'react';
import { interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { COLOR, FONT } from '../theme';
import { useLayout } from '../layout';

/** Spring-driven slide-up + fade. Stagger a list with delay={30 + i * 6}. */
export const Reveal: React.FC<{
  delay?: number;
  distance?: number;
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ delay = 0, distance, children, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { u } = useLayout();
  const d = distance ?? u * 2.6;
  const s = spring({ frame: frame - delay, fps, config: { damping: 200, mass: 0.7 } });
  return (
    <div
      style={{
        transform: `translateY(${interpolate(s, [0, 1], [d, 0])}px)`,
        opacity: interpolate(s, [0, 1], [0, 1]),
        ...style,
      }}
    >
      {children}
    </div>
  );
};

export const Kicker: React.FC<{ children: React.ReactNode; color?: string }> = ({
  children,
  color = COLOR.accent,
}) => {
  const { fs, u } = useLayout();
  return (
    <div
      style={{
        fontFamily: FONT.mono,
        fontWeight: 700,
        fontSize: fs(1.75),
        letterSpacing: u * 0.42,
        textTransform: 'uppercase',
        color,
      }}
    >
      {children}
    </div>
  );
};

export const Headline: React.FC<{
  children: React.ReactNode;
  size?: number;
  color?: string;
}> = ({ children, size = 6.4, color = COLOR.ink }) => {
  const { fs } = useLayout();
  return (
    <div
      style={{
        fontFamily: FONT.sans,
        fontWeight: 800,
        fontSize: fs(size),
        lineHeight: 1.06,
        letterSpacing: -fs(0.14),
        color,
      }}
    >
      {children}
    </div>
  );
};

export const Sub: React.FC<{ children: React.ReactNode; size?: number; color?: string }> = ({
  children,
  size = 2.5,
  color = COLOR.inkDim,
}) => {
  const { fs } = useLayout();
  return (
    <div
      style={{
        fontFamily: FONT.sans,
        fontWeight: 400,
        fontSize: fs(size),
        lineHeight: 1.45,
        color,
      }}
    >
      {children}
    </div>
  );
};
