import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";

export default function Collections() {
  const qc = useQueryClient();
  const { data: collections } = useQuery({ queryKey: ["collections"], queryFn: () => api.listCollections() });
  const [name, setName] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const { data: games } = useQuery({ queryKey: ["collection-games", selected], queryFn: () => (selected ? api.collectionGames(selected) : Promise.resolve([])), enabled: !!selected });

  const create = async () => {
    if (!name.trim()) return;
    await api.createCollection(name.trim());
    setName("");
    qc.invalidateQueries({ queryKey: ["collections"] });
  };

  return (
    <>
      <h1 className="page-title">Collections</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12 }}>
          <input className="input" placeholder="Nouvelle collection…" value={name} onChange={(e) => setName(e.target.value)} style={{ maxWidth: 240 }} />
          <button className="btn primary" onClick={create}>Créer</button>
        </div>
        <div className="row" style={{ gap: 8, flexWrap: "wrap" }}>
          {(collections || []).map((c: any) => (
            <button key={c.id} className={`btn ${selected === c.id ? "primary" : ""}`} onClick={() => setSelected(c.id)}>
              {c.name}
            </button>
          ))}
        </div>
        {selected && (
          <div style={{ marginTop: 16 }}>
            <p style={{ color: "var(--text-dim)", fontSize: 12 }}>{(games || []).length} jeu(x) dans cette collection.</p>
            <ul style={{ fontSize: 12, color: "var(--text-dim)", paddingLeft: 16 }}>
              {(games || []).map((g: string) => <li key={g}>{g}</li>)}
            </ul>
          </div>
        )}
      </div>
    </>
  );
}
