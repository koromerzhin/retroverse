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
  const [msg, setMsg] = useState<string | null>(null);

  const counts: Record<string, number> = {};
  games?.forEach((g: any) => {
    const pid = g.game.platform_id;
    if (pid) counts[pid] = (counts[pid] || 0) + 1;
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

  return (
    <>
      <h1 className="page-title">Plateformes</h1>
      <p className="page-sub">Parcourez votre collection par plateforme. Cliquez sur une machine pour voir ses jeux.</p>

      {msg && <p style={{ color: "var(--text-dim)", marginBottom: 12, fontSize: 13 }}>{msg}</p>}

      <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
        {platforms?.map((p) => (
          <div className="card" key={p.id} style={{ cursor: "pointer" }}>
            <div style={{ fontSize: 28, marginBottom: 8 }} onClick={() => navigate(`/library?platform=${p.id}`)}>🎮</div>
            <div style={{ fontWeight: 600 }} onClick={() => navigate(`/library?platform=${p.id}`)}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 10, display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
              <span className="badge">{formatNumber(counts[p.id] ?? 0)} jeu(x)</span>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px" }}
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