interface PercentFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
}

export function isValidPercent(value: string) {
  return /^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= 100;
}

export function PercentField({ label, value, onChange }: PercentFieldProps) {
  const valid = isValidPercent(value);
  return (
    <div className="resize-fields">
      <label>
        {label} (%)
        <input
          inputMode="numeric"
          value={value}
          onChange={(event) => onChange(event.target.value)}
          aria-invalid={!valid || undefined}
        />
      </label>
      <label className="percent-slider">
        Ajustar {label.toLowerCase()}
        <input
          type="range"
          min={1}
          max={100}
          value={valid ? Number(value) : 1}
          onChange={(event) => onChange(event.target.value)}
        />
      </label>
    </div>
  );
}
