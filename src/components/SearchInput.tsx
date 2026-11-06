
export default function SearchInput({ value, onChange, placeholder, maxWidth = 260 }: { value: string; onChange: (v: string) => void; placeholder?: string; maxWidth?: number }) {
  return (
    <input
      className="input"
      style={{ maxWidth }}
      placeholder={placeholder || "Rechercher…"}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  );
}
