
export default function Toolbar({ children }: { children: React.ReactNode }) {
  return (
    <div className="row" style={{ marginBottom: 16, alignItems: "center", flexWrap: "wrap", gap: 8 }}>
      {children}
    </div>
  );
}
