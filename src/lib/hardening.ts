/**
 * Release-build webview hardening, installed on every document load (it used to be a one-shot
 * `eval` from Rust, which a reload silently dropped):
 * - files dragged onto the window must not navigate the webview away from the Studio;
 * - the browser context menu is suppressed except on editable fields (copy / paste).
 */
export function installHardening(): void {
  const block = (e: Event) => e.preventDefault();
  window.addEventListener('dragover', block);
  window.addEventListener('drop', block);
  window.addEventListener('contextmenu', (e) => {
    const target = e.target as HTMLElement | null;
    const editable =
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target?.isContentEditable === true;
    if (!editable) e.preventDefault();
  });
}
