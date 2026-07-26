import React from 'react';
import { interpolate, useCurrentFrame } from 'remotion';
import { useLayout } from '../layout';
import { Split } from '../components/Split';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';
import { Plate } from '../components/AppWindow';
import { TrayStrip } from '../components/Tray';
import { COLOR } from '../theme';

/** Step 01 — the app is a tray icon that shows the live number. */
export const Tray: React.FC = () => {
  const frame = useCurrentFrame();
  const { u, vertical, vw } = useLayout();
  const value = interpolate(frame, [12, 70], [12, 70], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const hover = interpolate(frame, [78, 92], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const stripW = vertical ? vw(84) : vw(38);

  return (
    <Split
      text={
        <>
          <Reveal>
            <Kicker>Step 01 · System tray</Kicker>
          </Reveal>
          <Reveal delay={6}>
            <Headline size={vertical ? 6 : 6.2}>
              The number is
              <br />
              always on screen.
            </Headline>
          </Reveal>
          <Reveal delay={14}>
            <Sub>
              Green under 50%, amber to 79%, red at 80% and above — and grey the moment the data is
              only cached, so a stale percentage never looks live.
            </Sub>
          </Reveal>
          <Reveal delay={22}>
            <Sub size={2.2} color={COLOR.muted}>
              Hover for every limit, reset time and plan.
            </Sub>
          </Reveal>
        </>
      }
      visual={
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: u * 2.4 }}>
          <div style={{ opacity: hover, transform: `translateY(${interpolate(hover, [0, 1], [u * 2, 0])}px)` }}>
            <Plate screen="tooltip" width={stripW * 0.72} delay={78} />
          </div>
          <TrayStrip width={stripW} value={value} highlight={hover} />
        </div>
      }
      reverse
    />
  );
};
