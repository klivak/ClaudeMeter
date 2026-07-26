import React from 'react';
import { AbsoluteFill, interpolate, useCurrentFrame, useVideoConfig } from 'remotion';

/** Wraps every scene so cuts never hard-pop. Shorter fades on the vertical cut. */
export const SceneFade: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const frame = useCurrentFrame();
  const { durationInFrames, width, height } = useVideoConfig();
  const f = height > width ? 7 : 12;
  const opacity = interpolate(
    frame,
    [0, f, durationInFrames - f, durationInFrames - 1],
    [0, 1, 1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' },
  );
  return <AbsoluteFill style={{ opacity }}>{children}</AbsoluteFill>;
};
