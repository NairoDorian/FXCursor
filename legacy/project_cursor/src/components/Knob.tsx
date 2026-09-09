import { useRef, useEffect, useCallback } from "react";

export function Slider({ label, value, min, max, step = 0.01, unit = "", onChange }: { label: string; value: number; min: number; max: number; step?: number; unit?: string; onChange: (v: number) => void; }) {
  const ref = useRef<HTMLInputElement>(null);
  const pct = ((value - min) / (max - min)) * 100;
  useEffect(() => { if (ref.current) ref.current.style.setProperty("--pct", `${pct}%`); }, [pct]);
  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => { const v = parseFloat(e.target.value); onChange(Math.round(v / step) * step); if (ref.current) ref.current.style.setProperty("--pct", `${((v - min) / (max - min)) * 100}%`); }, [min, max, step, onChange]);
  const fmt = step >= 1 ? value.toFixed(0) : value.toFixed(step < 0.1 ? 2 : 1);
  return (<div className="mb-2.5"><div className="mb-1 flex items-center justify-between"><span className="text-[11px] font-medium text-gray-400 select-none">{label}</span><span className="min-w-[42px] text-right text-[11px] font-mono font-semibold tabular-nums text-cyan-400">{fmt}<span className="text-[9px] text-gray-500 ml-0.5">{unit}</span></span></div><input ref={ref} type="range" min={min} max={max} step={step} value={value} onChange={handleChange} /></div>);
}

export function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void; }) {
  return (<label className="flex cursor-pointer items-center gap-2.5 select-none"><div className="relative"><div className={`h-5 w-9 rounded-full transition-colors duration-200 ${checked ? "bg-cyan-500 shadow-[0_0_8px_rgba(0,204,255,0.3)]" : "bg-gray-700"}`} /><div className={`absolute left-0.5 top-0.5 h-4 w-4 rounded-full bg-white shadow transition-transform duration-200 ${checked ? "translate-x-4" : "translate-x-0"}`} /></div><span className="text-[12px] font-medium text-gray-300">{label}</span><input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} className="hidden" /></label>);
}

export function ChipButton({ label, active, onClick }: { label: string; active: boolean; onClick: () => void; }) {
  return (<button onClick={onClick} className={`rounded-lg px-3 py-1.5 text-[11px] font-semibold transition-all duration-150 ${active ? "bg-cyan-500/15 text-cyan-300 ring-1 ring-cyan-500/40 shadow-[0_0_12px_rgba(0,204,255,0.1)]" : "bg-white/5 text-gray-500 hover:bg-white/10 hover:text-gray-300"}`}>{label}</button>);
}

export function Section({ icon, title, open, onToggle, children }: { icon: string; title: string; open: boolean; onToggle: () => void; children: React.ReactNode; }) {
  return (<section className="group mb-2 overflow-hidden rounded-2xl border border-white/[0.06] bg-white/[0.02] backdrop-blur transition-all duration-200 hover:border-white/[0.1]"><button onClick={onToggle} className="flex w-full items-center gap-3 px-4 py-3 text-left"><span className="text-base leading-none opacity-60 transition-opacity group-hover:opacity-100">{icon}</span><span className="flex-1 text-[12px] font-semibold uppercase tracking-[0.08em] text-gray-400 transition-colors group-hover:text-gray-200">{title}</span><svg className={`h-4 w-4 text-gray-600 transition-transform duration-200 ${open ? "rotate-180" : ""}`} fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" /></svg></button><div className={`overflow-hidden transition-all duration-300 ease-in-out ${open ? "max-h-[2000px] opacity-100" : "max-h-0 opacity-0"}`}><div className="px-4 pb-4">{children}</div></div></section>);
}
