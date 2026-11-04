import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";

export default function Publishers() {
  const qc = useQueryClient();
  const { data } = useQuery({ queryKey: ["publishers-full"], queryFn: () => api.listPublishersFull() });
  const [name, setName] = useState("");
  const [ssId, setSsId] = useState("");

  async function add() {
    if (!name.trim()) return;
    await api.addPublisher(name.trim(), ssId.trim() || undefined);
    setName(""); setSsId("");
    qc.invalidateQueries({ queryKey: ["publishers-full"] });
  }
  async function remove(id: string) {
    if (!confirm("Supprimer cet éditeur ?")) return;
    try {
      await api.deletePublisher(id);
      qc.invalidateQueries({ queryKey: ["publishers-full"] });
    } catch (e) { alert(String(e)); }
  }

  return (
    <>
      <h1 className="page-title">Éditeurs</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12 }}>
          <input className="input" placeholder="Nom" value={name} onChange={(e) => setName(e.target.value)} style={{ maxWidth: 240 }} />
          <input className="input" placeholder="ID ScreenScraper" value={ssId} onChange={(e) => setSsId(e.target.value)} style={{ maxWidth: 200 }} />
          <button className="btn primary" onClick={add}>Ajouter</button>
        </div>
        <table className="table">
          <thead><tr><th>Nom</th><th>ID SS</th><th>Actions</th></tr></thead>
          <tbody>
            {(data || []).map((p: any) => (
              <tr key={p.id}>
                <td>{p.name}</td>
                <td>{p.ss_id ?? "—"}</td>
                <td><button className="btn danger" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => remove(p.id)}>Supprimer</button></td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{(data || []).length} éditeur(s)</p>
      </div>
    </>
  );
}
