import React from 'react';
import { AbsoluteFill } from 'remotion';
import { BRAND, COLOR, FONT } from '../theme';
import { useLayout } from '../layout';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';

/** The 40 shipped locales, in their own names — straight from src/i18n/. */
const LANGS = [
  'English', 'Українська', 'Español', 'Deutsch', 'Français', 'Português', 'Italiano', 'Polski',
  'Nederlands', 'Svenska', 'Norsk', 'Dansk', 'Suomi', 'Čeština', 'Slovenčina', 'Magyar',
  'Română', 'Български', 'Hrvatski', 'Српски', 'Eesti', 'Latviešu', 'Lietuvių', 'Català',
  'Ελληνικά', 'Türkçe', 'Русский', '日本語', '한국어', '简体中文', 'ภาษาไทย', 'Tiếng Việt',
  'Bahasa Indonesia', 'Bahasa Melayu', 'Filipino', 'हिन्दी', 'বাংলা', 'العربية', 'עברית', 'فارسی',
];

export const Languages: React.FC = () => {
  const { u, fs, vertical, pad, safeTop, safeBottom, vw } = useLayout();
  const cols = vertical ? 4 : 8;

  return (
    <AbsoluteFill
      style={{
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        gap: u * 2.4,
        paddingLeft: pad,
        paddingRight: pad,
        paddingTop: safeTop,
        paddingBottom: safeBottom,
      }}
    >
      <Reveal>
        <Kicker>Localization</Kicker>
      </Reveal>
      <Reveal delay={5} style={{ textAlign: 'center' }}>
        <Headline size={vertical ? 6 : 6.2}>
          <span style={{ color: COLOR.accent }}>{BRAND.languages}</span> languages, fully translated.
        </Headline>
      </Reveal>
      <Reveal delay={11} style={{ textAlign: 'center' }}>
        <Sub size={2.2}>Auto-detects your system language. Every locale carries every string.</Sub>
      </Reveal>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${cols}, 1fr)`,
          gap: u * 1.1,
          marginTop: u * 1.6,
          width: vertical ? '100%' : vw(78),
        }}
      >
        {LANGS.map((l, i) => (
          <Reveal key={l} delay={18 + i * 1.6}>
            <div
              style={{
                border: `1px solid ${COLOR.line}`,
                background: `${COLOR.surface}b0`,
                borderRadius: u * 0.8,
                padding: `${u * 0.9}px ${u * 0.6}px`,
                textAlign: 'center',
                fontFamily: FONT.sans,
                fontSize: fs(1.55),
                color: COLOR.inkDim,
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
              }}
            >
              {l}
            </div>
          </Reveal>
        ))}
      </div>
    </AbsoluteFill>
  );
};
