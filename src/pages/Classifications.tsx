import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { SearchInput, EmptyState } from "../components";
import { useI18n } from "../i18n";

export default function Classifications() {
  const qc = useQueryClient();
  const { t } = useI18n();
  const { data } = useQuery({ queryKey: ["classifications-full"], queryFn: () => api.listClassificationsFull() });
  const [search, setSearch] = useState("");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("asc");
  const [view, setView] = useState<"cards" | "list">(() => (localStorage.getItem("retroverse.classifications.view") as "cards" | "list") || "list");
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
      <div className="card" style={{ marginTop: 16, maxHeight: 480, overflow: "auto" }}>
        <div className="row" style={{ gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={240} />
          <button className="btn" onClick={() => setSortDir((d) => (d === "asc" ? "desc" : "asc"))}>{sortDir === "asc" ? "▲" : "▼"}</button>
          <div className="row" style={{ gap: 4, marginLeft: "auto" }}>
            <button className={`btn ${view === "cards" ? "primary" : ""}`} onClick={() => { setView("cards"); localStorage.setItem("retroverse.classifications.view", "cards"); }}>Cards</button>
            <button className={`btn ${view === "list" ? "primary" : ""}`} onClick={() => { setView("list"); localStorage.setItem("retroverse.classifications.view", "list"); }}>Liste</button>
          </div>
          <button className="btn primary" disabled={syncing} onClick={sync}>
            {syncing ? <><span className="spinner" /> Synchronisation...</> : t("classifications.sync")}
          </button>
        </div>
        {filtered.length === 0 ? (
          <EmptyState icon="🏷️" message="Aucune classification." />
        ) : view === "cards" ? (
        <div className="grid" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 12 }}>
          {filtered.map((c: any) => (
            <div className="card" key={c.id} style={{ padding: 12 }}>
              <div style={{ fontWeight: 600, marginBottom: 4 }}>{c.system_name ?? "—"}</div>
              <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{c.value}</div>
              <div style={{ fontSize: 11, color: "var(--text-muted)", marginTop: 4 }}>SS: {c.ss_id ?? "—"}</div>
            </div>
          ))}
        </div>
        ) : (
        (() => {
          const groups = new Map<string, any[]>();
          for (const c of filtered) {
            const key = c.system_name ?? "—";
            if (!groups.has(key)) groups.set(key, []);
            groups.get(key)!.push(c);
          }
          return Array.from(groups.entries()).map(([sys, items]) => (
            <div key={sys} style={{ marginBottom: 12 }}>
              <h4 style={{ margin: "8px 0 4px", color: "var(--text-dim)", fontSize: 13 }}>{sys} ({items.length})</h4>
              <table className="table">
                <thead style={{ position: "sticky", top: 0, background: "var(--bg-2)", zIndex: 1 }}><tr><th>Valeur</th><th>ID SS</th></tr></thead>
                <tbody>
                  {items.map((c: any) => (
                    <tr key={c.id}>
                      <td>{c.value}</td>
                      <td>{c.ss_id ?? "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ));
        })()
        )}
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{filtered.length} classification(s)</p>
      </div>
    </>
  );
}
