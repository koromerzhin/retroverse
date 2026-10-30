import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import { formatBytes } from "../lib/format";
import type { LibraryEntry, GameSource } from "../types";

export default function GameDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const { data, isLoading } = useQuery<LibraryEntry>({
    queryKey: ["game", id],
    queryFn: () => api.getGame(id!),
    enabled: !!id,
  });
  const { data: sources } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });
  const { data: providers } = useQuery<string[]>({
    queryKey: ["metadata-providers"],
    queryFn: () => api.listMetadataProviders(),
  });
  const { data: metadata, refetch: refetchMeta } = useQuery<any[]>({
    queryKey: ["game-metadata", id],
    queryFn: () => api.getGameMetadata(id!),
    enabled: !!id,
  });

  const [title, setTitle] = useState("");
  const [copyMsg, setCopyMsg] = useState<string | null>(null);
  const [enrichMsg, setEnrichMsg] = useState<string | null>(null);
  const [provider, setProvider] = useState("wikipedia");
  useEffect(() => {
    if (data) setTitle(data.game.title);
  }, [data]);

  const sourceMap = new Map(sources?.map((s) => [s.id, s]) || []);
  const isRemote = data?.files.some((f) => sourceMap.get(f.source_id)?.kind === "remote") ?? false;
  const remoteSourceId = data?.files.map((f) => f.source_id).find((sid) => sourceMap.get(sid)?.kind === "remote");

  const saveMutation = useMutation({
    mutationFn: () => api.correctTitle(id!, title),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["game", id] });
      qc.invalidateQueries({ queryKey: ["games"] });
    },
  });

  async function handleEnrich() {
    if (!id) return;
    setEnrichMsg(null);
    try {
      const res = await api.enrichGame(id, provider);
      setEnrichMsg(`Enrichi via ${res.provider} (description: ${res.has_description}, jaquette: ${res.has_cover})`);
      refetchMeta();
    } catch (e: any) {
      setEnrichMsg(String(e));
    }
  }

  async function handleCopy() {
    if (!remoteSourceId || !data) return;
    const dest = await pickDirectory();
    if (!dest) return;
    setCopyMsg(null);
    try {
      const res = await api.copyRemoteFiles(remoteSourceId, data.files.map((f) => f.id), dest);
      setCopyMsg(`Copié : ${res.files} fichier(s), ${res.bytes} octets.${res.errors.length ? " Erreurs: " + res.errors.join("; ") : ""}`);
    } catch (e: any) {
      setCopyMsg(String(e));
    }
  }

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
        {isRemote && (
          <div style={{ marginTop: 12 }}>
            <button className="btn primary" onClick={handleCopy}>Copier localement…</button>
            {copyMsg && <p style={{ color: "var(--text-dim)", marginTop: 8, fontSize: 13 }}>{copyMsg}</p>}
          </div>
        )}
        <div style={{ marginTop: 12, fontSize: 12, color: "var(--text-muted)" }}>
          Créé le {new Date(data.game.created_at).toLocaleString()} · Mis à jour le {new Date(data.game.updated_at).toLocaleString()}
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Enrichissement (RetroData)</h3>
        <div className="row">
          <select className="select" style={{ maxWidth: 220 }} value={provider} onChange={(e) => setProvider(e.target.value)}>
            {(providers || []).map((p) => <option key={p} value={p}>{p}</option>)}
          </select>
          <button className="btn primary" onClick={handleEnrich}>Enrichir</button>
        </div>
        {enrichMsg && <p style={{ color: "var(--text-dim)", marginTop: 8, fontSize: 13 }}>{enrichMsg}</p>}
        {metadata && metadata.length > 0 && (
          <div style={{ marginTop: 12 }}>
            {metadata.map((m: any, i: number) => (
              <div key={i} style={{ fontSize: 13, marginBottom: 6 }}>
                <span className="badge">{m.field ?? m.kind}</span>{" "}
                <span style={{ color: "var(--text-dim)" }}>{m.provider ?? ""}</span>
                {m.value && <p style={{ margin: "4px 0", color: "var(--text)" }}>{String(m.value).slice(0, 300)}</p>}
                {m.url && <a href={m.url} target="_blank" rel="noreferrer" style={{ color: "var(--accent-2)", fontSize: 12 }}>{m.url}</a>}
              </div>
            ))}
          </div>
        )}
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
                <td>{formatBytes(f.size)}</td>
                <td style={{ fontFamily: "monospace", fontSize: 11, color: "var(--text-muted)" }}>{f.path}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}