import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
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
  { to: "/library", label: "Bibliothèque", icon: "📚" },
  { to: "/platforms", label: "Plateformes", icon: "🎮" },
  { to: "/sources", label: "Sources", icon: "📁" },
  { to: "/companion", label: "Companion", icon: "🕹️" },
  { to: "/discovery", label: "Découverte", icon: "🧭" },
  { to: "/preservation", label: "Préservation", icon: "🛡️" },
  { to: "/settings", label: "Paramètres", icon: "⚙️" },
];

export default function App() {
  const [scanProgress, setScanProgress] = useState<{ scanned: number; total: number; status: string } | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<any>("scan-progress", (ev) => {
      setScanProgress({ scanned: ev.payload?.scanned ?? 0, total: ev.payload?.total ?? 0, status: ev.payload?.status ?? "scanning" });
      if (ev.payload?.status === "done") {
        setTimeout(() => setScanProgress(null), 4000);
      }
    }).then((u) => { unlisten = u; });
    return () => { if (unlisten) unlisten(); };
  }, []);

  return (
    <div className="app">
      {scanProgress && (
        <div style={{
          position: "fixed", top: 0, left: 240, right: 0, zIndex: 200,
          background: "var(--bg-2)", borderBottom: "1px solid var(--border)",
          padding: "6px 16px", fontSize: 12, color: "var(--text-dim)",
          display: "flex", alignItems: "center", gap: 12,
        }}>
          <span>Scan en cours… {scanProgress.scanned}{scanProgress.total > 0 ? ` / ${scanProgress.total}` : ""} fichier(s)</span>
          <div style={{ flex: 1, height: 4, background: "var(--bg-3)", borderRadius: 2, overflow: "hidden", maxWidth: 300 }}>
            <div style={{
              height: "100%",
              width: scanProgress.total > 0 ? `${Math.min(100, (scanProgress.scanned / scanProgress.total) * 100)}%` : "100%",
              background: "var(--accent)",
              transition: "width .2s",
            }} />
          </div>
          {scanProgress.status === "done" && <span style={{ color: "var(--success)" }}>Terminé</span>}
        </div>
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