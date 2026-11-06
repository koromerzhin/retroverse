import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";

export default function Developers() {
  const { data } = useQuery({ queryKey: ["developers-full"], queryFn: () => api.listDevelopersFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");

  const filtered = (data || [])
    .filter((d: any) => !search || d.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => sortDir === "asc" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name));

  return (
    <>
      <h1 className="page-title">Développeurs</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
        </div>
        <table className="table">
          <thead><tr><th style={{ cursor: "pointer" }} onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>Nom {sortDir === "asc" ? "▲" : "▼"}</th><th>ID SS</th></tr></thead>
          <tbody>
            {filtered.map((d: any) => (
              <tr key={d.id}>
                <td>{d.name}</td>
                <td>{d.game_count ?? 0}</td>
                <td>{d.ss_id ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} développeur(s)</p>
      </div>
    </>
  );
}
