import type { Component, Element } from 'solid-js';

interface SectionCardProps {
  title?: string;
  desc?: string;
  headerRight?: Element;
  children: Element;
}

export const SectionCard: Component<SectionCardProps> = (props) => {
  return (
    <div class="settings-card">
      {(props.title || props.headerRight) && (
        <div class="card-header">
          {props.title && (
            <div>
              <div class="card-title">{props.title}</div>
              {props.desc && <div class="card-desc">{props.desc}</div>}
            </div>
          )}
          {props.headerRight}
        </div>
      )}
      {props.children}
    </div>
  );
};
