import { useState, useEffect, useRef } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import type { LibraryEntry, Platform } from "../types";

export default function Library() {
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { t } = useI18n();
  const [searchParams] = useSearchParams();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const platform = searchParams.get("platform") || undefined;
  const developer = searchParams.get("developer") || undefined;
  const publisher = searchParams.get("publisher") || undefined;
  const genre = searchParams.get("genre") || undefined;
  const [page, setPage] = useState(1);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.library.view") as "cards" | "list") || "cards");
  const [calculatingAll, setCalculatingAll] = useState(false);
  const [onlyMissingHash, setOnlyMissingHash] = useState(false);
  const [onlyNoMeta, setOnlyNoMeta] = useState(false);
  const [onlyRecent, setOnlyRecent] = useState(false);
  const [onlyFavorites, setOnlyFavorites] = useState(false);
  const [showFilters, setShowFilters] = useState(false);
  const [hashingGame, setHashingGame] = useState<string | null>(null);
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
  const { data: presStatuses } = useQuery<any[]>({ queryKey: ["preservation-statuses"], queryFn: () => api.listPreservationStatuses() });
  const { data: developers } = useQuery<string[]>({ queryKey: ["developers"], queryFn: () => api.listDevelopers() });
  const { data: genres } = useQuery<string[]>({ queryKey: ["genres"], queryFn: () => api.listGenres() });
  const { data: favorites } = useQuery<string[]>({ queryKey: ["favorites"], queryFn: () => api.listFavorites() });
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
  const sortedGames = [...games].filter((e) => {
    if (onlyMissingHash && !needsHash(e)) return false;
    if (onlyNoMeta && e.game.metadata_json) return false;
    if (onlyFavorites && !(favorites || []).includes(e.game.id)) return false;
    if (genre) {
      let g: any = null;
      try { g = e.game.metadata_json ? JSON.parse(e.game.metadata_json) : null; } catch {}
      const gen = (g?.genres || []).map((x: any) => (x.noms || []).map((n: any) => n.text).join("/")).join(", ") || (g?._genres || []).join(", ");
      if (!gen.toLowerCase().includes(genre.toLowerCase())) return false;
    }
    return true;
  }).sort((a, b) => {
    let cmp = 0;
    switch (sortKey) {
      case "title": cmp = a.game.title.localeCompare(b.game.title); break;
      case "platform": cmp = (a.platform?.name ?? "").localeCompare(b.platform?.name ?? ""); break;
      case "developer": cmp = (a.developer ?? "").localeCompare(b.developer ?? ""); break;
      case "publisher": cmp = (a.publisher ?? "").localeCompare(b.publisher ?? ""); break;
      case "files": cmp = a.file_count - b.file_count; break;
    }
    if (onlyRecent) {
    const da = new Date(a.game.created_at).getTime();
    const db = new Date(b.game.created_at).getTime();
    return db - da;
  }
  return sortDir === "asc" ? cmp : -cmp;
  });

  const platformCounts: Record<string, number> = platformCountsData || {};
  const availablePlatforms = (platforms || []).filter((p) => (platformCounts[p.id] ?? 0) > 0);

  const needsHash = (e: LibraryEntry) => (e.files || []).some((f: any) => !f.hash || !f.sha512 || ((f.extension === "zip" || f.extension === "7z") && !f.archive_hash)) && !(e.files || []).every((f: any) => f.extension === "scummvm");
  const presStatus = (id: string) => (presStatuses || []).find((p: any) => p[0] === id)?.[1] ?? "unknown";
  const presBadge = (status: string) => status === "verified" ? "✓" : status === "bad_dump" ? "✗" : "·";

  const hashGame = async (gameId: string) => {
    setHashingGame(gameId);
    try {
      const n = await api.computeHashesForGame(gameId);
      qc.invalidateQueries({ queryKey: ["games"] });
      api.showNotification("RETROVERSE", `Empreintes calculées : ${n} fichier(s).`).catch(() => {});
    } catch (e: any) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setHashingGame(null);
    }
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  // Virtual scrolling for the list view.
  const scrollRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const ROW_HEIGHT = 40;
  const VIEWPORT_HEIGHT = 400;
  const visibleCount = Math.ceil(VIEWPORT_HEIGHT / ROW_HEIGHT) + 6;
  const startIndex = Math.floor(scrollTop / ROW_HEIGHT);
  const endIndex = Math.min(sortedGames.length, startIndex + visibleCount);
  const visibleRows = sortedGames.slice(startIndex, endIndex);
  const offsetY = startIndex * ROW_HEIGHT;
  const totalHeight = sortedGames.length * ROW_HEIGHT;

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
      await api.computeHashesForAllGames();
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
        <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={() => setShowFilters((v) => !v)}>
          {showFilters ? "Masquer filtres" : "Filtres avancés"}
        </button>
        <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={() => {
          const blob = new Blob([JSON.stringify(result, null, 2)], { type: "application/json" });
          const url = URL.createObjectURL(blob);
          const a = document.createElement("a");
          a.href = url; a.download = "retroverse-library.json"; a.click();
          URL.revokeObjectURL(url);
        }}>{t("library.exportJson")}</button>
        <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)" }}>
          <input type="checkbox" checked={onlyMissingHash} onChange={(e) => setOnlyMissingHash(e.target.checked)} />
          {t("library.missingHashOnly")}
        </label>
        <button className="btn primary" disabled={calculatingAll} onClick={handleCalculateAll}>
          {calculatingAll ? <><span className="spinner" /> Calcul en cours...</> : "Tout calculer"}
        </button>

        <span className="badge">{formatNumber(games?.length ?? 0)} jeu(x)</span>
        <div className="row" style={{ marginLeft: "auto", gap: 4 }}>
          <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.library.view", "cards"); }} title="Cartes">Cards</button>
          <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.library.view", "list"); }} title="Liste">Liste</button>
        </div>
      </div>

      {showFilters && (
        <div className="card" style={{ marginBottom: 12, padding: 12 }}>
          <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
            <select className="select" style={{ maxWidth: 200 }} value={genre ?? ""} onChange={(e) => {
              const v = e.target.value;
              const params = new URLSearchParams(searchParams);
              if (v) params.set("genre", v); else params.delete("genre");
              navigate(`/library?${params.toString()}`);
            }}>
              <option value="">Tous genres</option>
              {(genres || []).map((g: string) => <option key={g} value={g}>{g}</option>)}
            </select>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)" }}>
              <input type="checkbox" checked={onlyMissingHash} onChange={(e) => setOnlyMissingHash(e.target.checked)} />
              Empreintes manquantes
            </label>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)" }}>
              <input type="checkbox" checked={onlyNoMeta} onChange={(e) => setOnlyNoMeta(e.target.checked)} />
              Sans métadonnées
            </label>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)" }}>
              <input type="checkbox" checked={onlyRecent} onChange={(e) => setOnlyRecent(e.target.checked)} />
              Récemment ajoutés
            </label>
            <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13, color: "var(--text-dim)" }}>
              <input type="checkbox" checked={onlyFavorites} onChange={(e) => setOnlyFavorites(e.target.checked)} />
              Favoris uniquement
            </label>
          </div>
        </div>
      )}

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
          {sortedGames.map((e) => {
            let cover: string | null = null;
            try { cover = e.game.metadata_json ? JSON.parse(e.game.metadata_json).cover_url : null; } catch {}
            return (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">{cover ? <img src={cover} alt="" style={{ width: "100%", height: "100%", objectFit: "cover", borderRadius: 8 }} /> : "🎮"}</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s) · {presBadge(presStatus(e.game.id))}</div>
                {(e.developer || e.publisher) && <div className="meta" style={{ fontSize: 10 }}>{[e.developer, e.publisher].filter(Boolean).join(" · ")}</div>}
                <button className="btn" style={{ fontSize: 11, padding: "2px 6px", marginTop: 4 }} onClick={(ev) => { ev.stopPropagation(); api.toggleFavorite(e.game.id).then(() => qc.invalidateQueries({ queryKey: ["favorites"] })); }}>
                  {(favorites || []).includes(e.game.id) ? "★" : "☆"}
                </button>
                {needsHash(e) && (
                  <button className="btn" style={{ fontSize: 11, padding: "2px 8px", marginTop: 6 }} disabled={hashingGame === e.game.id} onClick={(ev) => { ev.stopPropagation(); hashGame(e.game.id); }}>
                    {hashingGame === e.game.id ? "…" : "Calculer les empreintes"}
                  </button>
                )}
              </div>
            </div>
            );
          })}
        </div>
        ) : (
        <div ref={scrollRef} onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)} style={{ maxHeight: VIEWPORT_HEIGHT, overflow: "auto", border: "1px solid var(--border)", borderRadius: 8 }}>
          <table className="table" style={{ width: "100%" }}>
            <thead style={{ position: "sticky", top: 0, background: "var(--bg-2)", zIndex: 1 }}>
              <tr>
                <th style={{ cursor: "pointer" }} onClick={() => toggleSort("title")}>Titre {sortKey === "title" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
                <th style={{ cursor: "pointer" }} onClick={() => toggleSort("platform")}>Plateforme {sortKey === "platform" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
                <th style={{ cursor: "pointer" }} onClick={() => toggleSort("developer")}>Développeur {sortKey === "developer" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
                <th style={{ cursor: "pointer" }} onClick={() => toggleSort("publisher")}>Éditeur {sortKey === "publisher" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
                <th style={{ cursor: "pointer" }} onClick={() => toggleSort("files")}>Fichiers {sortKey === "files" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>
                <th>Empreintes</th>
              <th>Préserv.</th>
              </tr>
            </thead>
            <tbody>
              <tr style={{ height: offsetY }}><td colSpan={6} style={{ padding: 0, border: "none" }}></td></tr>
              {visibleRows.map((e) => (
                <tr key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)} style={{ cursor: "pointer", height: ROW_HEIGHT }}>
                  <td>{e.game.title}</td>
                  <td>{e.platform?.name ?? "Inconnu"}</td>
                  <td>{e.developer ?? "—"}</td>
                  <td>{e.publisher ?? "—"}</td>
                  <td>{e.file_count}</td>
                  <td onClick={(ev) => ev.stopPropagation()}>
                    {needsHash(e) ? (
                      <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} disabled={hashingGame === e.game.id} onClick={() => hashGame(e.game.id)}>
                        {hashingGame === e.game.id ? "…" : "Calculer"}
                      </button>
                    ) : "✓"}
                  </td>
                </tr>
              ))}
              <tr style={{ height: Math.max(0, totalHeight - offsetY - visibleRows.length * ROW_HEIGHT) }}><td colSpan={6} style={{ padding: 0, border: "none" }}></td></tr>
            </tbody>
          </table>
        </div>
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