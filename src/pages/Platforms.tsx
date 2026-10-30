import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { Platform } from "../types";

export default function Platforms() {
  const { data: platforms } = useQuery<Platform[]>({
    queryKey: ["platforms"],
    queryFn: () => api.listPlatforms(),
  });
  const { data: games } = useQuery({
    queryKey: ["games"],
    queryFn: () => api.listGames(),
  });

  const counts: Record<string, number> = {};
  games?.forEach((g: any) => {
    const pid = g.game.platform_id;
    if (pid) counts[pid] = (counts[pid] || 0) + 1;
  });

  return (
    <>
      <h1 className="page-title">Plateformes</h1>
      <p className="page-sub">Parcourez votre collection par plateforme.</p>

      <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
        {platforms?.map((p) => (
          <div className="card" key={p.id}>
            <div style={{ fontSize: 28, marginBottom: 8 }}>🎮</div>
            <div style={{ fontWeight: 600 }}>{p.name}</div>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 4 }}>
              {p.manufacturer ?? "—"} {p.release_year ? `· ${p.release_year}` : ""}
            </div>
            <div style={{ marginTop: 10 }}>
              <span className="badge">{counts[p.id] ?? 0} jeu(x)</span>
            </div>
          </div>
        ))}
      </div>
    </>
  );
}