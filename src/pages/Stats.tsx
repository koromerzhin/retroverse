import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n } from "../i18n";

export default function Stats() {
  const { t } = useI18n();
  const { data: platforms } = useQuery({ queryKey: ["platforms"], queryFn: () => api.listPlatforms() });
  const { data: counts } = useQuery({ queryKey: ["platform-game-counts"], queryFn: () => api.platformGameCounts() });
  const { data: fileCounts } = useQuery({ queryKey: ["platform-file-counts"], queryFn: () => api.platformFileCounts() });

  const rows = (platforms || []).map((p: any) => ({
    name: p.name,
    games: (counts || {})[p.id] ?? 0,
    files: (fileCounts || {})[p.id] ?? 0,
  })).filter((r: any) => r.games > 0).sort((a: any, b: any) => b.games - a.games);

  const totalGames = rows.reduce((s: number, r: any) => s + r.games, 0);
  const totalFiles = rows.reduce((s: number, r: any) => s + r.files, 0);
  const maxGames = rows.length ? Math.max(...rows.map((r: any) => r.games)) : 1;

  return (
    <>
      <h1 className="page-title">{t("nav.stats")}</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>Jeux : {totalGames} · Fichiers : {totalFiles} · Plateformes : {rows.length}</p>

        <h3 style={{ marginTop: 12 }}>Jeux par plateforme</h3>
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          {rows.slice(0, 15).map((r: any) => (
            <div key={r.name} style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <span style={{ width: 160, fontSize: 12, color: "var(--text-dim)", textAlign: "right", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{r.name}</span>
              <div style={{ flex: 1, height: 14, background: "var(--bg-3)", borderRadius: 4, overflow: "hidden" }}>
                <div style={{ height: "100%", width: `${(r.games / maxGames) * 100}%`, background: "var(--accent)", borderRadius: 4 }} />
              </div>
              <span style={{ width: 40, fontSize: 12, color: "var(--text)" }}>{r.games}</span>
            </div>
          ))}
        </div>

        <h3 style={{ marginTop: 16 }}>Détail par plateforme</h3>
        <table className="table">
          <thead><tr><th>{t("stats.platform")}</th><th>{t("stats.games")}</th><th>{t("stats.files")}</th><th>Ratio fichiers/jeu</th></tr></thead>
          <tbody>
            {rows.map((r: any) => (
              <tr key={r.name}>
                <td>{r.name}</td>
                <td>{r.games}</td>
                <td>{r.files}</td>
                <td>{r.games ? (r.files / r.games).toFixed(1) : "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
