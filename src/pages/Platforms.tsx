import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";
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
  const { data: fileCountsData } = useQuery({
    queryKey: ["platform-file-counts"],
    queryFn: () => api.platformFileCounts(),
    staleTime: 60_000,
  });
  const { data: unhashedCounts } = useQuery<Record<string, number>>({
    queryKey: ["unhashed-counts"],
    queryFn: () => api.unhashedCountsByPlatform(),
    staleTime: 0,
    refetchOnMount: "always",
  });
  const { data: sources } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });

  const [scanning, setScanning] = useState<string | null>(null);
  const [scanningAll, setScanningAll] = useState(false);
  const [hashingPlatform, setHashingPlatform] = useState<string | null>(null);
  const [calculatingAll, setCalculatingAll] = useState(false);
  const [sortKey, setSortKey] = useState<"name" | "manufacturer" | "year" | "count" | "files">("name");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const toggleSort = (key: "name" | "manufacturer" | "year" | "count" | "files") => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else { setSortKey(key); setSortDir("asc"); }
  };
  const [hideEmpty, setHideEmpty] = useState(true);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.platforms.view") as "cards" | "list") || "cards");
  const [search, setSearch] = useState("");

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
  const fileCounts: Record<string, number> = fileCountsData || {};

  const q = search.trim().toLowerCase();
  const sortedPlatforms = [...(platforms || [])]
    .filter((p) => !hideEmpty || (counts[p.id] ?? 0) > 0)
    .filter((p) => !q || p.name.toLowerCase().includes(q) || (p.manufacturer ?? "").toLowerCase().includes(q) || p.slug.toLowerCase().includes(q) || p.id.toLowerCase().includes(q))
    .sort((a, b) => {
      let cmp = 0;
      switch (sortKey) {
        case "name": cmp = a.name.localeCompare(b.name); break;
        case "manufacturer": cmp = (a.manufacturer ?? "").localeCompare(b.manufacturer ?? ""); break;
        case "year": cmp = (a.release_year ?? 0) - (b.release_year ?? 0); break;
        case "count": cmp = (counts[a.id] ?? 0) - (counts[b.id] ?? 0); break;
        case "files": cmp = (fileCounts[a.id] ?? 0) - (fileCounts[b.id] ?? 0); break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });

  async function handleScanPlatform(p: Platform) {
    if (!sources || sources.length === 0) {
      api.showNotification("RETROVERSE", "Aucune source configurée.").catch(() => {});
      return;
    }
    setScanning(p.id);
    (window as any).__retroverseScanningPlatform = true;
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
      api.showNotification("RETROVERSE", `Scan ${p.name} : ${errors.join(" | ")}`).catch(() => {});
    } else if (totalFiles === 0) {
      api.showNotification("RETROVERSE", `Scan ${p.name} : aucun fichier trouvé.`).catch(() => {});
    } else {
      api.showNotification("RETROVERSE", `Scan ${p.name} terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`).catch(() => {});
    }
    setScanning(null);
    (window as any).__retroverseScanningPlatform = false;
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
  }


  async function handleHashPlatform(p: Platform) {
    setHashingPlatform(p.id);
    try {
      const n = await api.computeHashesForPlatform(p.id);
      api.showNotification("RETROVERSE", `Empreintes calculées : ${formatNumber(n)} fichier(s) pour ${p.name}.`).catch(() => {});
      qc.invalidateQueries({ queryKey: ["unhashed-counts"] });
    } catch (e: any) {
      api.showNotification("RETROVERSE", `Erreur : ${e}`).catch(() => {});
    } finally {
      setHashingPlatform(null);
    }
  }

  async function handleScanAll() {
    if (!platforms || platforms.length === 0) {
      api.showNotification("RETROVERSE", "Aucune plateforme.").catch(() => {});
      return;
    }
    setScanningAll(true);
    (window as any).__retroverseScanAll = true;
    let totalFiles = 0;
    let totalGames = 0;
    let errors: string[] = [];
    for (const p of platforms) {
      if (!sources || sources.length === 0) {
        api.showNotification("RETROVERSE", "Aucune source configurée.").catch(() => {});
        continue;
      }
      for (const s of sources) {
        try {
          const res = s.kind === "remote"
            ? await api.scanRemoteSource(s.id, undefined, p.id)
            : await api.scanSource(s.id, undefined, p.id);
          totalFiles += res.files_found ?? 0;
          totalGames += res.games_created ?? 0;
        } catch (e: any) {
          errors.push(`${s.name}/${p.name}: ${e}`);
        }
      }
    }
    if (errors.length > 0) {
      api.showNotification("RETROVERSE", `Scan complet terminé avec erreurs : ${errors.join(" | ")}`).catch(() => {});
    } else if (totalFiles === 0) {
      api.showNotification("RETROVERSE", "Scan complet terminé : aucun fichier trouvé.").catch(() => {});
    } else {
      api.showNotification("RETROVERSE", `Scan complet terminé : ${formatNumber(totalFiles)} fichier(s), ${formatNumber(totalGames)} nouveau(x) jeu(x).`).catch(() => {});
    }
    setScanningAll(false);
    (window as any).__retroverseScanAll = false;
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
    qc.invalidateQueries({ queryKey: ["platforms"] });
  }

  async function handleCalculateAll() {
    if (calculatingAll) return;
    setCalculatingAll(true);
    let errors: string[] = [];
    if (platforms && sources && sources.length > 0) {
      for (const p of platforms) {
        for (const s of sources) {
          try {
            if (s.kind === "remote") await api.scanRemoteSource(s.id, undefined, p.id);
            else await api.scanSource(s.id, undefined, p.id);
          } catch (e: any) {
            errors.push(`${s.name}/${p.name}: ${e}`);
          }
        }
      }
    }
    if (platforms) {
      for (const p of platforms) {
        try {
          await api.computeHashesForPlatform(p.id);
        } catch (e: any) {
          errors.push(`${p.name}: ${e}`);
        }
      }
    }
    qc.invalidateQueries({ queryKey: ["games"] });
    qc.invalidateQueries({ queryKey: ["stats"] });
    qc.invalidateQueries({ queryKey: ["platforms"] });
    qc.invalidateQueries({ queryKey: ["unhashed-counts"] });
    qc.invalidateQueries({ queryKey: ["platform-game-counts"] });
    qc.invalidateQueries({ queryKey: ["platform-file-counts"] });
    if (errors.length > 0) {
      api.showNotification("RETROVERSE", `Calcul terminé avec erreurs : ${errors.join(" | ")}`).catch(() => {});
    } else {
      api.showNotification("RETROVERSE", "Calcul terminé.").catch(() => {});
    }
    setCalculatingAll(false);
  }

  return (
    <>
      <h1 className="page-title">{t("platforms.title")}</h1>
      <p className="page-sub">{t("platforms.subtitle")}</p>

      <div className="row" style={{ marginBottom: 16, alignItems: "center", flexWrap: "wrap" }}>
        <SearchInput value={search} onChange={setSearch} placeholder="Rechercher une plateforme…" maxWidth={260} />
        <span style={{ fontSize: 13, color: "var(--text-dim)" }}>{t("platforms.sortBy")}</span>
        <select className="select" style={{ maxWidth: 180 }} value={sortKey} onChange={(e) => setSortKey(e.target.value as "name" | "manufacturer" | "year" | "count")}>
          <option value="name">{t("platforms.sort.name")}</option>
          <option value="manufacturer">Fabricant</option>
          <option value="year">Année</option>
          <option value="count">{t("platforms.sort.count")}</option>
          <option value="files">Fichiers</option>
        </select>
        <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))} title="Ordre">{sortDir === "asc" ? "▲" : "▼"}</button>
        <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)", marginLeft: 12 }}>
          <input type="checkbox" checked={hideEmpty} onChange={(e) => setHideEmpty(e.target.checked)} />
          Masquer les plateformes sans jeux
        </label>
        <button className="btn primary" onClick={handleScanAll} disabled={scanningAll} style={{ marginLeft: "auto" }}>
          {scanningAll ? <><span className="spinner" /> {t("platforms.scanning")}</> : t("platforms.scanAll")}
        </button>
        <button className="btn primary" onClick={handleCalculateAll} disabled={calculatingAll}>
          {calculatingAll ? <><span className="spinner" /> Calcul en cours...</> : "Tout calculer"}
        </button>
        <div className="row" style={{ gap: 4 }}>
          <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.platforms.view", "cards"); }} title="Cartes">Cards</button>
          <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.platforms.view", "list"); }} title="Liste">Liste</button>
        </div>
      </div>

      {view === "cards" ? (
      <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", alignItems: "stretch" }}>
        {sortedPlatforms.map((p) => (
          <div className="card" key={p.id} style={{ cursor: "pointer", display: "flex", flexDirection: "column", height: 300, overflow: "hidden" }}>
            <div style={{ fontSize: 28, marginBottom: 8 }} onClick={() => navigate(`/library?platform=${p.id}`)}>🎮</div>
            <div style={{ fontWeight: 600 }} onClick={() => navigate(`/library?platform=${p.id}`)}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 8, display: "flex", gap: 6, flexWrap: "wrap" }}>
              <span className="badge">{formatNumber(counts[p.id] ?? 0)} {t("platforms.games")}</span>
              <span className="badge">{formatNumber(fileCounts[p.id] ?? 0)} fichiers</span>
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
                disabled={hashingPlatform === p.id || p.id === "scummvm" || (unhashedCounts?.[p.id] ?? 0) === 0}
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
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("name")}>Nom {sortKey === "name" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("manufacturer")}>Fabricant {sortKey === "manufacturer" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("year")}>Année {sortKey === "year" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("count")}>Jeux {sortKey === "count" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("files")}>Fichiers {sortKey === "files" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
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
              <td>{formatNumber(fileCounts[p.id] ?? 0)}</td>
              <td>
                <div className="row" style={{ gap: 6 }}>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={(counts[p.id] ?? 0) === 0} onClick={() => navigate(`/library?platform=${p.id}`)}>{t("platforms.viewLibrary")}</button>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={() => handleOpenFolder(p)}>Ouvrir le dossier</button>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={scanning === p.id} onClick={() => handleScanPlatform(p)}>{scanning === p.id ? t("platforms.scanning") : t("platforms.scan")}</button>
                  <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={hashingPlatform === p.id || p.id === "scummvm" || (unhashedCounts?.[p.id] ?? 0) === 0} onClick={() => handleHashPlatform(p)}>{hashingPlatform === p.id ? "…" : "Calculer les empreintes"}</button>
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