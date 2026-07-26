import React from 'react';
import { AbsoluteFill, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from 'remotion';
import { BRAND, COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline, Reveal, Sub } from '../components/Type';

export const Brand: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { u, fs, vertical, pad } = useLayout();
  const pop = spring({ frame, fps, config: { damping: 12, mass: 0.6 } });
  const size = u * (vertical ? 22 : 17);

  return (
    <AbsoluteFill
      style={{
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: u * 2.2,
        paddingLeft: pad,
        paddingRight: pad,
        textAlign: 'center',
      }}
    >
      <Img
        src={staticFile('brand/icon.png')}
        style={{
          width: size,
          height: size,
          transform: `scale(${interpolate(pop, [0, 1], [0.4, 1])})`,
          opacity: pop,
          filter: `drop-shadow(0 ${u * 1.6}px ${u * 4}px ${COLOR.bg})`,
        }}
      />
      <Reveal delay={10}>
        <Headline size={vertical ? 8 : 8.4}>{BRAND.name}</Headline>
      </Reveal>
      <Reveal delay={18}>
        <Sub size={2.7}>{BRAND.tagline}</Sub>
      </Reveal>
      <Reveal delay={28} style={{ marginTop: u * 1.4 }}>
        <div
          style={{
            fontFamily: FONT.mono,
            fontWeight: 700,
            fontSize: fs(2.2),
            color: COLOR.accent,
            border: `1px solid ${COLOR.accent}66`,
            background: `${COLOR.accent}14`,
            borderRadius: u * 10,
            padding: `${u * 1.1}px ${u * 2.6}px`,
          }}
        >
          BUILT IN RUST · UNDER 10 MB RAM
        </div>
      </Reveal>
    </AbsoluteFill>
  );
};
