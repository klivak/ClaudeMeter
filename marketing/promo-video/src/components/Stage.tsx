import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { COLOR } from '../theme';
import { useLayout } from '../layout';

/** The dark backdrop everything sits on: grid drift, radial accent glow, vignette. */
export const Stage: React.FC = () => {
  const frame = useCurrentFrame();
  const { u, width } = useLayout();
  const cell = u * 6;
  const drift = (frame * 0.22) % cell;

  return (
    <AbsoluteFill style={{ backgroundColor: COLOR.bg }}>
      <AbsoluteFill
        style={{
          backgroundImage: `linear-gradient(${COLOR.surfaceAlt}55 1px, transparent 1px), linear-gradient(90deg, ${COLOR.surfaceAlt}55 1px, transparent 1px)`,
          backgroundSize: `${cell}px ${cell}px`,
          backgroundPosition: `${drift}px ${drift}px`,
          opacity: 0.5,
        }}
      />
      <AbsoluteFill
        style={{
          background: `radial-gradient(ellipse ${width * 0.75}px ${width * 0.5}px at 30% 22%, ${COLOR.accent}22, transparent 62%),
                       radial-gradient(ellipse ${width * 0.7}px ${width * 0.45}px at 78% 82%, ${COLOR.accent2}1e, transparent 60%)`,
        }}
      />
      <AbsoluteFill
        style={{
          background: `radial-gradient(ellipse at center, transparent 45%, ${COLOR.bg}dd 100%)`,
        }}
      />
    </AbsoluteFill>
  );
};
