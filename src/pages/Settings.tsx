import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n, LANGS } from "../i18n";
import type { AppInfo } from "../types";

export default function Settings() {
  const qc = useQueryClient();
  const { t, lang, setLang } = useI18n();
  const { data } = useQuery<AppInfo>({
    queryKey: ["appInfo"],
    queryFn: () => api.appInfo(),
  });
  const [msg, setMsg] = useState<string | null>(null);

  async function handleReset() {
    if (!confirm("Vider toute la base de données ? Cette action est irréversible.")) return;
    try {
      await api.resetDatabase();
      setMsg("Base de données vidée.");
      qc.invalidateQueries();
    } catch (e: any) {
      setMsg(String(e));
    }
  }

  async function handleClearLibrary() {
    if (!confirm("Vider la bibliothèque (jeux, fichiers, métadonnées) ? Les sources sont conservées.")) return;
    try {
      await api.clearLibrary();
      setMsg("Bibliothèque vidée.");
      qc.invalidateQueries();
    } catch (e: any) {
      setMsg(String(e));
    }
  }

  return (
    <>
      <h1 className="page-title">{t("settings.title")}</h1>
      <p className="page-sub">{t("settings.subtitle")}</p>

      <div className="card">
        <div className="row" style={{ justifyContent: "space-between" }}>
          <span>{t("settings.version")}</span>
          <span className="badge">{data?.version ?? "—"}</span>
        </div>
        <div className="row" style={{ justifyContent: "space-between", marginTop: 10 }}>
          <span>{t("settings.dbPath")}</span>
          <span style={{ fontFamily: "monospace", fontSize: 12, color: "var(--text-dim)" }}>{data?.db_path ?? "—"}</span>
        </div>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("settings.maintenance")}</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>
          {t("settings.maintenance.desc")}
        </p>
        <button className="btn danger" onClick={handleClearLibrary}>{t("settings.clearLibrary")}</button>
        <button className="btn danger" onClick={handleReset} style={{ marginLeft: 8 }}>{t("settings.clearDb")}</button>
        {msg && <p style={{ color: "var(--text-dim)", marginTop: 8, fontSize: 13 }}>{msg}</p>}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("settings.language")}</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>{t("settings.language.desc")}</p>
        <select className="select" style={{ maxWidth: 260 }} value={lang} onChange={(e) => setLang(e.target.value as any)}>
          {LANGS.map((l) => <option key={l.code} value={l.code}>{l.label}</option>)}
        </select>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("settings.about")}</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13, lineHeight: 1.6 }}>
          {t("settings.about.text")}
        </p>
        <p style={{ color: "var(--text-muted)", fontSize: 12 }}>
          {t("settings.license")}
        </p>
      </div>
    </>
  );
}