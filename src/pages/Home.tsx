import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { Stats } from "../types";

export default function Home() {
  const navigate = useNavigate();
  const { data: stats } = useQuery<Stats>({
    queryKey: ["stats"],
    queryFn: () => api.stats(),
  });
  const { data: games } = useQuery({
    queryKey: ["games", "recent"],
    queryFn: () => api.listGames(),
  });

  const recent = (games || []).slice(0, 8);

  return (
    <>
      <h1 className="page-title">Accueil</h1>
      <p className="page-sub">Bienvenue dans RETROVERSE — votre compagnon de collection vidéoludique.</p>

      <div className="stat-grid">
        <div className="stat"><div className="label">Jeux</div><div className="value">{stats?.games ?? 0}</div></div>
        <div className="stat"><div className="label">Fichiers</div><div className="value">{stats?.files ?? 0}</div></div>
        <div className="stat"><div className="label">Sources</div><div className="value">{stats?.sources ?? 0}</div></div>
        <div className="stat"><div className="label">Plateformes</div><div className="value">{stats?.platforms ?? 0}</div></div>
      </div>

      <h2 style={{ marginTop: 28, fontSize: 18 }}>Jeux récemment découverts</h2>
      {recent.length === 0 ? (
        <div className="empty">
          <div className="icon">🎮</div>
          <p>Aucun jeu pour le moment. Ajoutez une source pour commencer.</p>
        </div>
      ) : (
        <div className="grid games" style={{ marginTop: 14 }}>
          {recent.map((e: any) => (
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