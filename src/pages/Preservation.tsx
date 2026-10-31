import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatBytes } from "../lib/format";
import { useI18n } from "../i18n";

export default function Preservation() {
  const qc = useQueryClient();
  const { t } = useI18n();
  const { data: jobs } = useQuery({ queryKey: ["transfer-jobs"], queryFn: () => api.listTransferJobs() });
  const [msg, setMsg] = useState<string | null>(null);

  const hashMut = useMutation({
    mutationFn: async () => {
      const n = await api.computeHashes();
      return n;
    },
    onSuccess: (n) => setMsg(`Empreintes calculées pour ${n} fichier(s).`),
    onError: (e: any) => setMsg(String(e)),
  });

  async function handleExport() {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const dest = await save({ defaultPath: "retroverse-backup.json" });
    if (!dest) return;
    try {
      const p = await api.exportBackup(dest);
      setMsg(`Sauvegarde exportée : ${p}`);
    } catch (e: any) {
      setMsg(String(e));
    }
  }

  async function handleImport() {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const src = await open({ multiple: false, filters: [{ name: "JSON", extensions: ["json"] }] });
    if (typeof src !== "string") return;
    try {
      const n = await api.importBackup(src);
      setMsg(`Import terminé : ${n} élément(s) ajouté(s).`);
      qc.invalidateQueries();
    } catch (e: any) {
      setMsg(String(e));
    }
  }

  return (
    <>
      <h1 className="page-title">{t("preservation.title")}</h1>
      <p className="page-sub">{t("preservation.subtitle")}</p>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Intégrité</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>Calcule les empreintes SHA-256 des fichiers locaux.</p>
        <button className="btn primary" onClick={() => hashMut.mutate()} disabled={hashMut.isPending}>
          {hashMut.isPending ? <span className="spinner" /> : "Calculer les empreintes"}
        </button>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Sauvegarde / Restauration</h3>
        <div className="row">
          <button className="btn primary" onClick={handleExport}>Exporter une sauvegarde</button>
          <button className="btn" onClick={handleImport}>Importer une sauvegarde</button>
        </div>
        {msg && <p style={{ color: "var(--text-dim)", marginTop: 10, fontSize: 13 }}>{msg}</p>}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Historique des transferts</h3>
        {(jobs || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucun transfert.</p>
        ) : (
          <table className="table">
            <thead>
              <tr><th>Date</th><th>Statut</th><th>Octets</th><th>Erreur</th></tr>
            </thead>
            <tbody>
              {(jobs || []).map((j: any) => (
                <tr key={j.id}>
                  <td>{new Date(j.created_at).toLocaleString()}</td>
                  <td><span className="badge">{j.status}</span></td>
                  <td>{formatBytes(j.transferred_bytes)} / {formatBytes(j.total_bytes)}</td>
                  <td style={{ fontSize: 12, color: "var(--danger)" }}>{j.error ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </>
  );
}