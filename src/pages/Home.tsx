import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import type { Stats } from "../types";

export default function Home() {
  const navigate = useNavigate();
  const { t } = useI18n();
  const { data: stats } = useQuery<Stats>({
    queryKey: ["stats"],
    queryFn: () => api.stats(),
  });
  const { data: games } = useQuery({
    queryKey: ["games", "recent"],
    queryFn: () => api.listGames(undefined, undefined, 8, 0),
  });

  const recent = (games?.items || []).slice(0, 8);

  return (
    <>
      <h1 className="page-title">{t("home.title")}</h1>
      <p className="page-sub">{t("home.subtitle")}</p>

      <div className="stat-grid">
        <div className="stat"><div className="label">{t("home.stat.games")}</div><div className="value">{formatNumber(stats?.games ?? 0)}</div></div>
        <div className="stat"><div className="label">{t("home.stat.files")}</div><div className="value">{formatNumber(stats?.files ?? 0)}</div></div>
        <div className="stat"><div className="label">{t("home.stat.sources")}</div><div className="value">{formatNumber(stats?.sources ?? 0)}</div></div>
        <div className="stat"><div className="label">{t("home.stat.platforms")}</div><div className="value">{formatNumber(stats?.platforms ?? 0)}</div></div>
      </div>

      <h2 style={{ marginTop: 28, fontSize: 18 }}>{t("home.recent")}</h2>
      {recent.length === 0 ? (
        <div className="empty">
          <div className="icon">🎮</div>
          <p>{t("home.empty")}</p>
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