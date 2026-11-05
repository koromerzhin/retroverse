import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import type { LibraryEntry, Platform } from "../types";

export default function Library() {
  const navigate = useNavigate();
  const { t } = useI18n();
  const qc = useQueryClient();
  const [searchParams] = useSearchParams();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const platform = searchParams.get("platform") || undefined;
  const developer = searchParams.get("developer") || undefined;
  const publisher = searchParams.get("publisher") || undefined;
  const [page, setPage] = useState(1);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.library.view") as "cards" | "list") || "cards");
  const [calculatingAll, setCalculatingAll] = useState(false);
  const [hashingAllGames, setHashingAllGames] = useState(false);
  const [sortKey, setSortKey] = useState<"title" | "platform" | "developer" | "publisher" | "files">("title");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const toggleSort = (k: "title" | "platform" | "developer" | "publisher" | "files") => {
    if (sortKey === k) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else { setSortKey(k); setSortDir("asc"); }
  };
  const pageSize = 24;

  useEffect(() => {
    const t = setTimeout(() => setSearch(searchInput), 300);
    return () => clearTimeout(t);
  }, [searchInput]);

  useEffect(() => { setPage(1); }, [search, platform, developer, publisher]);

  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
  const { data: sources } = useQuery<any[]>({ queryKey: ["sources"], queryFn: () => api.listSources() });
  const { data: developers } = useQuery<string[]>({ queryKey: ["developers"], queryFn: () => api.listDevelopers() });
  const { data: publishers } = useQuery<string[]>({ queryKey: ["publishers"], queryFn: () => api.listPublishers() });
  const { data: platformCountsData } = useQuery<any>({
    queryKey: ["platform-game-counts"],
    queryFn: () => api.platformGameCounts(),
    staleTime: 60_000,
  });
  const { data: result, isLoading, refetch } = useQuery<{ items: LibraryEntry[]; total: number }>({
    queryKey: ["games", platform, search, page],
    queryFn: async () => {
      const r = search.trim()
        ? await api.searchGamesFts(search.trim(), platform, pageSize, (page - 1) * pageSize)
        : await api.listGames(platform, undefined, developer, publisher, pageSize, (page - 1) * pageSize);
      return r;
    },
  });

  const games = result?.items ?? [];
  const total = result?.total ?? 0;
  const sortedGames = [...games].sort((a, b) => {
    let cmp = 0;
    switch (sortKey) {
      case "title": cmp = a.game.title.localeCompare(b.game.title); break;
      case "platform": cmp = (a.platform?.name ?? "").localeCompare(b.platform?.name ?? ""); break;
      case "developer": cmp = (a.developer ?? "").localeCompare(b.developer ?? ""); break;
      case "publisher": cmp = (a.publisher ?? "").localeCompare(b.publisher ?? ""); break;
      case "files": cmp = a.file_count - b.file_count; break;
    }
    return sortDir === "asc" ? cmp : -cmp;
  });

  const platformCounts: Record<string, number> = platformCountsData || {};
  const availablePlatforms = (platforms || []).filter((p) => (platformCounts[p.id] ?? 0) > 0);

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  async function handleCalculateAll() {
    if (calculatingAll) return;
    setCalculatingAll(true);
    let errors: string[] = [];
    const plats = platforms || [];
    const srcs = sources || [];
    if (plats.length > 0 && srcs.length > 0) {
      for (const p of plats) {
        for (const s of srcs) {
          try {
            if (s.kind === "remote") await api.scanRemoteSource(s.id, undefined, p.id);
            else await api.scanSource(s.id, undefined, p.id);
          } catch (e: any) {
            errors.push(`${s.name}/${p.name}: ${e}`);
          }
        }
      }
    }
    try {
      await api.computeHashes();
    } catch (e: any) {
      errors.push(String(e));
    }
    qc.invalidateQueries();
    if (errors.length > 0) {
      api.showNotification("RETROVERSE", `Calcul terminé avec erreurs : ${errors.join(" | ")}`).catch(() => {});
    } else {
      api.showNotification("RETROVERSE", "Calcul terminé.").catch(() => {});
    }
    setCalculatingAll(false);
  }

  async function handleHashAllGames() {
    if (hashingAllGames) return;
    setHashingAllGames(true);
    try {
      const n = await api.computeHashesForAllGames();
      qc.invalidateQueries();
      api.showNotification("RETROVERSE", `Empreintes calculées : ${n} fichier(s).`).catch(() => {});
    } catch (e: any) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setHashingAllGames(false);
    }
  }

  return (
    <>
      <h1 className="page-title">{t("library.title")}</h1>
      <p className="page-sub">{t("library.subtitle")}</p>

      <div className="row" style={{ marginBottom: 18 }}>
        <input
          className="input"
          style={{ maxWidth: 320 }}
          placeholder={t("library.search")}
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
        />
        <select
          className="select"
          style={{ maxWidth: 260 }}
          value={platform ?? ""}
          onChange={(e) => {
            const v = e.target.value || undefined;
            if (v) navigate(`/library?platform=${v}`);
            else navigate("/library");
          }}
        >
          <option value="">Toutes les plateformes</option>
          {availablePlatforms.map((p) => (
            <option key={p.id} value={p.id}>{p.name} ({platformCounts[p.id]})</option>
          ))}
        </select>
        <select
          className="select"
          style={{ maxWidth: 200 }}
          value={developer ?? ""}
          onChange={(e) => {
            const v = e.target.value;
            const params = new URLSearchParams(searchParams);
            if (v) params.set("developer", v); else params.delete("developer");
            navigate(`/library?${params.toString()}`);
          }}
        >
          <option value="">Tous développeurs</option>
          {(developers || []).map((d) => <option key={d} value={d}>{d}</option>)}
        </select>
        <select
          className="select"
          style={{ maxWidth: 200 }}
          value={publisher ?? ""}
          onChange={(e) => {
            const v = e.target.value;
            const params = new URLSearchParams(searchParams);
            if (v) params.set("publisher", v); else params.delete("publisher");
            navigate(`/library?${params.toString()}`);
          }}
        >
          <option value="">Tous éditeurs</option>
          {(publishers || []).map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
        <button className="btn" onClick={() => refetch()}>{t("common.refresh")}</button>
        <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={calculatingAll} onClick={handleCalculateAll}>
          {calculatingAll ? <><span className="spinner" /> Calcul en cours...</> : "Tout calculer"}
        </button>
        <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={hashingAllGames} onClick={handleHashAllGames}>
          {hashingAllGames ? <><span className="spinner" /> Calcul des empreintes en cours...</> : "Calculer les empreintes par jeu vidéo"}
        </button>
        <span className="badge">{formatNumber(games?.length ?? 0)} jeu(x)</span>
        <div className="row" style={{ marginLeft: "auto", gap: 4 }}>
          <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.library.view", "cards"); }} title="Cartes">Cards</button>
          <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.library.view", "list"); }} title="Liste">Liste</button>
        </div>
      </div>

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : total === 0 ? (
        <div className="empty">
          <div className="icon">📚</div>
          <p>{t("library.empty")}</p>
        </div>
      ) : (
        <>
        {view === "cards" ? (
        <div className="grid games">
          {sortedGames.map((e) => (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">🎮</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s)</div>
                {(e.developer || e.publisher) && <div className="meta" style={{ fontSize: 10 }}>{[e.developer, e.publisher].filter(Boolean).join(" · ")}</div>}
              </div>
            </div>
          ))}
        </div>
        ) : (
        <table className="table">
          <thead>
            <tr>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("title")}>Titre {sortKey === "title" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("platform")}>Plateforme {sortKey === "platform" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("developer")}>Développeur {sortKey === "developer" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("publisher")}>Éditeur {sortKey === "publisher" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
              <th style={{ cursor: "pointer" }} onClick={() => toggleSort("files")}>Fichiers {sortKey === "files" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
            </tr>
          </thead>
          <tbody>
            {sortedGames.map((e) => (
              <tr key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)} style={{ cursor: "pointer" }}>
                <td>{e.game.title}</td>
                <td>{e.platform?.name ?? "Inconnu"}</td>
                <td>{e.developer ?? "—"}</td>
                <td>{e.publisher ?? "—"}</td>
                <td>{e.file_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
        )}
        {totalPages > 1 && (
          <div className="row" style={{ marginTop: 20, justifyContent: "center", gap: 8 }}>
            <button className="btn" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>←</button>
            <span className="badge">{page} / {totalPages}</span>
            <button className="btn" disabled={page >= totalPages} onClick={() => setPage((p) => Math.min(totalPages, p + 1))}>→</button>
          </div>
        )}
        </>
      )}
    </>
  );
}