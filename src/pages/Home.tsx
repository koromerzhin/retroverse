import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import { EmptyState } from "../components";
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
    queryFn: () => api.listGames(undefined, undefined, undefined, undefined, 8, 0),
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

      <div className="card" style={{ marginTop: 20, marginBottom: 20 }}>
        <h3 style={{ marginTop: 0 }}>Actions rapides</h3>
        <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
          <button className="btn primary" onClick={() => navigate("/platforms")}>Scanner les plateformes</button>
          <button className="btn primary" onClick={() => navigate("/library")}>Parcourir la bibliothèque</button>
          <button className="btn" onClick={() => navigate("/discovery")}>Découvrir les doublons</button>
        </div>
      </div>

      <h2 style={{ marginTop: 28, fontSize: 18 }}>{t("home.recent")}</h2>
      {recent.length === 0 ? (
        <EmptyState icon="🎮" message={t("home.empty")} />
      ) : (
        <div className="grid games" style={{ marginTop: 14 }}>
          {recent.map((e: any) => {
            let cover: string | null = null;
            try {
              const d = e.game.metadata_json ? JSON.parse(e.game.metadata_json) : null;
              cover = d?.cover_url || null;
              if (!cover && Array.isArray(d?.medias)) {
                const m = d.medias.find((x: any) => x?.type === "box-2D" || x?.type === "box-3D");
                cover = m?.url || null;
              }
            } catch {}
            return (
            <div className="game-card" key={e.game.id} onClick={() => navigate(`/game/${e.game.id}`)}>
              <div className="cover">{cover ? <img src={cover} alt="" style={{ width: "100%", height: "100%", objectFit: "contain", borderRadius: 8 }} loading="lazy" /> : "🎮"}</div>
              <div className="info">
                <div className="title">{e.game.title}</div>
                <div className="meta">{e.platform?.name ?? "Inconnu"} · {e.file_count} fichier(s)</div>
              </div>
            </div>
            );
          })}
        </div>
      )}
    </>
  );
}