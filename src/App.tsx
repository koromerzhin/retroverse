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
  return (
    <div className="app">
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