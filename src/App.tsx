import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/api";
import { formatNumber } from "./lib/format";
import { useI18n } from "./i18n";
import { NavLink, Routes, Route, Navigate } from "react-router-dom";
const Home = lazy(() => import("./pages/Home"));
const Library = lazy(() => import("./pages/Library"));
const GameDetail = lazy(() => import("./pages/GameDetail"));
const Platforms = lazy(() => import("./pages/Platforms"));
const Sources = lazy(() => import("./pages/Sources"));
const Companion = lazy(() => import("./pages/Companion"));
const Discovery = lazy(() => import("./pages/Discovery"));
const Preservation = lazy(() => import("./pages/Preservation"));
const Settings = lazy(() => import("./pages/Settings"));

const nav = [
  { to: "/", label: "Accueil", icon: "🏠" },
  { to: "/library", label: "Bibliothèque", icon: "📚" },
  { to: "/platforms", label: "Plateformes", icon: "🎮" },
  { to: "/sources", label: "Sources", icon: "📁" },
  { to: "/companion", label: "Companion", icon: "🕹️" },
  { to: "/discovery", label: "Découverte", icon: "🧭" },
  { to: "/preservation", label: "Préservation", icon: "🛡️" },
  { to: "/settings", label: "Paramètres", icon: "⚙️" },
];

