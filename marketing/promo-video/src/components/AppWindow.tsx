import React from 'react';
import { Img, staticFile, interpolate, useCurrentFrame } from 'remotion';
import { COLOR, SCREENS, ScreenKey } from '../theme';
import { useLayout } from '../layout';

/**
 * The desktop equivalent of a phone bezel: the popup already ships its own
 * chrome in the screenshot, so the frame is a thin border + a large soft shadow
 * + a faint accent glow. Height always derives from the real screenshot ratio.
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

/** A real screenshot inside the frame. */
export const Screen: React.FC<{
  screen: ScreenKey;
  width: number;
  drift?: boolean;
  glow?: string;
  style?: React.CSSProperties;
}> = ({ screen, width, drift, glow, style }) => {
  const s = SCREENS[screen];
  return (
    <WindowFrame width={width} ratio={s.h / s.w} drift={drift} glow={glow} style={style}>
      <Img src={staticFile(s.src)} style={{ width: '100%', height: '100%', display: 'block' }} />
    </WindowFrame>
  );
};

/** A bare screenshot with a soft shadow — for the small chrome-owning captures. */
export const Plate: React.FC<{
  screen: ScreenKey;
  width: number;
  delay?: number;
  style?: React.CSSProperties;
}> = ({ screen, width, delay = 0, style }) => {
  const s = SCREENS[screen];
  const frame = useCurrentFrame();
  const { u } = useLayout();
  const o = interpolate(frame, [delay, delay + 14], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <Img
      src={staticFile(s.src)}
      style={{
        width,
        height: (width * s.h) / s.w,
        borderRadius: u * 1.1,
        border: `1px solid ${COLOR.line}`,
        boxShadow: `0 ${u * 2}px ${u * 6}px ${COLOR.bg}cc`,
        opacity: o,
        display: 'block',
        ...style,
      }}
    />
  );
};
