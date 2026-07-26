import React from 'react';
import { AbsoluteFill, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from 'remotion';
import { BRAND, COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline, Reveal, Sub } from '../components/Type';
import { WindowsIcon, AppleIcon } from '../components/OsIcons';

const USE_OFFICIAL_BADGE = false;

const PlatformButton: React.FC<{ os: 'windows' | 'macos' }> = ({ os }) => {
  const { u, fs } = useLayout();
  const iconSize = fs(2.6);
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: u * 1.1,
        padding: `${u * 1.5}px ${u * 2.4}px`,
        borderRadius: u * 1.1,
        background: COLOR.accent,
        boxShadow: `0 ${u * 1.6}px ${u * 4}px ${COLOR.accent}44`,
      }}
    >
      {os === 'windows' ? (
        <WindowsIcon size={iconSize} color="#08221d" />
      ) : (
        <AppleIcon size={iconSize} color="#08221d" bg={COLOR.accent} />
      )}
      <div style={{ fontFamily: FONT.sans, fontWeight: 800, fontSize: fs(2.1), color: '#08221d', whiteSpace: 'nowrap' }}>
        {os === 'windows' ? 'Windows 10/11' : 'macOS 12+'}
      </div>
    </div>
  );
};

export const Cta: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const { u, fs, vertical, pad } = useLayout();
  const pop = spring({ frame, fps, config: { damping: 14, mass: 0.6 } });
  const size = u * (vertical ? 16 : 12);

  return (
    <AbsoluteFill
      style={{
        alignItems: 'center',
        justifyContent: 'center',
        flexDirection: 'column',
        gap: u * 2,
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
          transform: `scale(${interpolate(pop, [0, 1], [0.5, 1])})`,
          opacity: pop,
        }}
      />
      <Reveal delay={8}>
        <Headline size={vertical ? 6.4 : 6.6}>{BRAND.name}</Headline>
      </Reveal>
      <Reveal delay={14}>
        <Sub size={2.6}>{BRAND.tagline}</Sub>
      </Reveal>
      <Reveal delay={22} style={{ marginTop: u * 1.4 }}>
        {USE_OFFICIAL_BADGE ? (
          <Img
            src={staticFile('brand/badge.png')}
            style={{ height: u * (vertical ? 12 : 9), filter: `drop-shadow(0 ${u}px ${u * 3}px ${COLOR.bg})` }}
          />
        ) : (
          <div style={{ display: 'flex', flexDirection: vertical ? 'column' : 'row', gap: u * 1.6 }}>
            <PlatformButton os="windows" />
            <PlatformButton os="macos" />
          </div>
        )}
      </Reveal>
      <Reveal delay={30}>
        <div
          style={{
            fontFamily: FONT.mono,
            fontSize: fs(1.9),
            color: COLOR.muted,
            marginTop: u * 0.6,
          }}
        >
          Free & open source · {BRAND.repo}
        </div>
      </Reveal>
      <Reveal delay={36}>
        <div style={{ fontFamily: FONT.sans, fontSize: fs(1.7), color: COLOR.muted }}>{BRAND.studio}</div>
      </Reveal>
    </AbsoluteFill>
  );
};
