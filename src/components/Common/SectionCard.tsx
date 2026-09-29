import type { Component, Element } from 'solid-js';
import { Show } from 'solid-js';

interface SectionCardProps {
  title?: string;
  desc?: string;
  headerRight?: Element;
  /**
   * Restores every parameter in this card. When supplied, a "Default" button appears in the
   * card header; it is disabled while the section already matches its defaults, so a button that
   * would do nothing is never clickable.
   */
  onReset?: () => void;
  /** Whether the section currently differs from its defaults. */
  modified?: boolean;
  children: Element;
}

export const SectionCard: Component<SectionCardProps> = (props) => {
  return (
    <div class="settings-card">
      {(props.title || props.headerRight || props.onReset) && (
        <div class="card-header">
          {props.title && (
            <div>
              <div class="card-title">{props.title}</div>
              {props.desc && <div class="card-desc">{props.desc}</div>}
            </div>
          )}
          <div class="card-header-actions">
            <Show when={props.onReset}>
              <button
                type="button"
                class={`reset-section ${props.modified ? 'active' : ''}`}
                disabled={!props.modified}
                title="Restore every parameter in this section to its default"
                onClick={() => props.onReset?.()}
              >
                Default
              </button>
            </Show>
            {props.headerRight}
          </div>
        </div>
      )}
      {props.children}
    </div>
  );
};
