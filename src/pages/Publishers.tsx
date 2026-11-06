import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";

export default function Publishers() {
  const { data } = useQuery({ queryKey: ["publishers-full"], queryFn: () => api.listPublishersFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const filtered = (data || [])
    .filter((p: any) => !search || p.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => sortDir === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

  return (
    <>
      <h1 className="page-title">Éditeurs</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
        </div>
        <table className="table">
          <thead><tr><th>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>ID SS</th></tr></thead>
          <tbody>
            {filtered.map((p: any) => (
              <tr key={p.id}>
                <td>{p.name}</td>
                <td>{p.ss_id ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} éditeur(s)</p>
      </div>
    </>
  );
}
