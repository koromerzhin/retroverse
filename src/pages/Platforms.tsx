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
  const { data: countsData } = useQuery({
    queryKey: ["platform-game-counts"],
    queryFn: () => api.platformGameCounts(),
    staleTime: 60_000,
  });
  const { data: sources } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });

  const [scanning, setScanning] = useState<string | null>(null);
  const [scanningAll, setScanningAll] = useState(false);
  const [hashingPlatform, setHashingPlatform] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<"name" | "count">("name");
  const [hideEmpty, setHideEmpty] = useState(true);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.platforms.view") as "cards" | "list") || "cards");

  async function handleOpenFolder(p: Platform) {
    if (!sources) return;
    for (const s of sources) {
      if (s.kind !== "local") continue;
      const folder = `${s.path}/${p.id}`;
      try {
        const { openPath } = await import("@tauri-apps/plugin-opener");
        await openPath(folder);
        return;
      } catch (e: any) {
        // try next source
      }
    }
    // fallback: try the source path itself
    const local = sources.find((s) => s.kind === "local");
    if (local) {
      try {
        const { openPath } = await import("@tauri-apps/plugin-opener");
        await openPath(local.path);
        return;
      } catch (e: any) {
        alert(`Impossible d'ouvrir : ${local.path} (${e})`);
        return;
      }
    }
    const remote = sources.find((s) => s.kind === "remote");
    if (remote) {
      alert(`Dossier distant : ${remote.remote_path || remote.path}/${p.id}`);
    } else {
      alert("Aucun dossier local trouvé pour cette plateforme.");
    }
  }

  const counts: Record<string, number> = countsData || {};

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
    setMsg(`Scan ${p.name} en cours…`);
    let totalFiles = 0;
    let totalGames = 0;
    let errors: string[] = [];
    for (const s of sources) {
      try {
        const res = s.kind === "remote"
          ? await api.scanRemoteSource(s.id, undefined, p.id)
          : await api.scanSource(s.id, undefined, p.id);
        totalFiles += res.files_found;
        totalGames += res.games_created;
      } catch (e: any) {
        errors.push(`${s.name}: ${e}`);
      }
    }
    if (errors.length > 0) {
      setMsg(`Scan ${p.name} : ${errors.join(" | ")}`);
    } else if (totalFiles === 0) {
      setMsg(`Scan ${p.name} : aucun fichier trouvé.`);
    } else {
      setMsg(`Scan ${p.name} terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`);
    }
    setScanning(null);
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
  }


  async function handleHashPlatform(p: Platform) {
    setHashingPlatform(p.id);
    try {
      const n = await api.computeHashesForPlatform(p.id);
      setMsg(`Empreintes calculées : ${formatNumber(n)} fichier(s) pour ${p.name}.`);
    } catch (e: any) {
      setMsg(`Erreur : ${e}`);
    } finally {
      setHashingPlatform(null);
    }
  }

  async function handleScanAll() {
    if (!platforms || platforms.length === 0) {
      setMsg("Aucune plateforme.");
      return;
    }
    setScanningAll(true);
    for (const p of platforms) {
      if ((counts[p.id] ?? 0) === 0 && hideEmpty) {
        // skip empty platforms if hidden, but still allow scanning
      }
      setMsg(`Scan en cours : ${p.name}…`);
      await handleScanPlatform(p);
    }
    setMsg("Scan complet terminé.");
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
          <div className="card" key={p.id} style={{ cursor: "pointer", display: "flex", flexDirection: "column", height: 260, overflow: "hidden" }}>
            <div style={{ fontSize: 28, marginBottom: 8 }} onClick={() => navigate(`/library?platform=${p.id}`)}>🎮</div>
            <div style={{ fontWeight: 600 }} onClick={() => navigate(`/library?platform=${p.id}`)}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 8 }}>
              <span className="badge">{formatNumber(counts[p.id] ?? 0)} {t("platforms.games")}</span>
            </div>
            <div style={{ marginTop: "auto", paddingTop: 8, display: "flex", flexDirection: "column", gap: 6 }}>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", width: "100%", justifyContent: "center", textAlign: "center" }}
                disabled={(counts[p.id] ?? 0) === 0}
                onClick={(e) => { e.stopPropagation(); navigate(`/library?platform=${p.id}`); }}
              >
                {t("platforms.viewLibrary")}
              </button>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", width: "100%", justifyContent: "center", textAlign: "center" }}
                onClick={(e) => { e.stopPropagation(); handleOpenFolder(p); }}
              >
                Ouvrir le dossier
              </button>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", width: "100%", justifyContent: "center", textAlign: "center" }}
                disabled={scanning === p.id}
                onClick={(e) => { e.stopPropagation(); handleScanPlatform(p); }}
              >
                {scanning === p.id ? <><span className="spinner" /> {t("platforms.scanning")}</> : t("platforms.scan")}
              </button>
              <button
                className="btn"
                style={{ fontSize: 11, padding: "4px 8px", width: "100%", justifyContent: "center", textAlign: "center" }}
                disabled={hashingPlatform === p.id}
                onClick={(e) => { e.stopPropagation(); handleHashPlatform(p); }}
              >
                {hashingPlatform === p.id ? "…" : "Calculer les empreintes"}
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
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={() => handleOpenFolder(p)}>Ouvrir le dossier</button>
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