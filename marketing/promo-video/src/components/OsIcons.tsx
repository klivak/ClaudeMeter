import React from 'react';

/** The classic 2x2 Windows logo — plain geometric squares, no raster asset needed. */
export const WindowsIcon: React.FC<{ size: number; color: string }> = ({ size, color }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <rect x="2" y="2" width="9" height="9" rx="0.6" fill={color} />
    <rect x="13" y="2" width="9" height="9" rx="0.6" fill={color} />
    <rect x="2" y="13" width="9" height="9" rx="0.6" fill={color} />
    <rect x="13" y="13" width="9" height="9" rx="0.6" fill={color} />
  </svg>
);

/**
 * A simplified apple silhouette (body + bite + stem + leaf) built from basic
 * shapes rather than a traced trademark path — reads as "macOS" at a glance
 * without reproducing Apple's exact logo artwork.
 */
export const AppleIcon: React.FC<{ size: number; color: string; bg: string }> = ({ size, color, bg }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <ellipse cx="12" cy="14.5" rx="7.3" ry="6.9" fill={color} />
    <circle cx="16.7" cy="9.8" r="3.7" fill={bg} />
    <ellipse cx="9.4" cy="3.6" rx="2.5" ry="1.25" fill={color} transform="rotate(-30 9.4 3.6)" />
    <rect x="11.3" y="2" width="1.5" height="4.6" rx="0.75" fill={color} transform="rotate(10 12.05 4.3)" />
  </svg>
);
