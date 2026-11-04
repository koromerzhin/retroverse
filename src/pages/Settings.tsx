import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n, LANGS } from "../i18n";
import { useTheme } from "../lib/theme";
import type { AppInfo } from "../types";

export default function Settings() {
  const qc = useQueryClient();
  const { t, lang, setLang } = useI18n();
  const { theme, setTheme, resolved } = useTheme();
  const { data: provCfg } = useQuery({ queryKey: ["providerConfig"], queryFn: () => api.getProviderConfig() });
  const [pc, setPc] = useState<any>(null);
  useEffect(() => { if (provCfg) setPc(provCfg); }, [provCfg]);
  const { data } = useQuery<AppInfo>({
    queryKey: ["appInfo"],
    queryFn: () => api.appInfo(),
  });
  const [confirmAction, setConfirmAction] = useState<"clear" | "reset" | null>(null);

  async function handleReset() {
    setConfirmAction("reset");
  }

  async function handleClearLibrary() {
    setConfirmAction("clear");
  }

  async function confirmActionRun() {
    const action = confirmAction;
    setConfirmAction(null);
    if (!action) return;
    try {
      if (action === "reset") {
        await api.resetDatabase();
          api.showNotification("RETROVERSE", "Base de données vidée.").catch(() => {});
      } else {
        await api.clearLibrary();
          api.showNotification("RETROVERSE", "Bibliothèque vidée.").catch(() => {});
      }
      qc.invalidateQueries();
    } catch (e: any) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
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
      </div>

      {confirmAction && (
        <div className="modal-backdrop" onClick={() => setConfirmAction(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 style={{ marginTop: 0 }}>Confirmation</h3>
            <p style={{ color: "var(--text-dim)", fontSize: 13 }}>
              {confirmAction === "reset"
                ? "Vider toute la base de données ? Cette action est irréversible."
                : "Vider la bibliothèque (jeux, fichiers, métadonnées) ? Les sources sont conservées."}
            </p>
            <div className="row" style={{ marginTop: 16, justifyContent: "flex-end", gap: 8 }}>
              <button className="btn" onClick={() => setConfirmAction(null)}>Annuler</button>
              <button className="btn danger" onClick={confirmActionRun}>Confirmer</button>
            </div>
          </div>
        </div>
      )}

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Thème</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>Choisir le thème de l'interface.</p>
        <select className="select" style={{ maxWidth: 260 }} value={theme} onChange={(e) => setTheme(e.target.value as any)}>
          <option value="system">Système</option>
          <option value="dark">Sombre</option>
          <option value="light">Clair</option>
        </select>
        <p style={{ color: "var(--text-muted)", fontSize: 12, marginTop: 8 }}>Actuel : {resolved}</p>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("settings.language")}</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>{t("settings.language.desc")}</p>
        <select className="select" style={{ maxWidth: 260 }} value={lang} onChange={(e) => setLang(e.target.value as any)}>
          {LANGS.map((l) => <option key={l.code} value={l.code}>{l.label}</option>)}
        </select>
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Fournisseurs de métadonnées</h3>
        <p style={{ color: "var(--text-dim)", fontSize: 13 }}>Configurez les clés API pour ScreenScraper.</p>
        {pc && (
          <>
            <div style={{ marginTop: 8 }}>
              <label style={{ fontSize: 12, color: "var(--text-muted)" }}>ScreenScraper dev id</label>
              <input className="input" value={pc.screenscraper_devid || ""} onChange={(e) => setPc({ ...pc, screenscraper_devid: e.target.value })} />
            </div>
            <div style={{ marginTop: 8 }}>
              <label style={{ fontSize: 12, color: "var(--text-muted)" }}>ScreenScraper dev password</label>
              <input className="input" type="password" value={pc.screenscraper_devpassword || ""} onChange={(e) => setPc({ ...pc, screenscraper_devpassword: e.target.value })} />
            </div>
            <div style={{ marginTop: 8 }}>
              <label style={{ fontSize: 12, color: "var(--text-muted)" }}>ScreenScraper user id (ssid)</label>
              <input className="input" value={pc.screenscraper_ssid || ""} onChange={(e) => setPc({ ...pc, screenscraper_ssid: e.target.value })} />
            </div>
            <div style={{ marginTop: 8 }}>
              <label style={{ fontSize: 12, color: "var(--text-muted)" }}>ScreenScraper user password</label>
              <input className="input" type="password" value={pc.screenscraper_sspassword || ""} onChange={(e) => setPc({ ...pc, screenscraper_sspassword: e.target.value })} />
            </div>
            <button className="btn primary" style={{ marginTop: 12 }} onClick={async () => {
              try {
                await api.setProviderConfig(pc);
                qc.invalidateQueries({ queryKey: ["providerConfig"] });
                api.showNotification("RETROVERSE", "Configuration enregistrée.").catch(() => {});
              } catch (e: any) {
                api.showNotification("RETROVERSE", String(e)).catch(() => {});
              }
            }}>Enregistrer</button>
          </>
        )}
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