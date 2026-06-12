import type { AppConfig } from "../lib/bindings";

interface Props {
  label: string;
  color: [number, number, number, number];
  onChange: (color: [number, number, number, number]) => void;
}

export default function ColorPicker({ label, color, onChange }: Props) {
  const hexColor = `#${Math.round(color[0] * 255)
    .toString(16)
    .padStart(2, "0")}${Math.round(color[1] * 255)
    .toString(16)
    .padStart(2, "0")}${Math.round(color[2] * 255)
    .toString(16)
    .padStart(2, "0")}`;

  const handleColorChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const hex = e.target.value;
    const r = parseInt(hex.slice(1, 3), 16) / 255;
    const g = parseInt(hex.slice(3, 5), 16) / 255;
    const b = parseInt(hex.slice(5, 7), 16) / 255;
    onChange([r, g, b, color[3]]);
  };

  const handleAlphaChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange([color[0], color[1], color[2], parseFloat(e.target.value)]);
  };

  return (
    <div className="flex items-center gap-2 mb-1">
      <span className="text-[10px] text-gray-500 w-16 shrink-0">{label}</span>
      <input
        type="color"
        value={hexColor}
        onChange={handleColorChange}
        className="h-5 w-8 rounded border-0 bg-transparent cursor-pointer"
      />
      <input
        type="range"
        min={0}
        max={1}
        step={0.01}
        value={color[3]}
        onChange={handleAlphaChange}
        className="flex-1 h-1 bg-gray-700 rounded-full appearance-none cursor-pointer accent-cyan-500"
        title={`Alpha: ${color[3].toFixed(2)}`}
      />
    </div>
  );
}
