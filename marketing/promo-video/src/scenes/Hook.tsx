import React from 'react';
import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from 'remotion';
import { COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline } from '../components/Type';

const LINE = 'refactor the auth module and update the tests';

/**
 * The pain, in the user's own world: a prompt being typed, then the limit wall.
 * Recreated in JSX because the typing has to animate — no screenshot can.
 */
export const Hook: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { u, fs, vertical, pad } = useLayout();

  const typed = LINE.slice(0, Math.max(0, Math.floor(interpolate(frame, [10, 55], [0, LINE.length], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  }))));
  const caret = Math.floor(frame / 8) % 2 === 0;
  const wall = spring({ frame: frame - 64, fps, config: { damping: 200, mass: 0.8 } });
  const shake = frame > 64 && frame < 78 ? Math.sin((frame - 64) * 1.8) * u * 0.3 : 0;
  const width = vertical ? undefined : u * 78;

  return (
    <AbsoluteFill
      style={{
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: u * 4,
        paddingLeft: pad,
        paddingRight: pad,
      }}
    >
      <div
        style={{
          width: width ?? '100%',
          maxWidth: '100%',
          background: `${COLOR.surface}f0`,
          border: `1px solid ${COLOR.line}`,
          borderRadius: u * 1.6,
          padding: u * 3,
          fontFamily: FONT.mono,
          fontSize: fs(2.1),
          color: COLOR.inkDim,
          boxShadow: `0 ${u * 2.5}px ${u * 7}px ${COLOR.bg}`,
          transform: `translateX(${shake}px)`,
        }}
      >
        <div style={{ color: COLOR.claude, fontWeight: 700 }}>
          &gt; <span style={{ color: COLOR.ink }}>{typed}</span>
          {caret && frame < 64 ? <span style={{ color: COLOR.accent }}>▍</span> : null}
        </div>
        <div
          style={{
            marginTop: u * 2.4,
            opacity: wall,
            transform: `translateY(${interpolate(wall, [0, 1], [u * 1.6, 0])}px)`,
            background: `${COLOR.red}1e`,
            border: `1px solid ${COLOR.red}66`,
            borderRadius: u * 1.1,
            padding: u * 1.8,
            color: COLOR.red,
            fontWeight: 700,
          }}
        >
          5-hour limit reached — resets in 1h 51m
        </div>
      </div>

      <div style={{ opacity: interpolate(frame, [92, 108], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }), textAlign: 'center' }}>
        <Headline size={vertical ? 5.4 : 5.6}>You never saw it coming.</Headline>
      </div>
    </AbsoluteFill>
  );
};
