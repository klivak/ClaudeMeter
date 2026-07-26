import { useVideoConfig } from 'remotion';

/**
 * Every size in this project is derived from the frame, so the same scene
 * components render correctly at 1920x1080 and at 1080x1920.
 */
export const useLayout = () => {
  const { width, height, fps, durationInFrames } = useVideoConfig();
  const vertical = height > width;
  const u = Math.min(width, height) / 100;
  return {
    width,
    height,
    fps,
    durationInFrames,
    vertical,
    /** 1 unit = 1% of the short edge. */
    u,
    /** horizontal percentage of the frame */
    vw: (n: number) => (width * n) / 100,
    /** vertical percentage of the frame */
    vh: (n: number) => (height * n) / 100,
    /** type scale — a bit larger on the vertical cut, which is watched on a phone */
    fs: (n: number) => u * n * (vertical ? 1.18 : 1),
    /** page padding, respecting the Shorts safe areas (12% top / 20% bottom) */
    pad: vertical ? width * 0.08 : width * 0.075,
    safeTop: vertical ? height * 0.12 : height * 0.08,
    safeBottom: vertical ? height * 0.2 : height * 0.08,
  };
};

export type Layout = ReturnType<typeof useLayout>;
