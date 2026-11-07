import { useState, useEffect, useRef } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput, EmptyState } from "../components";
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
  const [view, setView] = useState<"cards" | "list" | "compact">(() => (localStorage.getItem("retroverse.library.view") as "cards" | "list" | "compact") || "list");
  const [calculatingAll, setCalculatingAll] = useState(false);
  const [enrichingAll, setEnrichingAll] = useState(false);
  const [enrichProgress, setEnrichProgress] = useState<any>(null);
  const [onlyMissingHash, setOnlyMissingHash] = useState(false);
  const [onlyNoMeta, setOnlyNoMeta] = useState(false);
  const [onlyRecent, setOnlyRecent] = useState(false);
  const [cols, setCols] = useState<Record<string, boolean>>(() => {
    try { const c = JSON.parse(localStorage.getItem("retroverse.library.cols") || "{}"); return Object.keys(c).length ? c : { title: true, platform: true, developer: true, publisher: true, files: true, hashes: true, pres: true }; } catch { return { title: true, platform: true, developer: true, publisher: true, files: true, hashes: true, pres: true }; }
  });
  const [group, setGroup] = useState<string>("none");

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [batching, setBatching] = useState(false);
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
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen<any>("enrich-progress", (ev) => setEnrichProgress(ev.payload)).then((u) => { unlisten = u; });
    });
    return () => { if (unlisten) unlisten(); };
  }, []);

  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
  const { data: activeSource } = useQuery<any>({ queryKey: ["active-source"], queryFn: () => api.getActiveSource() });
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

  const isEligible = (e: LibraryEntry) => (e.files || []).length > 0 && !(e.files || []).some((f: any) => !f.hash || !f.sha512 || ((f.extension === "zip" || f.extension === "7z") && !f.archive_hash));
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

  useEffect(() => { localStorage.setItem("retroverse.library.cols", JSON.stringify(cols)); }, [cols]);
  const toggleSelect = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  };
  const selectAll = () => setSelected(new Set(sortedGames.map((e) => e.game.id)));
  const clearSelection = () => setSelected(new Set());

  const batchHash = async () => {
    if (selected.size === 0) return;
    setBatching(true);
    let done = 0;
    for (const id of selected) {
      try { await api.computeHashesForGame(id); done++; } catch {}
    }
    qc.invalidateQueries({ queryKey: ["games"] });
    api.showNotification("RETROVERSE", `Empreintes calculées pour ${done} jeu(x).`).catch(() => {});
    setBatching(false);
    clearSelection();
  };

  const batchEnrich = async () => {
    if (selected.size === 0) return;
    setBatching(true);
    let done = 0;
    for (const id of selected) {
      try { await api.enrichGame(id, "screenscraper"); done++; } catch {}
    }
    qc.invalidateQueries({ queryKey: ["games"] });
    api.showNotification("RETROVERSE", `Métadonnées récupérées pour ${done} jeu(x).`).catch(() => {});
    setBatching(false);
    clearSelection();
  };

  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const grouped: [string, LibraryEntry[]][] = (() => {
    if (group === "none") return [["", sortedGames]];
    const map = new Map<string, LibraryEntry[]>();
    for (const e of sortedGames) {
      const key = group === "platform" ? (e.platform?.name ?? "Inconnu") : (e.developer ?? "—");
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(e);
    }
    return Array.from(map.entries());
  })();

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
    if (!activeSource) {
      api.showNotification("RETROVERSE", "Aucune source active. Configurez une source dans la vue Sources.").catch(() => {});
      setCalculatingAll(false);
      return;
    }
    if (plats.length > 0) {
      for (const p of plats) {
        const s = activeSource;
        try {
          if (s.kind === "remote") await api.scanRemoteSource(s.id, undefined, p.id);
          else await api.scanSource(s.id, undefined, p.id);
        } catch (e: any) {
          errors.push(`${s.name}/${p.name}: ${e}`);
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
        <SearchInput value={searchInput} onChange={setSearchInput} placeholder={t("library.search")} maxWidth={320} />
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
        <button className="btn primary" disabled={enrichingAll} onClick={async () => {
          setEnrichingAll(true);
          try { await api.enrichAllEligible(); } catch (e: any) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
          finally { setEnrichingAll(false); setEnrichProgress(null); qc.invalidateQueries({ queryKey: ["games"] }); }
        }}>
          {enrichingAll ? <><span className="spinner" /> Recherche en cours...</> : "Rechercher pour tous les jeux"}
        </button>

        <span className="badge">{formatNumber(games?.length ?? 0)} jeu(x)</span>
        <div className="row" style={{ marginLeft: "auto", gap: 4, flexWrap: "wrap" }}>
          <select className="select" style={{ maxWidth: 160 }} value={group} onChange={(e) => setGroup(e.target.value)}>
            <option value="none">Sans groupement</option>
            <option value="platform">Par plateforme</option>
            <option value="developer">Par développeur</option>
          </select>
          <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.library.view", "cards"); }} title="Cartes">Cards</button>
          <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.library.view", "list"); }} title="Liste">Liste</button>
          <button className={`btn ${view === "compact" ? "primary" : ""}`} onClick={() => { setView("compact"); localStorage.setItem("retroverse.library.view", "compact"); }} title="Compact">Compact</button>
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

      {selected.size > 0 && (
        <div className="card" style={{ marginBottom: 12, padding: 10, display: "flex", gap: 8, alignItems: "center" }}>
          <span style={{ fontSize: 13, color: "var(--text-dim)" }}>{selected.size} sélectionné(s)</span>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={batching} onClick={batchHash}>
            {batching ? "…" : "Calculer les empreintes"}
          </button>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={batching} onClick={batchEnrich}>
            {batching ? "…" : "Rechercher sur ScreenScraper"}
          </button>
          <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={clearSelection}>Annuler</button>
        </div>
      )}

      {view !== "cards" && (
        <div className="row" style={{ gap: 8, marginBottom: 8, flexWrap: "wrap", fontSize: 12, color: "var(--text-dim)" }}>
          {Object.keys(cols).map((c) => (
            <label key={c} style={{ display: "flex", alignItems: "center", gap: 4 }}>
              <input type="checkbox" checked={cols[c]} onChange={(e) => setCols({ ...cols, [c]: e.target.checked })} />
              {c}
            </label>
          ))}
        </div>
      )}

      {enrichProgress && enrichingAll && (
        <div className="card" style={{ marginBottom: 12, padding: 12 }}>
          <p style={{ fontSize: 13, color: "var(--text-dim)", margin: 0 }}>
            Jeux : {enrichProgress.total} · Éligibles : {enrichProgress.eligible} · Ignorés : {enrichProgress.ignored}
          </p>
          <p style={{ fontSize: 13, color: "var(--text-dim)", margin: "4px 0" }}>
            Traités : {enrichProgress.done} / {enrichProgress.eligible} ({enrichProgress.eligible ? Math.round((enrichProgress.done / enrichProgress.eligible) * 100) : 0}%)
          </p>
          <p style={{ fontSize: 13, color: "var(--text-dim)", margin: "4px 0" }}>
            Réussis : {enrichProgress.success} · Sans résultat : {enrichProgress.no_result} · Erreurs : {enrichProgress.errors}
          </p>
          {enrichProgress.game && <p style={{ fontSize: 12, color: "var(--text-muted)", margin: "4px 0" }}>Jeu actuel : {enrichProgress.game}</p>}
        </div>
      )}

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : total === 0 ? (
        <EmptyState icon="📚" message={t("library.empty")} />
      ) : (
        <>
        {view === "cards" ? (
        <div className="grid games">
          {sortedGames.map((e) => {
            let cover: string | null = null;
            try {
              const d = e.game.metadata_json ? JSON.parse(e.game.metadata_json) : null;
              cover = d?.cover_url || null;
              if (!cover && Array.isArray(d?.medias)) {
                const m = d.medias.find((x: any) => x?.type === "box-2D" || x?.type === "box-3D" || x?.type === "cover");
                cover = m?.url || null;
              }
            } catch {}
            return (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <input type="checkbox" checked={selected.has(e.game.id)} onChange={() => toggleSelect(e.game.id)} onClick={(ev) => ev.stopPropagation()} style={{ position: "absolute", top: 8, left: 8, zIndex: 2 }} />
              <div className="cover">{cover ? <img src={cover} alt="" style={{ width: "100%", height: "100%", objectFit: "contain", borderRadius: 8 }} loading="lazy" /> : "🎮"}</div>
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
                {isEligible(e) && (
                  <button className="btn primary" style={{ fontSize: 11, padding: "2px 8px", marginTop: 6, marginLeft: 4 }} onClick={(ev) => { ev.stopPropagation(); api.enrichGame(e.game.id, "screenscraper").then(() => { qc.invalidateQueries({ queryKey: ["games"] }); api.showNotification("RETROVERSE", "Métadonnées récupérées.").catch(() => {}); }).catch((err) => api.showNotification("RETROVERSE", String(err)).catch(() => {})); }}>
                    Rechercher
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
                <th><input type="checkbox" onChange={(e) => e.target.checked ? selectAll() : clearSelection()} /></th>
                {cols.title && <th style={{ cursor: "pointer" }} onClick={() => toggleSort("title")}>Titre {sortKey === "title" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>}
                {cols.platform && <th style={{ cursor: "pointer" }} onClick={() => toggleSort("platform")}>Plateforme {sortKey === "platform" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>}
                {cols.developer && <th style={{ cursor: "pointer" }} onClick={() => toggleSort("developer")}>Développeur {sortKey === "developer" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>}
                {cols.publisher && <th style={{ cursor: "pointer" }} onClick={() => toggleSort("publisher")}>Éditeur {sortKey === "publisher" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>}
                {cols.files && <th style={{ cursor: "pointer" }} onClick={() => toggleSort("files")}>Fichiers {sortKey === "files" ? (sortDir === "asc" ? "▲" : "▼") : ""}</th>}
                {cols.hashes && <th>Empreintes</th>}
                {cols.pres && <th>Préserv.</th>}
              </tr>
            </thead>
            <tbody>
              <tr style={{ height: offsetY }}><td colSpan={6} style={{ padding: 0, border: "none" }}></td></tr>
              {(group === "none" ? [["_all", visibleRows] as [string, LibraryEntry[]]] : grouped).flatMap(([gkey, items]) => {
                const rows = (group === "none" ? items : items).map((e) => (
                <tr key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)} style={{ cursor: "pointer", height: ROW_HEIGHT }}>
                <td onClick={(ev) => ev.stopPropagation()}><input type="checkbox" checked={selected.has(e.game.id)} onChange={() => toggleSelect(e.game.id)} /></td>
                  {cols.title && <td>{e.game.title}</td>}
                  {cols.platform && <td>{e.platform?.name ?? "Inconnu"}</td>}
                  {cols.developer && <td>{e.developer ?? "—"}</td>}
                  {cols.publisher && <td>{e.publisher ?? "—"}</td>}
                  {cols.files && <td>{e.file_count}</td>}
                  {cols.hashes && <td onClick={(ev) => ev.stopPropagation()}>
                    {needsHash(e) ? (
                      <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} disabled={hashingGame === e.game.id} onClick={() => hashGame(e.game.id)}>
                        {hashingGame === e.game.id ? "…" : "Calculer"}
                      </button>
                    ) : "✓"}
                  </td>}
                  {cols.pres && <td>{presBadge(presStatus(e.game.id))}</td>}
                </tr>
                ));
                if (group !== "none") {
                  return [
                    <tr key={"grp-" + gkey} style={{ background: "var(--bg-3)" }}><td colSpan={Object.values(cols).filter(Boolean).length + 1} style={{ fontWeight: 600, fontSize: 12 }}>{gkey} ({items.length})</td></tr>,
                    ...rows,
                  ];
                }
                return rows;
              })}
              <tr style={{ height: Math.max(0, totalHeight - offsetY - visibleRows.length * ROW_HEIGHT) }}><td colSpan={6} style={{ padding: 0, border: "none" }}></td></tr>
            </tbody>
          </table>
        </div>
        )}
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{formatNumber(total)} jeu(x)</p>
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