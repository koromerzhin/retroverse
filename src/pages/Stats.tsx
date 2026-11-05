import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";

export default function Stats() {
  const { data: platforms } = useQuery({ queryKey: ["platforms"], queryFn: () => api.listPlatforms() });
  const { data: counts } = useQuery({ queryKey: ["platform-game-counts"], queryFn: () => api.platformGameCounts() });
  const { data: fileCounts } = useQuery({ queryKey: ["platform-file-counts"], queryFn: () => api.platformFileCounts() });

  const rows = (platforms || []).map((p: any) => ({
    name: p.name,
    games: (counts || {})[p.id] ?? 0,
    files: (fileCounts || {})[p.id] ?? 0,
  })).sort((a: any, b: any) => b.games - a.games);

  const totalGames = rows.reduce((s: number, r: any) => s + r.games, 0);
  const totalFiles = rows.reduce((s: number, r: any) => s + r.files, 0);

  return (
    <>
      <h1 className="page-title">Statistiques</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>Jeux : {totalGames} · Fichiers : {totalFiles} · Plateformes : {rows.length}</p>
        <table className="table">
          <thead><tr><th>Plateforme</th><th>Jeux</th><th>Fichiers</th></tr></thead>
          <tbody>
            {rows.map((r: any) => (
              <tr key={r.name}><td>{r.name}</td><td>{r.games}</td><td>{r.files}</td></tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
