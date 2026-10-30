import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { LibraryEntry } from "../types";

export default function GameDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { data, isLoading } = useQuery<LibraryEntry>({
    queryKey: ["game", id],
    queryFn: () => api.getGame(id!),
    enabled: !!id,
  });

  const [title, setTitle] = useState("");
  useEffect(() => {
    if (data) setTitle(data.game.title);
  }, [data]);

  const saveMutation = useMutation({
    mutationFn: () => api.correctTitle(id!, title),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["game", id] });
      qc.invalidateQueries({ queryKey: ["games"] });
    },
  });

  if (isLoading || !data) {
    return <div className="empty"><div className="spinner" /></div>;
  }

  return (
    <>
      <button className="btn" onClick={() => navigate(-1)} style={{ marginBottom: 16 }}>← Retour</button>
      <h1 className="page-title">{data.game.title}</h1>
      <p className="page-sub">{data.platform?.name ?? "Plateforme inconnue"}</p>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Informations</h3>
        <div className="row">
          <input className="input" value={title} onChange={(e) => setTitle(e.target.value)} style={{ maxWidth: 400 }} />
          <button className="btn primary" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
            {saveMutation.isPending ? <span className="spinner" /> : "Enregistrer le titre"}
          </button>
        </div>
        {saveMutation.isSuccess && <p style={{ color: "var(--success)", marginTop: 8 }}>Titre mis à jour.</p>}
        <div style={{ marginTop: 12, fontSize: 12, color: "var(--text-muted)" }}>
          Créé le {new Date(data.game.created_at).toLocaleString()} · Mis à jour le {new Date(data.game.updated_at).toLocaleString()}
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Fichiers ({data.files.length})</h3>
        <table className="table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Extension</th>
              <th>Taille</th>
              <th>Chemin</th>
            </tr>
          </thead>
          <tbody>
            {data.files.map((f) => (
              <tr key={f.id}>
                <td>{f.filename}</td>
                <td><span className="badge">{f.extension}</span></td>
                <td>{(f.size / 1024).toFixed(1)} Ko</td>
                <td style={{ fontFamily: "monospace", fontSize: 11, color: "var(--text-muted)" }}>{f.path}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}