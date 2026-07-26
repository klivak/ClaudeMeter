import React from 'react';
import { AbsoluteFill, Audio, Sequence, staticFile, interpolate } from 'remotion';
import { SHORTS_SCENE, SHORTS_TOTAL_FRAMES, WITH_MUSIC, SHORTS_MUSIC_START_SECONDS, MUSIC_LEVEL, FPS } from './theme';
import { Stage } from './components/Stage';
import { SceneFade } from './components/SceneFade';
import { Hook } from './scenes/Hook';
import { Brand } from './scenes/Brand';
import { Tray } from './scenes/Tray';
import { Dashboard } from './scenes/Dashboard';
import { Examples } from './scenes/Examples';
import { Stats } from './scenes/Stats';
import { Cta } from './scenes/Cta';

/** Cut, not compressed: drops Settings/Languages, keeps 2 example cards, faster beats (§4c). */
const SCENES = [
  { key: 'hook', node: <Hook /> },
  { key: 'brand', node: <Brand /> },
  { key: 'tray', node: <Tray /> },
  { key: 'dashboard', node: <Dashboard /> },
  { key: 'examples', node: <Examples count={2} duration={SHORTS_SCENE.examples.duration} /> },
  { key: 'stats', node: <Stats /> },
  { key: 'cta', node: <Cta /> },
] as const;

const musicVolume = (frame: number) => {
  const envelope = interpolate(frame, [0, 20, SHORTS_TOTAL_FRAMES - 90, SHORTS_TOTAL_FRAMES - 1], [0, 1, 1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return MUSIC_LEVEL * envelope * envelope;
};

export const Shorts: React.FC = () => (
  <AbsoluteFill>
    <Stage />
    {SCENES.map((s) => {
      const { from, duration } = SHORTS_SCENE[s.key];
      return (
        <Sequence key={s.key} from={from} durationInFrames={duration} name={s.key}>
          <SceneFade>{s.node}</SceneFade>
        </Sequence>
      );
    })}
    {WITH_MUSIC && (
      <Audio
        src={staticFile('audio/music.mp3')}
        trimBefore={Math.round(SHORTS_MUSIC_START_SECONDS * FPS)}
        trimAfter={Math.round(SHORTS_MUSIC_START_SECONDS * FPS) + SHORTS_TOTAL_FRAMES}
        volume={musicVolume}
      />
    )}
  </AbsoluteFill>
);
