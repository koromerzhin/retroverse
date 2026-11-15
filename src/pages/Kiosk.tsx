import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { api } from "../lib/api";

function coverOf(g: any): string | null {
  try {
    const d = g.game?.metadata_json ? JSON.parse(g.game.metadata_json) : null;
    if (!d) return null;
    if (d.cover_url) return d.cover_url;
    if (Array.isArray(d.medias)) {
      const m = d.medias.find((x: any) => x?.type === "box-2D" || x?.type === "box-3D" || x?.type === "cover" || x?.type === "screenshot");
      if (m?.url) return m.url;
    }
    return null;
  } catch { return null; }
}

export default function Kiosk() {
  const navigate = useNavigate();
  const [games, setGames] = useState<any[]>([]);
  const [idx, setIdx] = useState(0);
  const [playing, setPlaying] = useState(true);

  useEffect(() => {
    (async () => {
      try {
        const r = await api.listGames(undefined, undefined, undefined, undefined, false, 500, 0);
        const items = r?.items ?? r ?? [];
        setGames(items);
      } catch { setGames([]); }
    })();
  }, []);

  useEffect(() => {
    if (!playing || games.length === 0) return;
    const id = setInterval(() => setIdx((i) => (i + 1) % games.length), 5000);
    return () => clearInterval(id);
  }, [playing, games.length]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") navigate("/");
      if (e.key === " " ) { e.preventDefault(); setPlaying((p) => !p); }
      if (e.key === "ArrowRight") setIdx((i) => (i + 1) % Math.max(1, games.length));
      if (e.key === "ArrowLeft") setIdx((i) => (i - 1 + games.length) % Math.max(1, games.length));
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [games.length, navigate]);

  const g = games[idx];
  const cover = g ? coverOf(g) : null;
  const title = g?.game?.title ?? "";
  const platform = g?.platform?.name ?? "";

  return (
    <div style={{ position: "fixed", inset: 0, background: "#000", color: "#fff", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", zIndex: 1000 }}>
      <button onClick={() => navigate("/")} style={{ position: "absolute", top: 12, right: 12, background: "rgba(255,255,255,.15)", color: "#fff", border: "none", borderRadius: 6, padding: "6px 12px", cursor: "pointer" }}>✕ Quitter</button>
      {games.length === 0 ? (
        <div style={{ opacity: .6 }}>Chargement…</div>
      ) : (
        <>
          <div style={{ width: "60vw", maxWidth: 600, aspectRatio: "3/4", display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 24 }}>
            {cover ? <img src={cover} alt={title} style={{ maxWidth: "100%", maxHeight: "100%", objectFit: "contain", borderRadius: 12, boxShadow: "0 10px 40px rgba(0,0,0,.6)" }} /> : <div style={{ fontSize: 120 }}>🎮</div>}
          </div>
          <h1 style={{ fontSize: 32, margin: 0, textAlign: "center" }}>{title}</h1>
          <p style={{ opacity: .7, marginTop: 8 }}>{platform}</p>
          <div style={{ position: "absolute", bottom: 16, fontSize: 12, opacity: .5 }}>
            {idx + 1} / {games.length} · {playing ? "lecture" : "pause"} · Espace: pause · ←/→: naviguer · Échap: quitter
          </div>
        </>
      )}
    </div>
  );
}
