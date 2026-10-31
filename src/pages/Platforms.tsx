import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import type { Platform, GameSource } from "../types";

export default function Platforms() {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
  const { data: games } = useQuery({
    queryKey: ["games"],
    queryFn: () => api.listGames(),
  });
  const { data: sources } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });

  const [scanning, setScanning] = useState<string | null>(null);
  const [scanningAll, setScanningAll] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<"name" | "count">("name");

  const counts: Record<string, number> = {};
  games?.forEach((g: any) => {
    const pid = g.game.platform_id;
    if (pid) counts[pid] = (counts[pid] || 0) + 1;
  });

  const sortedPlatforms = [...(platforms || [])].sort((a, b) => {
    if (sortBy === "count") {
      return (counts[b.id] ?? 0) - (counts[a.id] ?? 0) || a.name.localeCompare(b.name);
    }
    return a.name.localeCompare(b.name);
  });

  async function handleScanPlatform(p: Platform) {
    if (!sources || sources.length === 0) {
      setMsg("Aucune source configurée.");
      return;
    }
    setScanning(p.id);
    setMsg(null);
    let totalFiles = 0;
    let totalGames = 0;
    for (const s of sources) {
      const folder = s.kind === "remote"
        ? `${s.remote_path || s.path}/${p.id}`.replace(/\/+/g, "/")
        : `${s.path}/${p.id}`;
      try {
        const res = s.kind === "remote"
          ? await api.scanRemoteFolder(s.id, folder)
          : await api.scanLocalFolder(s.id, folder);
        totalFiles += res.files_found;
        totalGames += res.games_created;
      } catch (e: any) {
        // folder may not exist for this source; ignore
      }
    }
    setMsg(`Scan ${p.name} terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`);
    setScanning(null);
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
  }

  async function handleScanAll() {
    if (!sources || sources.length === 0) {
      setMsg("Aucune source configurée.");
      return;
    }
    setScanningAll(true);
    setMsg(null);
    let totalFiles = 0;
    let totalGames = 0;
    for (const s of sources) {
      try {
        const res = s.kind === "remote"
          ? await api.scanRemoteSource(s.id)
          : await api.scanSource(s.id);
        totalFiles += res.files_found;
        totalGames += res.games_created;
      } catch (e: any) {
        // ignore per-source errors
      }
    }
    setMsg(`Scan complet terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`);
    setScanningAll(false);
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
    qc.invalidateQueries({ queryKey: ["platforms"] });
  }

  return (
    <>
      <h1 className="page-title">Plateformes</h1>
      <p className="page-sub">Parcourez votre collection par plateforme. Cliquez sur une machine pour voir ses jeux.</p>

      {msg && <p style={{ color: "var(--text-dim)", marginBottom: 12, fontSize: 13 }}>{msg}</p>}

      <div className="row" style={{ marginBottom: 16, alignItems: "center", flexWrap: "wrap" }}>
        <span style={{ fontSize: 13, color: "var(--text-dim)" }}>Trier par :</span>
        <select className="select" style={{ maxWidth: 180 }} value={sortBy} onChange={(e) => setSortBy(e.target.value as "name" | "count")}>
          <option value="name">Nom</option>
          <option value="count">Nombre de jeux</option>
        </select>
        <button className="btn primary" onClick={handleScanAll} disabled={scanningAll} style={{ marginLeft: "auto" }}>
          {scanningAll ? <><span className="spinner" /> Scan en cours…</> : "Tout scanner"}
        </button>
      </div>

      <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
        {sortedPlatforms.map((p) => (
          <div className="card" key={p.id} style={{ cursor: "pointer", display: "flex", flexDirection: "column", minHeight: 160 }}>
            <div style={{ fontSize: 28, marginBottom: 8 }} onClick={() => navigate(`/library?platform=${p.id}`)}>🎮</div>
            <div style={{ fontWeight: 600 }} onClick={() => navigate(`/library?platform=${p.id}`)}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 8 }}>
              <span className="badge">{formatNumber(counts[p.id] ?? 0)} jeu(x)</span>
            </div>
            <div style={{ marginTop: "auto", paddingTop: 8, display: "flex", gap: 8, flexWrap: "nowrap" }}>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", flex: 1 }}
                onClick={(e) => { e.stopPropagation(); navigate(`/library?platform=${p.id}`); }}
              >
                Voir la bibliothèque
              </button>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", flex: 1, justifyContent: "center", textAlign: "center" }}
                disabled={scanning === p.id}
                onClick={(e) => { e.stopPropagation(); handleScanPlatform(p); }}
              >
                {scanning === p.id ? <><span className="spinner" /> Scan…</> : "Scanner"}
              </button>
            </div>
          </div>
        ))}
      </div>
    </>
  );
}