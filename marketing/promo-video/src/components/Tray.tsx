import React from 'react';
import { COLOR, FONT } from '../theme';

export type IconStyle = 'number' | 'ring' | 'bar' | 'pie';

/** Threshold colours exactly as documented in Settings: <50 green, 50–79 amber, >=80 red. */
export const usageColor = (v: number) =>
  v >= 80 ? COLOR.red : v >= 50 ? COLOR.amber : COLOR.green;

/**
 * Recreation of the four tray icon styles (Number / Ring / Bar / Pie) so the
 * video can animate them. A static screenshot can't show the icon counting up.
 */
export const TrayIcon: React.FC<{
  value: number;
  size: number;
  variant?: IconStyle;
  dim?: boolean;
}> = ({ value, size, variant = 'number', dim = false }) => {
  const c = dim ? COLOR.muted : usageColor(value);
  const v = Math.max(0, Math.min(100, Math.round(value)));

  if (variant === 'number') {
    return (
      <div
        style={{
          width: size,
          height: size,
          borderRadius: size * 0.18,
          background: c,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily: FONT.sans,
          fontWeight: 800,
          fontSize: size * 0.62,
          color: '#12121c',
          letterSpacing: -size * 0.02,
        }}
      >
        {v}
      </div>
    );
  }

  if (variant === 'ring') {
    const r = size * 0.38;
    const circ = 2 * Math.PI * r;
    return (
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        <circle cx={size / 2} cy={size / 2} r={r} stroke={COLOR.surfaceHi} strokeWidth={size * 0.14} fill="none" />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          stroke={c}
          strokeWidth={size * 0.14}
          fill="none"
          strokeLinecap="round"
          strokeDasharray={`${(circ * v) / 100} ${circ}`}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
        />
      </svg>
    );
  }

  if (variant === 'bar') {
    return (
      <div
        style={{
          width: size,
          height: size,
          borderRadius: size * 0.18,
          background: COLOR.surfaceHi,
          display: 'flex',
          alignItems: 'flex-end',
          overflow: 'hidden',
        }}
      >
        <div style={{ width: '100%', height: `${v}%`, background: c }} />
      </div>
    );
  }

  const a = (v / 100) * 2 * Math.PI - Math.PI / 2;
  const R = size * 0.46;
  const cx = size / 2;
  const cy = size / 2;
  const large = v > 50 ? 1 : 0;
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
      <circle cx={cx} cy={cy} r={R} fill={COLOR.surfaceHi} />
      <path
        d={`M ${cx} ${cy} L ${cx} ${cy - R} A ${R} ${R} 0 ${large} 1 ${cx + R * Math.cos(a)} ${cy + R * Math.sin(a)} Z`}
        fill={c}
      />
    </svg>
  );
};

/** A minimal Windows 11 taskbar corner, so the tray icon has somewhere to live. */
export const TrayStrip: React.FC<{
  width: number;
  value: number;
  variant?: IconStyle;
  dim?: boolean;
  highlight?: number;
}> = ({ width, value, variant, dim, highlight = 0 }) => {
  const h = width * 0.13;
  const icon = h * 0.52;
  return (
    <div
      style={{
        width,
        height: h,
        borderRadius: h * 0.16,
        background: `${COLOR.surface}f2`,
        border: `1px solid ${COLOR.line}`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'flex-end',
        gap: icon * 0.75,
        paddingRight: icon,
        boxShadow: `0 ${h * 0.2}px ${h * 0.6}px ${COLOR.bg}`,
      }}
    >
      <div style={{ color: COLOR.muted, fontSize: icon * 0.8, fontFamily: FONT.sans }}>^</div>
      <div style={{ width: icon * 0.8, height: icon * 0.8, borderRadius: 3, background: COLOR.surfaceHi }} />
      <div
        style={{
          padding: icon * 0.22,
          borderRadius: icon * 0.3,
          background: highlight ? `${COLOR.accent}${Math.round(highlight * 40).toString(16).padStart(2, '0')}` : 'transparent',
          outline: highlight > 0.5 ? `2px solid ${COLOR.accent}` : 'none',
        }}
      >
        <TrayIcon value={value} size={icon} variant={variant} dim={dim} />
      </div>
      <div style={{ width: icon * 0.8, height: icon * 0.8, borderRadius: 3, background: COLOR.surfaceHi }} />
      <div
        style={{
          color: COLOR.inkDim,
          fontSize: icon * 0.5,
          fontFamily: FONT.sans,
          fontWeight: 500,
          lineHeight: 1.1,
          textAlign: 'right',
        }}
      >
        11:08
        <br />
        26.07.2026
      </div>
    </div>
  );
};
