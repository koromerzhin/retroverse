import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import type { GameSource, ScanResult } from "../types";

export default function Sources() {
  const qc = useQueryClient();
  const { data: sources, isLoading } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });

  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [scanning, setScanning] = useState<string | null>(null);
  const [lastResult, setLastResult] = useState<ScanResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const addMutation = useMutation({
    mutationFn: () => api.addSource(name.trim() || "Nouvelle source", path),
    onSuccess: () => {
      setName("");
      setPath("");
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
    },
    onError: (e: any) => setError(String(e)),
  });

  const removeMutation = useMutation({
    mutationFn: (id: string) => api.removeSource(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
    },
  });

  async function handlePick() {
    const dir = await pickDirectory();
    if (dir) {
      setPath(dir);
      if (!name) setName(dir.split("/").pop() || "Source");
    }
  }

  async function handleScan(id: string) {
    setScanning(id);
    setError(null);
    try {
      const res = await api.scanSource(id);
      setLastResult(res);
      qc.invalidateQueries({ queryKey: ["games"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
      qc.invalidateQueries({ queryKey: ["sources"] });
    } catch (e: any) {
      setError(String(e));
    } finally {
      setScanning(null);
    }
  }

  return (
    <>
      <h1 className="page-title">Sources</h1>
      <p className="page-sub">Gérez les dossiers scannés pour votre collection.</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <h3 style={{ marginTop: 0 }}>Ajouter une source locale</h3>
        <div className="row">
          <input className="input" placeholder="Nom" value={name} onChange={(e) => setName(e.target.value)} style={{ maxWidth: 220 }} />
          <input className="input" placeholder="Chemin du dossier" value={path} onChange={(e) => setPath(e.target.value)} />
          <button className="btn" onClick={handlePick}>Parcourir…</button>
          <button className="btn primary" onClick={() => addMutation.mutate()} disabled={!path || addMutation.isPending}>
            {addMutation.isPending ? <span className="spinner" /> : "Ajouter"}
          </button>
        </div>
        {error && <p style={{ color: "var(--danger)", marginTop: 10 }}>{error}</p>}
      </div>

      {lastResult && (
        <div className="card" style={{ marginBottom: 20, borderColor: "var(--success)" }}>
          <strong>Scan terminé :</strong> {lastResult.files_found} fichier(s), {lastResult.games_created} jeu(x) créé(s), {lastResult.games_updated} mis à jour.
        </div>
      )}

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : (sources?.length ?? 0) === 0 ? (
        <div className="empty">
          <div className="icon">📁</div>
          <p>Aucune source configurée.</p>
        </div>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Chemin</th>
              <th>Dernier scan</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sources!.map((s) => (
              <tr key={s.id}>
                <td>{s.name}</td>
                <td style={{ fontFamily: "monospace", fontSize: 12 }}>{s.path}</td>
                <td>{s.last_scan_at ? new Date(s.last_scan_at).toLocaleString() : "—"}</td>
                <td>
                  <div className="row">
                    <button
                      className="btn primary"
                      onClick={() => handleScan(s.id)}
                      disabled={scanning === s.id}
                    >
                      {scanning === s.id ? <><span className="spinner" /> Scan…</> : "Scanner"}
                    </button>
                    <button
                      className="btn danger"
                      onClick={() => { if (confirm("Supprimer cette source ?")) removeMutation.mutate(s.id); }}
                    >
                      Supprimer
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}