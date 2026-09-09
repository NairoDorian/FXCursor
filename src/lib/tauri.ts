/**
 * Shared Tauri v2 runtime detection utility.
 */
export const isTauri: boolean = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
