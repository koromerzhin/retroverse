import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";

export default function Companion() {
  const { data: emudeck } = useQuery({ queryKey: ["emudeck"], queryFn: () => api.detectEmudeck() });
  const { data: emus } = useQuery({ queryKey: ["emus"], queryFn: () => api.detectEmulators() });
  const { data: stats } = useQuery({ queryKey: ["session-stats"], queryFn: () => api.sessionStats() });
  const { data: sessions } = useQuery({ queryKey: ["sessions"], queryFn: () => api.listSessions() });

  return (
    <>
      <h1 className="page-title">Companion</h1>
      <p className="page-sub">Intégration avec les logiciels existants et historique de jeu.</p>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>EmuDeck</h3>
        {emudeck ? (
          <p style={{ color: "var(--success)" }}>Détecté : <code>{emudeck}</code></p>
        ) : (
          <p style={{ color: "var(--text-muted)" }}>Non détecté.</p>
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Émulateurs en cours d'exécution</h3>
        {(emus || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucun émulateur détecté.</p>
        ) : (
          <div className="row">{(emus || []).map((e: string) => <span className="badge" key={e}>{e}</span>)}</div>
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Statistiques de sessions</h3>
        <div className="row">
          <span className="badge">Sessions : {stats?.total_sessions ?? 0}</span>
          <span className="badge">Temps total : {Math.round((stats?.total_seconds ?? 0) / 60)} min</span>
        </div>
        {(stats?.top || []).length > 0 && (
          <table className="table" style={{ marginTop: 12 }}>
            <thead><tr><th>Jeu</th><th>Sessions</th><th>Temps (min)</th></tr></thead>
            <tbody>
              {stats.top.map((t: any) => (
                <tr key={t.game_id}><td>{t.game_id.slice(0,8)}</td><td>{t.sessions}</td><td>{Math.round(t.seconds/60)}</td></tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Sessions récentes</h3>
        {(sessions || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucune session.</p>
        ) : (
          <table className="table">
            <thead><tr><th>Jeu</th><th>Début</th><th>Durée</th></tr></thead>
            <tbody>
              {(sessions || []).slice(0, 20).map((s: any) => (
                <tr key={s.id}>
                  <td>{s.game_id.slice(0,8)}</td>
                  <td>{new Date(s.started_at).toLocaleString()}</td>
                  <td>{s.duration_seconds ? Math.round(s.duration_seconds/60) + " min" : "en cours"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </>
  );
}