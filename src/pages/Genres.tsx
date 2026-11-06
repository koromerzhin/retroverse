import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";
import { useI18n } from "../i18n";

export default function Genres() {
  const { lang } = useI18n();
  const qc = useQueryClient();
  const { data } = useQuery({ queryKey: ["genres-full"], queryFn: () => api.listGenresFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [syncing, setSyncing] = useState(false);

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
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
          <button className="btn primary" disabled={syncing} onClick={async () => {
            setSyncing(true);
            try {
              const n = await api.syncGenres();
              qc.invalidateQueries({ queryKey: ["genres-full"] });
              api.showNotification("RETROVERSE", `Genres synchronisés : ${n}`).catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            } finally {
              setSyncing(false);
            }
          }}>
            {syncing ? "…" : "Synchroniser les genres (ScreenScraper)"}
          </button>
        </div>
        <table className="table">
          <thead><tr><th style={{ cursor: "pointer" }} onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>Parent</th></tr></thead>
          <tbody>
            {filtered.map((g: any) => {
              const isChild = !!g.parent_id;
              return (
              <tr key={g.id}>
                <td style={{ paddingLeft: isChild ? 24 : 8 }}>{isChild ? "└ " : ""}{nameOf(g)}</td>
                <td>{g.parent_id ? (data || []).find((x: any) => x.id === g.parent_id)?.name_en || g.parent_id : "—"}</td>
              </tr>
              );
            })}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} genre(s)</p>
      </div>
    </>
  );
}