export default function App() {
  const qc = useQueryClient();
  const [scanProgress, setScanProgress] = useState<{ scanned: number; total: number; status: string; sourceId?: string; file?: string; folder?: string } | null>(null);
  const [hashProgress, setHashProgress] = useState<{ scanned: number; total: number; status: string; file?: string; done?: number } | null>(null);
  const [hashDetails, setHashDetails] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const { t } = useI18n();
  const notifyToast = (msg: string) => {
    setToast(msg);
    api.showNotification("RETROVERSE", msg).catch(() => {});
  };
  const [scanDetails, setScanDetails] = useState(false);
  const [sources, setSources] = useState<any[]>([]);
  const lastProgressUpdate = useRef(0);

  useEffect(() => {
    api.listSources().then((s) => setSources(s || [])).catch(() => {});
  }, []);

  // Preload data, then ask Rust to reveal the main window and close the splashscreen.
  useEffect(() => {
    (async () => {
      try {
        const [platforms, sources, stats] = await Promise.all([
          api.listPlatforms().catch(() => []),
          api.listSources().catch(() => []),
          api.stats().catch(() => null),
        ]);
        qc.setQueryData(["platforms"], platforms);
        qc.setQueryData(["sources"], sources);
        if (stats) qc.setQueryData(["stats"], stats);
      } catch {
        // Non-fatal: the app can still start.
      } finally {
        try { await api.appReady(); } catch { /* ignore */ }
      }
    })();
  }, [qc]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<any>("hash-progress", (ev) => {
      if (ev.payload?.status === "cancelled") {
        setHashProgress(null);
        notifyToast("Calcul annulé");
        setTimeout(() => setToast(null), 4000);
      } else if (ev.payload?.status === "done") {
        setHashProgress(null);
        notifyToast(`Empreintes calculées : ${ev.payload?.done ?? 0} fichier(s)`);
        setTimeout(() => setToast(null), 5000);
      } else {
        setHashProgress({ scanned: ev.payload?.scanned ?? 0, total: ev.payload?.total ?? 0, status: ev.payload?.status ?? "scanning", file: ev.payload?.file, done: ev.payload?.done });
      }
    }).then((u) => { unlisten = u; });
    return () => { if (unlisten) unlisten(); };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<any>("scan-progress", (ev) => {
      const now = Date.now();
      if (ev.payload?.status !== "done" && ev.payload?.status !== "cancelled" && now - lastProgressUpdate.current < 200) {
        return;
      }
      lastProgressUpdate.current = now;
      const scanAll = (window as any).__retroverseScanAll;
      const scanPlatform = (window as any).__retroverseScanningPlatform;
      if ((scanAll || scanPlatform) && (ev.payload?.status === "done" || ev.payload?.status === "cancelled" || ev.payload?.status === "skipped")) {
        qc.invalidateQueries();
        qc.refetchQueries();
        setScanProgress(null);
        return;
      }
      if (ev.payload?.status === "cancelled") {
        setScanProgress(null);
        notifyToast("Scan annulé");
        setTimeout(() => setToast(null), 4000);
      } else if (ev.payload?.status === "skipped") {
        setScanProgress(null);
        notifyToast("Source inaccessible — scan ignoré");
        setTimeout(() => setToast(null), 4000);
      } else if (ev.payload?.status === "done") {
        setScanProgress((prev) => ({ scanned: ev.payload?.scanned ?? 0, total: ev.payload?.total ?? 0, status: "done", sourceId: ev.payload?.source_id ?? prev?.sourceId, file: ev.payload?.file ?? prev?.file, folder: ev.payload?.folder ?? prev?.folder }));
        const created = ev.payload?.games_created ?? 0;
        const updated = ev.payload?.games_updated ?? 0;
        const files = ev.payload?.files_found ?? 0;
        let msg = "Scan terminé";
        if (created > 0 || updated > 0) {
          msg += ` : ${formatNumber(created)} nouveau(x) jeu(x), ${formatNumber(updated)} mis à jour (${formatNumber(files)} fichier(s))`;
        } else {
          msg += ` : ${formatNumber(files)} fichier(s), aucune nouveauté`;
        }
        notifyToast(msg);
        qc.invalidateQueries();
        qc.refetchQueries();
        setScanProgress(null);
        setTimeout(() => setToast(null), 5000);
      } else {
        setScanProgress((prev) => ({
          scanned: ev.payload?.scanned ?? 0,
          total: ev.payload?.total ?? 0,
          status: ev.payload?.status ?? "scanning",
          sourceId: ev.payload?.source_id ?? prev?.sourceId,
          file: ev.payload?.file ?? prev?.file,
          folder: ev.payload?.folder ?? prev?.folder,
        }));
      }
    }).then((u) => { unlisten = u; });
    return () => { if (unlisten) unlisten(); };
  }, []);


  const statusLabel = (s: string) => {
    switch (s) {
      case "scanning": return t("scan.status.scanning");
      case "folder_done": return t("scan.status.folderDone");
      case "done": return t("scan.status.done");
      case "cancelled": return t("scan.status.cancelled");
      case "skipped": return t("scan.status.skipped");
      default: return s;
    }
  };

  // No auto-scan on startup (user request).

  return (
    <div className="app">
      {scanProgress && (
        <div
          onClick={() => setScanDetails((v) => !v)}
          style={{
            position: "fixed", top: 0, left: 240, right: 0, zIndex: 200,
            background: "var(--bg-2)", borderBottom: "1px solid var(--border)",
            padding: "6px 16px", fontSize: 12, color: "var(--text-dim)",
            display: "flex", alignItems: "center", gap: 12,
            cursor: "pointer",
          }}
        >
          <span>Scan en cours… {formatNumber(scanProgress.scanned)}{scanProgress.total > 0 ? ` / ${formatNumber(scanProgress.total)}` : ""} fichier(s){scanProgress.total > 0 ? ` (${Math.round((scanProgress.scanned / scanProgress.total) * 100)}%)` : ""}</span>
          <div style={{ flex: 1, height: 4, background: "var(--bg-3)", borderRadius: 2, overflow: "hidden", maxWidth: 300 }}>
            <div style={{
              height: "100%",
              width: scanProgress.total > 0 ? `${Math.min(100, (scanProgress.scanned / scanProgress.total) * 100)}%` : "100%",
              background: "var(--accent)",
              transition: "width .2s",
            }} />
          </div>
          <span style={{ fontSize: 10, color: "var(--text-muted)" }}>(cliquer pour détails)</span>
          <button className="btn danger" style={{ fontSize: 11, padding: "2px 8px" }} onClick={async (e) => { e.stopPropagation(); try { await api.cancelScan(); } catch (err) { console.error(err); } }}>Arrêter</button>
        </div>
      )}
      {scanDetails && scanProgress && (
        <div className="modal-backdrop" onClick={() => setScanDetails(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 style={{ marginTop: 0 }}>Détails du scan</h3>
            <p style={{ fontSize: 13, color: "var(--text-dim)" }}>
              Source : {sources.find((s: any) => s.id === scanProgress.sourceId)?.name || scanProgress.sourceId || "—"}
            </p>
            <p style={{ fontSize: 13, color: "var(--text-dim)" }}>
              Progression : {formatNumber(scanProgress.scanned)} / {formatNumber(scanProgress.total || 0)} fichier(s){scanProgress.total ? ` (${Math.round((scanProgress.scanned / scanProgress.total) * 100)}%)` : ""}
            </p>
            {scanProgress.file && (
              <p style={{ fontSize: 12, color: "var(--text-muted)", wordBreak: "break-all", fontFamily: "monospace" }}>
                Fichier en cours : {scanProgress.file.substring(scanProgress.file.lastIndexOf("/") + 1)}
              </p>
            )}
            <p style={{ fontSize: 13, color: "var(--text-dim)" }}>
              Statut : {statusLabel(scanProgress.status)}
            </p>
            <div className="row" style={{ marginTop: 16, justifyContent: "flex-end" }}>
              <button className="btn" onClick={() => setScanDetails(false)}>Fermer</button>
              <button className="btn danger" onClick={async () => { try { await api.cancelScan(); } catch {} setScanDetails(false); }}>Arrêter</button>
            </div>
          </div>
        </div>
      )}
      {hashProgress && (
        <div
          onClick={() => setHashDetails((v) => !v)}
          style={{
            position: "fixed", top: scanProgress ? 40 : 0, left: 240, right: 0, zIndex: 199,
            background: "var(--bg-2)", borderBottom: "1px solid var(--border)",
            padding: "6px 16px", fontSize: 12, color: "var(--text-dim)",
            display: "flex", alignItems: "center", gap: 12,
            cursor: "pointer",
          }}
        >
          <span>Calcul des empreintes… {formatNumber(hashProgress.scanned)} / {formatNumber(hashProgress.total)}{hashProgress.total > 0 ? ` (${Math.round((hashProgress.scanned / hashProgress.total) * 100)}%)` : ""}</span>
          <div style={{ flex: 1, height: 4, background: "var(--bg-3)", borderRadius: 2, overflow: "hidden", maxWidth: 300 }}>
            <div style={{
              height: "100%",
              width: hashProgress.total > 0 ? `${Math.min(100, (hashProgress.scanned / hashProgress.total) * 100)}%` : "100%",
              background: "var(--accent-2)",
              transition: "width .2s",
            }} />
          </div>
          <span style={{ fontSize: 10, color: "var(--text-muted)" }}>(cliquer pour détails)</span>
          <button className="btn danger" style={{ fontSize: 11, padding: "2px 8px" }} onClick={(e) => { e.stopPropagation(); api.cancelScan().catch(() => {}); }}>Arrêter</button>
        </div>
      )}
      {hashDetails && hashProgress && (
        <div className="modal-backdrop" onClick={() => setHashDetails(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 style={{ marginTop: 0 }}>Détails du calcul des empreintes</h3>
            <p style={{ fontSize: 13, color: "var(--text-dim)" }}>
              Progression : {formatNumber(hashProgress.scanned)} / {formatNumber(hashProgress.total)}{hashProgress.total > 0 ? ` (${Math.round((hashProgress.scanned / hashProgress.total) * 100)}%)` : ""}
            </p>
            {hashProgress.file && (
              <p style={{ fontSize: 12, color: "var(--text-muted)", wordBreak: "break-all", fontFamily: "monospace" }}>
                Fichier en cours : {hashProgress.file.substring(hashProgress.file.lastIndexOf("/") + 1)}
              </p>
            )}
            <div className="row" style={{ marginTop: 16, justifyContent: "flex-end" }}>
              <button className="btn" onClick={() => setHashDetails(false)}>Fermer</button>
              <button className="btn danger" onClick={() => { api.cancelScan().catch(() => {}); setHashDetails(false); }}>Arrêter</button>
            </div>
          </div>
        </div>
      )}
      {toast && (
        <div className="toast" style={{ borderColor: "var(--success)" }}>{toast}</div>
      )}
      <aside className="sidebar">
        <div className="brand">
          <div className="logo">R</div>
          <div>
            RETROVERSE
            <small>Collection Companion</small>
          </div>
        </div>
        {nav.map((n) => (
          <NavLink
            key={n.to}
            to={n.to}
            end={n.to === "/"}
            className={({ isActive }) => "nav-item" + (isActive ? " active" : "")}
          >
            <span>{n.icon}</span>
            {n.label}
          </NavLink>
        ))}
      </aside>
      <main className="main">
        <Suspense fallback={<div className="empty"><div className="spinner" /></div>}>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/library" element={<Library />} />
          <Route path="/game/:id" element={<GameDetail />} />
          <Route path="/platforms" element={<Platforms />} />
          <Route path="/sources" element={<Sources />} />
          <Route path="/companion" element={<Companion />} />
          <Route path="/discovery" element={<Discovery />} />
          <Route path="/preservation" element={<Preservation />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
        </Suspense>
      </main>
    </div>
  );
}