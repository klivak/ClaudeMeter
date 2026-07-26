import React from 'react';
import { AbsoluteFill, Audio, Sequence, staticFile, interpolate } from 'remotion';
import { SCENE, TOTAL_FRAMES, WITH_MUSIC, MUSIC_START_SECONDS, MUSIC_LEVEL, FPS } from './theme';
import { Stage } from './components/Stage';
import { SceneFade } from './components/SceneFade';
import { Hook } from './scenes/Hook';
import { Brand } from './scenes/Brand';
import { Tray } from './scenes/Tray';
import { Dashboard } from './scenes/Dashboard';
import { Examples } from './scenes/Examples';
import { Settings } from './scenes/Settings';
import { Languages } from './scenes/Languages';
import { Stats } from './scenes/Stats';
import { Cta } from './scenes/Cta';

const SCENES = [
  { key: 'hook', node: <Hook /> },
  { key: 'brand', node: <Brand /> },
  { key: 'tray', node: <Tray /> },
  { key: 'dashboard', node: <Dashboard /> },
  { key: 'examples', node: <Examples duration={SCENE.examples.duration} /> },
  { key: 'settings', node: <Settings /> },
  { key: 'languages', node: <Languages /> },
  { key: 'stats', node: <Stats /> },
  { key: 'cta', node: <Cta /> },
] as const;

/** Eased ~1s fade-in and ~4s fade-out to true silence, scaled to MUSIC_LEVEL at the plateau. */
const musicVolume = (frame: number) => {
  const envelope = interpolate(frame, [0, 30, TOTAL_FRAMES - 120, TOTAL_FRAMES - 1], [0, 1, 1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return MUSIC_LEVEL * envelope * envelope;
};

export const Promo: React.FC = () => (
  <AbsoluteFill>
    <Stage />
    {SCENES.map((s) => {
      const { from, duration } = SCENE[s.key];
      return (
        <Sequence key={s.key} from={from} durationInFrames={duration} name={s.key}>
          <SceneFade>{s.node}</SceneFade>
        </Sequence>
      );
    })}
    {WITH_MUSIC && (
      <Audio
        src={staticFile('audio/music.mp3')}
        trimBefore={Math.round(MUSIC_START_SECONDS * FPS)}
        trimAfter={Math.round(MUSIC_START_SECONDS * FPS) + TOTAL_FRAMES}
        volume={musicVolume}
      />
    )}
  </AbsoluteFill>
);
