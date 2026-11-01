import { useEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/api";
import { formatNumber } from "./lib/format";
import { NavLink, Routes, Route, Navigate } from "react-router-dom";
import Home from "./pages/Home";
import Library from "./pages/Library";
import GameDetail from "./pages/GameDetail";
import Platforms from "./pages/Platforms";
import Sources from "./pages/Sources";
import Companion from "./pages/Companion";
import Discovery from "./pages/Discovery";
import Preservation from "./pages/Preservation";
import Settings from "./pages/Settings";

const nav = [
  { to: "/", label: "Accueil", icon: "🏠" },
  { to: "/platforms", label: "Plateformes", icon: "🎮" },
  { to: "/sources", label: "Sources", icon: "📁" },
  { to: "/companion", label: "Companion", icon: "🕹️" },
  { to: "/discovery", label: "Découverte", icon: "🧭" },
  { to: "/preservation", label: "Préservation", icon: "🛡️" },
  { to: "/settings", label: "Paramètres", icon: "⚙️" },
];

export default function App() {
  const qc = useQueryClient();
  const [scanProgress, setScanProgress] = useState<{ scanned: number; total: number; status: string; sourceId?: string; file?: string } | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [scanDetails, setScanDetails] = useState(false);
  const [sources, setSources] = useState<any[]>([]);
  const lastProgressUpdate = useRef(0);

  useEffect(() => {
    api.listSources().then((s) => setSources(s || [])).catch(() => {});
  }, []);

  // Preload data + hide static splash
  useEffect(() => {
    const hide = () => {
      const el = document.getElementById("splash");
      if (el) {
        el.style.transition = "opacity .4s";
        el.style.opacity = "0";
        setTimeout(() => el.remove(), 400);
      }
    };
    const timer = setTimeout(hide, 4000);
    (async () => {
      try {
        // Prefetch into React Query cache for instant page rendering
        const [platforms, sources, games, stats] = await Promise.all([
          api.listPlatforms().catch(() => []),
          api.listSources().catch(() => []),
          api.listGames().catch(() => []),
          api.stats().catch(() => null),
        ]);
        qc.setQueryData(["platforms"], platforms);
        qc.setQueryData(["sources"], sources);
        qc.setQueryData(["games"], games);
        if (stats) qc.setQueryData(["stats"], stats);
        setTimeout(hide, 500);
      } catch {
        setTimeout(hide, 1200);
      }
    })();
    return () => clearTimeout(timer);
  }, [qc]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<any>("scan-progress", (ev) => {
      const now = Date.now();
      if (ev.payload?.status !== "done" && ev.payload?.status !== "cancelled" && now - lastProgressUpdate.current < 200) {
        return;
      }
      lastProgressUpdate.current = now;
      if (ev.payload?.status === "cancelled") {
        setScanProgress(null);
        setToast("Scan annulé");
        setTimeout(() => setToast(null), 4000);
      } else if (ev.payload?.status === "done") {
        setScanProgress(null);
        const created = ev.payload?.games_created ?? 0;
        const updated = ev.payload?.games_updated ?? 0;
        const files = ev.payload?.files_found ?? 0;
        let msg = "Scan terminé";
        if (created > 0 || updated > 0) {
          msg += ` : ${formatNumber(created)} nouveau(x) jeu(x), ${formatNumber(updated)} mis à jour (${formatNumber(files)} fichier(s))`;
        } else {
          msg += ` : ${formatNumber(files)} fichier(s), aucune nouveauté`;
        }
        setToast(msg);
        setTimeout(() => setToast(null), 5000);
      } else {
        setScanProgress({ scanned: ev.payload?.scanned ?? 0, total: ev.payload?.total ?? 0, status: ev.payload?.status ?? "scanning", sourceId: ev.payload?.source_id, file: ev.payload?.file });
      }
    }).then((u) => { unlisten = u; });
    return () => { if (unlisten) unlisten(); };
  }, []);

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
          <span>Scan en cours… {formatNumber(scanProgress.scanned)}{scanProgress.total > 0 ? ` / ${formatNumber(scanProgress.total)}` : ""} fichier(s)</span>
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
              Progression : {formatNumber(scanProgress.scanned)} / {formatNumber(scanProgress.total || 0)} fichier(s)
            </p>
            {scanProgress.file && (
              <p style={{ fontSize: 12, color: "var(--text-muted)", wordBreak: "break-all", fontFamily: "monospace" }}>
                Fichier en cours : {scanProgress.file}
              </p>
            )}
            <p style={{ fontSize: 13, color: "var(--text-dim)" }}>
              Statut : {scanProgress.status}
            </p>
            <div className="row" style={{ marginTop: 16, justifyContent: "flex-end" }}>
              <button className="btn" onClick={() => setScanDetails(false)}>Fermer</button>
              <button className="btn danger" onClick={async () => { try { await api.cancelScan(); } catch {} setScanDetails(false); }}>Arrêter</button>
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
      </main>
    </div>
  );
}