import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput } from "../components";
import { useI18n } from "../i18n";

export default function Classifications() {
  const qc = useQueryClient();
  const { t } = useI18n();
  const { data } = useQuery({ queryKey: ["classifications-full"], queryFn: () => api.listClassificationsFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [syncing, setSyncing] = useState(false);

  const sync = async () => {
    setSyncing(true);
    try {
      const n: number = await api.syncClassifications();
      qc.invalidateQueries({ queryKey: ["classifications-full"] });
      api.showNotification("RETROVERSE", `Synchronisation terminée : ${n} classification(s).`).catch(() => {});
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setSyncing(false);
    }
  };

  const filtered = (data || [])
    .filter((c: any) => !search || (c.system_name || "").toLowerCase().includes(search.toLowerCase()) || (c.value || "").toLowerCase().includes(search.toLowerCase()))
    .sort((a: any, b: any) => {
      const sa = (a.system_name || "") + " " + (a.value || "");
      const sb = (b.system_name || "") + " " + (b.value || "");
      return sortDir === "asc" ? sa.localeCompare(sb) : sb.localeCompare(sa);
    });

  return (
    <>
      <h1 className="page-title">{t("nav.classifications")}</h1>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
          <button className="btn primary" disabled={syncing} onClick={sync}>
            {syncing ? <><span className="spinner" /> Synchronisation...</> : t("classifications.sync")}
          </button>
        </div>
        <table className="table">
          <thead><tr><th>Organisme</th><th>Valeur</th><th>ID SS</th></tr></thead>
          <tbody>
            {filtered.map((c: any) => (
              <tr key={c.id}>
                <td>{c.system_name ?? "—"}</td>
                <td>{c.value}</td>
                <td>{c.ss_id ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} classification(s)</p>
      </div>
    </>
  );
}
