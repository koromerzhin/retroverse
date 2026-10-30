import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";

export default function Discovery() {
  const qc = useQueryClient();
  const { data: duplicates } = useQuery({ queryKey: ["duplicates"], queryFn: () => api.findDuplicates() });
  const { data: collections } = useQuery({ queryKey: ["collections"], queryFn: () => api.listCollections() });
  const { data: games } = useQuery({ queryKey: ["games"], queryFn: () => api.listGames() });

  const [collName, setCollName] = useState("");
  const createMut = useMutation({
    mutationFn: () => api.createCollection(collName.trim()),
    onSuccess: () => { setCollName(""); qc.invalidateQueries({ queryKey: ["collections"] }); },
  });

  // forgotten: games with 0 files? or never? We'll show games with missing files
  const missing = (games || []).filter((g: any) => g.files.some((f: any) => f.missing === 1) || g.files.length === 0);

  return (
    <>
      <h1 className="page-title">Découverte</h1>
      <p className="page-sub">Explorez et organisez votre collection.</p>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Doublons détectés</h3>
        {(duplicates || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucun doublon.</p>
        ) : (
          (duplicates || []).map((d: any, i: number) => (
            <div key={i} style={{ marginBottom: 10 }}>
              <strong>{d.title}</strong> — {d.games.length} exemplaires
              <ul style={{ margin: "4px 0", paddingLeft: 20, fontSize: 13, color: "var(--text-dim)" }}>
                {d.games.map((g: any) => <li key={g[0]}>{g[1]} ({g[0].slice(0,8)})</li>)}
              </ul>
            </div>
          ))
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Collections</h3>
        <div className="row" style={{ marginBottom: 12 }}>
          <input className="input" placeholder="Nouvelle collection" value={collName} onChange={(e) => setCollName(e.target.value)} style={{ maxWidth: 260 }} />
          <button className="btn primary" onClick={() => createMut.mutate()} disabled={!collName.trim() || createMut.isPending}>Créer</button>
        </div>
        {(collections || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucune collection.</p>
        ) : (
          <ul style={{ paddingLeft: 20, fontSize: 13 }}>
            {(collections || []).map((c: any) => <li key={c.id}>{c.name}</li>)}
          </ul>
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Jeux à redécouvrir</h3>
        {missing.length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Rien à signaler.</p>
        ) : (
          <p style={{ color: "var(--text-dim)", fontSize: 13 }}>{formatNumber(missing.length)} jeu(x) avec fichiers manquants ou sans fichier.</p>
        )}
      </div>
    </>
  );
}