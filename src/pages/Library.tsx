import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import type { LibraryEntry, Platform } from "../types";

export default function Library() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [search, setSearch] = useState("");
  const [platform] = useState<string | undefined>(searchParams.get("platform") || undefined);

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

  return (
    <>
      <h1 className="page-title">Bibliothèque</h1>
      <p className="page-sub">Tous les jeux de votre collection.</p>

      <div className="row" style={{ marginBottom: 18 }}>
        <input
          className="input"
          style={{ maxWidth: 320 }}
          placeholder="Rechercher un jeu..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {platform && (
          <span className="badge">{platforms?.find((p) => p.id === platform)?.name}</span>
        )}
        <button className="btn" onClick={() => refetch()}>Actualiser</button>
        <span className="badge">{formatNumber(games?.length ?? 0)} jeu(x)</span>
      </div>

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : (games?.length ?? 0) === 0 ? (
        <div className="empty">
          <div className="icon">📚</div>
          <p>Aucun jeu trouvé.</p>
        </div>
      ) : (
        <div className="grid games">
          {games!.map((e) => (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">🎮</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s)</div>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}