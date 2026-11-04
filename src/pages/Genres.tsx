import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n } from "../i18n";

export default function Genres() {
  const { lang } = useI18n();
  const { data } = useQuery({ queryKey: ["genres-full"], queryFn: () => api.listGenresFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const nameOf = (g: any) => {
    const map: Record<string, string | null> = { fr: g.name_fr, en: g.name_en, de: g.name_de, es: g.name_es, it: g.name_it, pt: g.name_pt };
    return map[lang] || g.name_en || g.name_fr || g.name_de || g.name_es || g.name_it || g.name_pt || "—";
  };

  const filtered = (data || [])
    .filter((g: any) => !search || nameOf(g).toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => {
      const an = nameOf(a), bn = nameOf(b);
      return sortDir === "asc" ? an.localeCompare(bn) : bn.localeCompare(an);
    });

  return (
    <>
      <h1 className="page-title">Genres</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12 }}>
          <input className="input" placeholder="Rechercher…" value={search} onChange={(e) => setSearch(e.target.value)} style={{ maxWidth: 240 }} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
        </div>
        <table className="table">
          <thead><tr><th>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>Parent</th></tr></thead>
          <tbody>
            {filtered.map((g: any) => (
              <tr key={g.id}>
                <td>{nameOf(g)}</td>
                <td>{g.parent_id ? (data || []).find((x: any) => x.id === g.parent_id)?.name_en || g.parent_id : "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} genre(s)</p>
      </div>
    </>
  );
}
