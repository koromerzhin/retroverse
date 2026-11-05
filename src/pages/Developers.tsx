import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";

export default function Developers() {
  const qc = useQueryClient();
  const { data } = useQuery({ queryKey: ["developers-full"], queryFn: () => api.listDevelopersFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [syncingPub, setSyncingPub] = useState(false);
  const [syncingDev, setSyncingDev] = useState(false);

  const syncPub = async () => {
    setSyncingPub(true);
    try {
      const r: any = await api.syncPublishers();
      qc.invalidateQueries({ queryKey: ["publishers-full"] });
      api.showNotification("RETROVERSE", `Synchronisation des éditeurs terminée\n${r.already} déjà présents.\n${r.added} nouveaux ajoutés.\n0 doublon créé.\n${r.games_analyzed} jeux analysés.`).catch(() => {});
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setSyncingPub(false);
    }
  };
  const syncDev = async () => {
    setSyncingDev(true);
    try {
      const r: any = await api.syncDevelopers();
      qc.invalidateQueries({ queryKey: ["developers-full"] });
      api.showNotification("RETROVERSE", `Synchronisation des développeurs terminée\n${r.already} déjà présents.\n${r.added} nouveaux ajoutés.\n0 doublon créé.\n${r.games_analyzed} jeux analysés.`).catch(() => {});
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setSyncingDev(false);
    }
  };

  const filtered = (data || [])
    .filter((d: any) => !search || d.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => sortDir === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

  return (
    <>
      <h1 className="page-title">Développeurs</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <input className="input" placeholder="Rechercher…" value={search} onChange={(e) => setSearch(e.target.value)} style={{ maxWidth: 240 }} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={syncingPub} onClick={syncPub}>
            {syncingPub ? <><span className="spinner" /> Synchronisation des éditeurs...</> : "Synchroniser les éditeurs"}
          </button>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={syncingDev} onClick={syncDev}>
            {syncingDev ? <><span className="spinner" /> Synchronisation des développeurs...</> : "Synchroniser les développeurs"}
          </button>
        </div>
        <table className="table">
          <thead><tr><th style={{ cursor: "pointer" }} onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>ID SS</th></tr></thead>
          <tbody>
            {filtered.map((d: any) => (
              <tr key={d.id}>
                <td>{d.name}</td>
                <td>{d.ss_id ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} développeur(s)</p>
      </div>
    </>
  );
}
