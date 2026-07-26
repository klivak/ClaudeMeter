import React from 'react';
import { AbsoluteFill } from 'remotion';
import { useLayout } from '../layout';

/** Text + visual side by side at 16:9, stacked at 9:16. */
export const Split: React.FC<{
  text: React.ReactNode;
  visual: React.ReactNode;
  reverse?: boolean;
  textFlex?: number;
}> = ({ text, visual, reverse = false, textFlex = 1 }) => {
  const { vertical, pad, u, safeTop, safeBottom } = useLayout();

  if (vertical) {
    return (
      <AbsoluteFill
        style={{
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          paddingLeft: pad,
          paddingRight: pad,
          paddingTop: safeTop,
          paddingBottom: safeBottom,
          gap: u * 4,
        }}
      >
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: u * 1.6 }}>{text}</div>
        <div style={{ display: 'flex', justifyContent: 'center', width: '100%' }}>{visual}</div>
      </AbsoluteFill>
    );
  }

  return (
    <AbsoluteFill
      style={{
        flexDirection: reverse ? 'row-reverse' : 'row',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingLeft: pad,
        paddingRight: pad,
        gap: u * 6,
      }}
    >
      <div style={{ flex: textFlex, display: 'flex', flexDirection: 'column', gap: u * 2 }}>{text}</div>
      <div style={{ flex: 1, display: 'flex', justifyContent: 'center', alignItems: 'center' }}>{visual}</div>
    </AbsoluteFill>
  );
};
