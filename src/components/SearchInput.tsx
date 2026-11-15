export default function SearchInput({ value, onChange, placeholder, maxWidth = 260, onBlur }: { value: string; onChange: (v: string) => void; placeholder?: string; maxWidth?: number; onBlur?: () => void }) {
  return (
    <input
      className="input"
      data-search="true"
      style={{ maxWidth }}
      placeholder={placeholder || "Rechercher…"}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onBlur={onBlur}
    />
  );
}
