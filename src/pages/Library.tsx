import { useState, useEffect } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import type { LibraryEntry, Platform } from "../types";

export default function Library() {
  const navigate = useNavigate();
  const { t } = useI18n();
  const [searchParams] = useSearchParams();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const platform = searchParams.get("platform") || undefined;
  const [page, setPage] = useState(1);
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.library.view") as "cards" | "list") || "cards");
  const pageSize = 24;

  useEffect(() => {
    const t = setTimeout(() => setSearch(searchInput), 300);
    return () => clearTimeout(t);
  }, [searchInput]);

  useEffect(() => { setPage(1); }, [search, platform]);

  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
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
        : await api.listGames(platform, undefined, pageSize, (page - 1) * pageSize);
      return r;
    },
  });

  const games = result?.items ?? [];
  const total = result?.total ?? 0;

  const platformCounts: Record<string, number> = platformCountsData || {};
  const availablePlatforms = (platforms || []).filter((p) => (platformCounts[p.id] ?? 0) > 0);

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

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
        <button className="btn" onClick={() => refetch()}>{t("common.refresh")}</button>
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
          {games.map((e) => (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">🎮</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s)</div>
              </div>
            </div>
          ))}
        </div>
        ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Titre</th>
              <th>Plateforme</th>
              <th>Fichiers</th>
            </tr>
          </thead>
          <tbody>
            {games.map((e) => (
              <tr key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)} style={{ cursor: "pointer" }}>
                <td>{e.game.title}</td>
                <td>{e.platform?.name ?? "Inconnu"}</td>
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