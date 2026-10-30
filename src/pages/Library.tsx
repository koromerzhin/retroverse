import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { LibraryEntry, Platform } from "../types";

export default function Library() {
  const navigate = useNavigate();
  const [search, setSearch] = useState("");
  const [platform, setPlatform] = useState<string | undefined>(undefined);

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
        <select
          className="select"
          style={{ maxWidth: 220 }}
          value={platform ?? ""}
          onChange={(e) => setPlatform(e.target.value || undefined)}
        >
          <option value="">Toutes les plateformes</option>
          {platforms?.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
        <button className="btn" onClick={() => refetch()}>Actualiser</button>
        <span className="badge">{games?.length ?? 0} jeu(x)</span>
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