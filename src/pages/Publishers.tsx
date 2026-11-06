import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";

export default function Publishers() {
  const { data } = useQuery({ queryKey: ["publishers-full"], queryFn: () => api.listPublishersFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [cols, setCols] = useState<Record<string, boolean>>(() => { try { const c = JSON.parse(localStorage.getItem("retroverse.publishers.cols") || "{}"); return Object.keys(c).length ? c : { name: true, games: true }; } catch { return { name: true, games: true }; } });
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.publishers.view") as "cards" | "list") || "list");

  useEffect(() => { localStorage.setItem("retroverse.publishers.cols", JSON.stringify(cols)); }, [cols]);

  const filtered = (data || [])
    .filter((p: any) => !search || p.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => sortDir === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

  return (
    <>
      <h1 className="page-title">Éditeurs</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
          <div className="row" style={{ gap: 4, marginLeft: "auto" }}>
            <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.publishers.view", "cards"); }}>Cards</button>
            <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.publishers.view", "list"); }}>Liste</button>
          </div>
        </div>
        <div className="row" style={{ gap: 8, marginBottom: 8, flexWrap: "wrap", fontSize: 12, color: "var(--text-dim)" }}>
          {Object.keys(cols).map((c) => (
            <label key={c} style={{ display: "flex", alignItems: "center", gap: 4 }}>
              <input type="checkbox" checked={cols[c]} onChange={(e) => setCols({ ...cols, [c]: e.target.checked })} />
              {c}
            </label>
          ))}
        </div>
        {view === "cards" ? (
        <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 12 }}>
          {filtered.map((p: any) => (
            <div className="card" key={p.id} style={{ padding: 12 }}>
              <div style={{ fontWeight: 600, marginBottom: 4 }}>{p.name}</div>
              <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{p.game_count ?? 0} jeu(x)</div>

            </div>
          ))}
        </div>
        ) : (
        <div style={{ maxHeight: 400, overflow: "auto", border: "1px solid var(--border)", borderRadius: 8 }}>
        <table className="table" style={{ width: "100%" }}>
          <thead style={{ position: "sticky", top: 0, background: "var(--bg-2)", zIndex: 1 }}><tr><th>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>Jeux</th><th>ID SS</th></tr></thead>
          <tbody>
            {filtered.map((p: any) => (
              <tr key={p.id}>
                {cols.name && <td>{p.name}</td>}
                {cols.games && <td>{p.game_count ?? 0}</td>}
              </tr>
            ))}
          </tbody>
        </table>
        </div>
        )}
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} éditeur(s)</p>
      </div>
    </>
  );
}
