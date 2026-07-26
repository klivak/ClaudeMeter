import React from 'react';
import { COLOR } from '../theme';
import { useLayout } from '../layout';
import { Split } from '../components/Split';
import { Headline, Kicker, Reveal, Sub } from '../components/Type';
import { MockSettings } from '../components/Mock';

const ROWS = ['Theme', 'Language', 'Compact mode', 'Icon style', 'Dashboard layout', 'Start with Windows'];

export const Settings: React.FC = () => {
  const { u, vw, vertical } = useLayout();
  return (
    <Split
      text={
        <>
          <Reveal>
            <Kicker color={COLOR.accent2}>Settings</Kicker>
          </Reveal>
          <Reveal delay={6}>
            <Headline size={vertical ? 5.6 : 5.8}>No config file to edit.</Headline>
          </Reveal>
          <Reveal delay={12}>
            <Sub>
              Theme, language, layout, tray icon style, autostart, the mini widget, colourblind
              patterns, update checks — all in the popup.
            </Sub>
          </Reveal>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: u * 1.1, marginTop: u * 1 }}>
            {ROWS.map((r, i) => (
              <Reveal key={r} delay={20 + i * 5}>
                <div
                  style={{
                    border: `1px solid ${COLOR.line}`,
                    background: `${COLOR.surface}cc`,
                    borderRadius: u * 0.9,
                    padding: `${u * 0.8}px ${u * 1.6}px`,
                    color: COLOR.inkDim,
                    fontSize: u * (vertical ? 2.2 : 1.9),
                  }}
                >
                  {r}
                </div>
              </Reveal>
            ))}
          </div>
        </>
      }
      visual={<MockSettings width={vertical ? vw(58) : vw(19)} />}
    />
  );
};
