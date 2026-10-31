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
  const [search, setSearch] = useState("");
  const [platform] = useState<string | undefined>(searchParams.get("platform") || undefined);
  const [page, setPage] = useState(1);
  const pageSize = 24;

  useEffect(() => { setPage(1); }, [search, platform]);

  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
  const { data: games, isLoading, refetch } = useQuery<LibraryEntry[]>({
    queryKey: ["games", platform, search],
    queryFn: async () => {
      const base = search.trim() ? await api.searchGamesFts(search.trim()) : await api.listGames();
      if (platform) return base.filter((g: any) => g.game.platform_id === platform);
      return base;
    },
  });

  const totalPages = Math.max(1, Math.ceil((games?.length ?? 0) / pageSize));
  const paged = (games || []).slice((page - 1) * pageSize, page * pageSize);

  return (
    <>
      <h1 className="page-title">{t("library.title")}</h1>
      <p className="page-sub">{t("library.subtitle")}</p>

      <div className="row" style={{ marginBottom: 18 }}>
        <input
          className="input"
          style={{ maxWidth: 320 }}
          placeholder={t("library.search")}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {platform && (
          <span className="badge">{platforms?.find((p) => p.id === platform)?.name}</span>
        )}
        <button className="btn" onClick={() => refetch()}>{t("common.refresh")}</button>
        <span className="badge">{formatNumber(games?.length ?? 0)} jeu(x)</span>
      </div>

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : (games?.length ?? 0) === 0 ? (
        <div className="empty">
          <div className="icon">📚</div>
          <p>{t("library.empty")}</p>
        </div>
      ) : (
        <>
        <div className="grid games">
          {paged.map((e) => (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">🎮</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s)</div>
              </div>
            </div>
          ))}
        </div>
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