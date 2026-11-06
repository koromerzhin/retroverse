
const variants: Record<string, React.CSSProperties> = {
  success: { background: "var(--success)", color: "#fff" },
  warning: { background: "var(--warning)", color: "#000" },
  danger: { background: "var(--danger)", color: "#fff" },
  muted: { background: "var(--bg-3)", color: "var(--text-muted)" },
  primary: { background: "var(--accent)", color: "#fff" },
};

export default function Badge({ children, variant = "muted" }: { children: React.ReactNode; variant?: keyof typeof variants }) {
  return (
    <span className="badge" style={variants[variant] || variants.muted}>
      {children}
    </span>
  );
}
