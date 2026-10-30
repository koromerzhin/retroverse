import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { AppInfo } from "../types";

export default function Settings() {
  const { data } = useQuery<AppInfo>({
    queryKey: ["appInfo"],
    queryFn: () => api.appInfo(),
  });

  return (
    <>
      <h1 className="page-title">Paramètres</h1>
      <p className="page-sub">Informations sur l'application.</p>

      <div className="card">
        <div className="row" style={{ justifyContent: "space-between" }}>
          <span>Version</span>
          <span className="badge">{data?.version ?? "—"}</span>
        </div>
        <div className="row" style={{ justifyContent: "space-between", marginTop: 10 }}>
          <span>Base de données</span>
          <span style={{ fontFamily: "monospace", fontSize: 12, color: "var(--text-dim)" }}>{data?.db_path ?? "—"}</span>
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>À propos</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13, lineHeight: 1.6 }}>
          RETROVERSE est un compagnon universel pour les collections de jeux vidéo.
          Fonctionne localement, sans compte obligatoire, sans cloud obligatoire.
        </p>
        <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
          Licence GPL-3.0-or-later · Open Source
        </p>
      </div>
    </>
  );
}