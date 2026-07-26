import React from 'react';
import { Composition, Still } from 'remotion';
import { loadFont as loadSans } from '@remotion/google-fonts/Inter';
import { loadFont as loadMono } from '@remotion/google-fonts/JetBrainsMono';
import { Promo } from './Promo';
import { Shorts } from './Shorts';
import { ThumbnailA, ThumbnailB, ThumbnailC } from './scenes/Thumbnail';
import { TOTAL_FRAMES, SHORTS_TOTAL_FRAMES, FPS } from './theme';

loadSans('normal', { weights: ['400', '500', '700', '800'], subsets: ['latin', 'latin-ext', 'cyrillic'] });
loadMono('normal', { weights: ['400', '700'], subsets: ['latin', 'latin-ext', 'cyrillic'] });

export const RemotionRoot: React.FC = () => (
  <>
    <Composition
      id="PromoEN"
      component={Promo}
      durationInFrames={TOTAL_FRAMES}
      fps={FPS}
      width={1920}
      height={1080}
    />
    <Composition
      id="ShortsEN"
      component={Shorts}
      durationInFrames={SHORTS_TOTAL_FRAMES}
      fps={FPS}
      width={1080}
      height={1920}
    />
    <Still id="ThumbnailA" component={ThumbnailA} width={1280} height={720} />
    <Still id="ThumbnailB" component={ThumbnailB} width={1280} height={720} />
    <Still id="ThumbnailC" component={ThumbnailC} width={1280} height={720} />
  </>
);
