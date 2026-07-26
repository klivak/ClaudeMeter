import React from 'react';
import { Img, staticFile } from 'remotion';
import { COLOR, SCREENS, ScreenKey } from '../theme';
import { useLayout } from '../layout';

/**
 * Shows a region of a real screenshot at a chosen width. Region is given in the
 * screenshot's own pixels, so it stays correct if an asset is replaced.
 */
export const Crop: React.FC<{
  screen: ScreenKey;
  region: { x: number; y: number; w: number; h: number };
  width: number;
  style?: React.CSSProperties;
}> = ({ screen, region, width, style }) => {
  const s = SCREENS[screen];
  const { u } = useLayout();
  const scale = width / region.w;
  return (
    <div
      style={{
        width,
        height: region.h * scale,
        overflow: 'hidden',
        position: 'relative',
        borderRadius: u * 1.2,
        border: `1px solid ${COLOR.line}`,
        boxShadow: `0 ${u * 2}px ${u * 6}px ${COLOR.bg}cc`,
        ...style,
      }}
    >
      <Img
        src={staticFile(s.src)}
        style={{
          position: 'absolute',
          width: s.w * scale,
          height: s.h * scale,
          left: -region.x * scale,
          top: -region.y * scale,
          maxWidth: 'none',
        }}
      />
    </div>
  );
};
