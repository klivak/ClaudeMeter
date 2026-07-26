import React from 'react';
import { useCurrentFrame } from 'remotion';
import { COLOR } from '../theme';
import { useLayout } from '../layout';

/**
 * The desktop equivalent of a phone bezel: a thin gradient border + a large
 * soft shadow + a faint accent glow, wrapping whatever popup content sits inside.
 */
export const WindowFrame: React.FC<{
  width: number;
  ratio: number;
  drift?: boolean;
  glow?: string;
  radius?: number;
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ width, ratio, drift = true, glow = COLOR.accent, radius, children, style }) => {
  const frame = useCurrentFrame();
  const { u } = useLayout();
  const y = drift ? Math.sin(frame / 62) * u * 0.5 : 0;
  const r = radius ?? width * 0.035;
  return (
    <div
      style={{
        width,
        height: width * ratio,
        borderRadius: r,
        padding: width * 0.006,
        background: `linear-gradient(150deg, ${COLOR.surfaceHi}, ${COLOR.line}55 40%, ${COLOR.surface})`,
        boxShadow: `0 ${u * 3}px ${u * 8}px ${COLOR.bg}, 0 0 ${u * 12}px ${glow}22`,
        transform: `translateY(${y}px)`,
        overflow: 'hidden',
        ...style,
      }}
    >
      <div
        style={{
          width: '100%',
          height: '100%',
          borderRadius: r * 0.94,
          overflow: 'hidden',
          background: COLOR.surface,
          position: 'relative',
        }}
      >
        {children}
      </div>
    </div>
  );
};
