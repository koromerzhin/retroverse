import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import type { Platform, GameSource } from "../types";

export default function Platforms() {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { t } = useI18n();
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
  const [hideEmpty, setHideEmpty] = useState(true);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.platforms.view") as "cards" | "list") || "cards");

  const counts: Record<string, number> = {};
  games?.forEach((g: any) => {
    const pid = g.game.platform_id;
    if (pid) counts[pid] = (counts[pid] || 0) + 1;
  });

  const sortedPlatforms = [...(platforms || [])]
    .filter((p) => !hideEmpty || (counts[p.id] ?? 0) > 0)
    .sort((a, b) => {
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
    let errors: string[] = [];
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
        errors.push(String(e));
      }
    }
    if (errors.length > 0 && totalFiles === 0) {
      setMsg(`Scan ${p.name} : dossier introuvable ou vide.`);
    } else {
      setMsg(`Scan ${p.name} terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`);
    }
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
    setMsg("Scan en cours…");
    let totalFiles = 0;
    let totalGames = 0;
    let errors: string[] = [];
    for (const s of sources) {
      setMsg(`Scan en cours : ${s.name}…`);
      try {
        const res = s.kind === "remote"
          ? await api.scanRemoteSource(s.id)
          : await api.scanSource(s.id);
        totalFiles += res.files_found;
        totalGames += res.games_created;
      } catch (e: any) {
        errors.push(`${s.name}: ${e}`);
      }
    }
    if (errors.length > 0) {
      setMsg(`Scan terminé avec erreurs : ${errors.join("; ")}`);
    } else {
      setMsg(`Scan complet terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`);
    }
    setScanningAll(false);
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
    qc.invalidateQueries({ queryKey: ["platforms"] });
  }

  return (
    <>
      <h1 className="page-title">{t("platforms.title")}</h1>
      <p className="page-sub">{t("platforms.subtitle")}</p>

      {msg && <p style={{ color: "var(--text-dim)", marginBottom: 12, fontSize: 13 }}>{msg}</p>}

      <div className="row" style={{ marginBottom: 16, alignItems: "center", flexWrap: "wrap" }}>
        <span style={{ fontSize: 13, color: "var(--text-dim)" }}>{t("platforms.sortBy")}</span>
        <select className="select" style={{ maxWidth: 180 }} value={sortBy} onChange={(e) => setSortBy(e.target.value as "name" | "count")}>
          <option value="name">{t("platforms.sort.name")}</option>
          <option value="count">{t("platforms.sort.count")}</option>
        </select>
        <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)", marginLeft: 12 }}>
          <input type="checkbox" checked={hideEmpty} onChange={(e) => setHideEmpty(e.target.checked)} />
          Masquer les plateformes sans jeux
        </label>
        <button className="btn primary" onClick={handleScanAll} disabled={scanningAll} style={{ marginLeft: "auto" }}>
          {scanningAll ? <><span className="spinner" /> {t("platforms.scanning")}</> : t("platforms.scanAll")}
        </button>
        <div className="row" style={{ gap: 4 }}>
          <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.platforms.view", "cards"); }} title="Cartes">Cards</button>
          <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.platforms.view", "list"); }} title="Liste">Liste</button>
        </div>
      </div>

      {view === "cards" ? (
      <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", alignItems: "stretch" }}>
        {sortedPlatforms.map((p) => (
          <div className="card" key={p.id} style={{ cursor: "pointer", display: "flex", flexDirection: "column", height: 200, overflow: "hidden" }}>
            <div style={{ fontSize: 28, marginBottom: 8 }} onClick={() => navigate(`/library?platform=${p.id}`)}>🎮</div>
            <div style={{ fontWeight: 600 }} onClick={() => navigate(`/library?platform=${p.id}`)}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 8 }}>
              <span className="badge">{formatNumber(counts[p.id] ?? 0)} {t("platforms.games")}</span>
            </div>
            <div style={{ marginTop: "auto", paddingTop: 8, display: "flex", gap: 8, flexWrap: "nowrap" }}>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", flex: 1 }}
                disabled={(counts[p.id] ?? 0) === 0}
                onClick={(e) => { e.stopPropagation(); navigate(`/library?platform=${p.id}`); }}
              >
                {t("platforms.viewLibrary")}
              </button>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", flex: 1, justifyContent: "center", textAlign: "center" }}
                disabled={scanning === p.id}
                onClick={(e) => { e.stopPropagation(); handleScanPlatform(p); }}
              >
                {scanning === p.id ? <><span className="spinner" /> {t("platforms.scanning")}</> : t("platforms.scan")}
              </button>
            </div>
          </div>
        ))}
      </div>
      ) : (
      <table className="table">
        <thead>
          <tr>
            <th>Nom</th>
            <th>Fabricant</th>
            <th>Année</th>
            <th>Jeux</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {sortedPlatforms.map((p) => (
            <tr key={p.id}>
              <td style={{ fontWeight: 600 }}>{p.name}</td>
              <td>{p.manufacturer ?? "—"}</td>
              <td>{p.release_year ?? "—"}</td>
              <td>{formatNumber(counts[p.id] ?? 0)}</td>
              <td>
                <div className="row" style={{ gap: 6 }}>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={(counts[p.id] ?? 0) === 0} onClick={() => navigate(`/library?platform=${p.id}`)}>{t("platforms.viewLibrary")}</button>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={scanning === p.id} onClick={() => handleScanPlatform(p)}>{scanning === p.id ? t("platforms.scanning") : t("platforms.scan")}</button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      )}
    </>
  );
}