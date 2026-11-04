import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n } from "../i18n";

export default function Genres() {
  const { lang } = useI18n();
  const qc = useQueryClient();
  const { data } = useQuery({ queryKey: ["genres-full"], queryFn: () => api.listGenresFull() });
  const [name, setName] = useState("");
  const [parentId, setParentId] = useState("");

  const nameOf = (g: any) => {
    const map: Record<string, string | null> = { fr: g.name_fr, en: g.name_en, de: g.name_de, es: g.name_es, it: g.name_it, pt: g.name_pt };
    return map[lang] || g.name_en || g.name_fr || g.name_de || g.name_es || g.name_it || g.name_pt || "—";
  };

  async function add() {
    if (!name.trim()) return;
    await api.addGenre({ nameEn: name.trim(), parentId: parentId || undefined });
    setName(""); setParentId("");
    qc.invalidateQueries({ queryKey: ["genres-full"] });
  }
  async function remove(id: string) {
    if (!confirm("Supprimer ce genre ?")) return;
    try {
      await api.deleteGenre(id);
      qc.invalidateQueries({ queryKey: ["genres-full"] });
    } catch (e) { alert(String(e)); }
  }

  return (
    <>
      <h1 className="page-title">Genres</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <input className="input" placeholder="Nom (EN)" value={name} onChange={(e) => setName(e.target.value)} style={{ maxWidth: 240 }} />
          <select className="select" style={{ maxWidth: 240 }} value={parentId} onChange={(e) => setParentId(e.target.value)}>
            <option value="">Genre principal</option>
            {(data || []).map((g: any) => <option key={g.id} value={g.id}>{nameOf(g)}</option>)}
          </select>
          <button className="btn primary" onClick={add}>Ajouter</button>
        </div>
        <table className="table">
          <thead><tr><th>Nom</th><th>Parent</th><th>Actions</th></tr></thead>
          <tbody>
            {(data || []).map((g: any) => (
              <tr key={g.id}>
                <td>{nameOf(g)}</td>
                <td>{g.parent_id ? (data || []).find((x: any) => x.id === g.parent_id)?.name_en || g.parent_id : "—"}</td>
                <td><button className="btn danger" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => remove(g.id)}>Supprimer</button></td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{(data || []).length} genre(s)</p>
      </div>
    </>
  );
}
