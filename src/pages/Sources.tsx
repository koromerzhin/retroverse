import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import type { GameSource } from "../types";

export default function Sources() {
  const qc = useQueryClient();
  const { data: sources, isLoading } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });

  // local
  const [localName, setLocalName] = useState("");
  const [localPath, setLocalPath] = useState("");

  // remote
  const [rName, setRName] = useState("");
  const [rUrl, setRUrl] = useState("");
  const [rHost, setRHost] = useState("");
  const [rPort, setRPort] = useState("22");
  const [rUser, setRUser] = useState("deck");
  const [rKey, setRKey] = useState("/home/koromerzhin/.ssh/id_ed25519");
  const [rPath, setRPath] = useState("/run/media/deck/WDC1/Emulation/");

  function parseUrl(v: string) {
    setRUrl(v);
    // sftp://user@host:port/path
    if (!v.toLowerCase().startsWith("sftp://")) return;
    let rest = v.slice(7);
    let user = "";
    if (rest.includes("@")) {
      const idx = rest.indexOf("@");
      user = rest.slice(0, idx);
      rest = rest.slice(idx + 1);
    }
    let host = rest;
    let port = "";
    let path = "";
    const slashIdx = rest.indexOf("/");
    if (slashIdx >= 0) {
      path = rest.slice(slashIdx);
      host = rest.slice(0, slashIdx);
    }
    if (host.includes(":")) {
      const ci = host.indexOf(":");
      port = host.slice(ci + 1);
      host = host.slice(0, ci);
    }
    if (user) setRUser(user);
    if (host) setRHost(host);
    if (port) setRPort(port);
    if (path) setRPath(path);
    if (host && !rName) setRName(host);
  }
  const [testMsg, setTestMsg] = useState<string | null>(null);

  const [error, setError] = useState<string | null>(null);

  const addLocal = useMutation({
    mutationFn: () => api.addSource(localName.trim() || "Nouvelle source", localPath),
    onSuccess: () => {
      setLocalName(""); setLocalPath("");
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
    },
    onError: (e: any) => setError(String(e)),
  });

  const addRemote = useMutation({
    mutationFn: () =>
      api.addRemoteSource(rName.trim() || "Source distante", rHost.trim(), Number(rPort) || 22, rUser.trim(), rKey.trim(), rPath.trim()),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const removeMutation = useMutation({
    mutationFn: (id: string) => api.removeSource(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["stats"] });
    },
  });

  async function handlePickLocal() {
    const dir = await pickDirectory();
    if (dir) {
      setLocalPath(dir);
      if (!localName) setLocalName(dir.split("/").pop() || "Source");
    }
  }
  async function handlePickKey() {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const res = await open({ multiple: false });
    if (typeof res === "string") setRKey(res);
  }

  async function handleTest() {
    setError(null); setTestMsg(null);
    try {
      const msg = await api.testRemoteConnection(rHost.trim(), Number(rPort) || 22, rUser.trim(), rKey.trim());
      setTestMsg(msg);
    } catch (e: any) {
      setError(String(e));
    }
  }

  return (
    <>
      <h1 className="page-title">Sources</h1>
      <p className="page-sub">Gérez les bibliothèques locales et distantes. Le scan se fait par dossier (voir ci-dessous).</p>

      <div className="card" style={{ marginBottom: 20 }}>
        <h3 style={{ marginTop: 0 }}>Ajouter une source locale</h3>
        <div className="row">
          <input className="input" placeholder="Nom" value={localName} onChange={(e) => setLocalName(e.target.value)} style={{ maxWidth: 220 }} />
          <input className="input" placeholder="Chemin du dossier" value={localPath} onChange={(e) => setLocalPath(e.target.value)} />
          <button className="btn" onClick={handlePickLocal}>Parcourir…</button>
          <button className="btn primary" onClick={() => addLocal.mutate()} disabled={!localPath || addLocal.isPending}>
            {addLocal.isPending ? <span className="spinner" /> : "Ajouter"}
          </button>
        </div>
      </div>

      <div className="card" style={{ marginBottom: 20 }}>
        <h3 style={{ marginTop: 0 }}>Ajouter une source distante (SSH/SFTP)</h3>
        <div className="row">
          <input className="input" placeholder="URL sftp://user@host:port/chemin (optionnel)" value={rUrl} onChange={(e) => parseUrl(e.target.value)} />
        </div>
        <div className="row" style={{ marginTop: 10 }}>
          <input className="input" placeholder="Nom" value={rName} onChange={(e) => setRName(e.target.value)} style={{ maxWidth: 200 }} />
          <input className="input" placeholder="Hôte (IP ou nom)" value={rHost} onChange={(e) => setRHost(e.target.value)} style={{ maxWidth: 200 }} />
          <input className="input" placeholder="Port" value={rPort} onChange={(e) => setRPort(e.target.value)} style={{ maxWidth: 80 }} />
          <input className="input" placeholder="Utilisateur" value={rUser} onChange={(e) => setRUser(e.target.value)} style={{ maxWidth: 120 }} />
        </div>
        <div className="row" style={{ marginTop: 10 }}>
          <input className="input" placeholder="Chemin de la clé SSH privée" value={rKey} onChange={(e) => setRKey(e.target.value)} />
          <button className="btn" onClick={handlePickKey}>Parcourir…</button>
        </div>
        <div className="row" style={{ marginTop: 10 }}>
          <input className="input" placeholder="Chemin distant des ROMs" value={rPath} onChange={(e) => setRPath(e.target.value)} />
        </div>
        <div className="row" style={{ marginTop: 12 }}>
          <button className="btn" onClick={handleTest} disabled={!rHost || !rKey}>Tester la connexion</button>
          <button className="btn primary" onClick={() => addRemote.mutate()} disabled={!rHost || !rKey || !rPath || addRemote.isPending}>
            {addRemote.isPending ? <span className="spinner" /> : "Ajouter la source distante"}
          </button>
          {testMsg && <span style={{ color: "var(--success)", fontSize: 13 }}>{testMsg}</span>}
        </div>
        <p style={{ fontSize: 12, color: "var(--text-muted)", marginTop: 8 }}>
          Exemple Steam Deck : hôte 192.168.x.x, utilisateur <code>deck</code>, clé <code>~/.ssh/id_ed25519</code>, chemin <code>/home/deck/Emulation/roms/</code>.
        </p>
      </div>

      {error && <p style={{ color: "var(--danger)", marginBottom: 12 }}>{error}</p>}

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : (sources?.length ?? 0) === 0 ? (
        <div className="empty">
          <div className="icon">📁</div>
          <p>Aucune source configurée.</p>
        </div>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Type</th>
              <th>Chemin</th>
              <th>Dernier scan</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sources!.map((s) => (
              <tr key={s.id}>
                <td>{s.name}</td>
                <td><span className="badge">{s.kind === "remote" ? "Distant" : "Local"}</span></td>
                <td style={{ fontFamily: "monospace", fontSize: 12 }}>{s.kind === "remote" ? `${s.remote_path}` : s.path}</td>
                <td>{s.last_scan_at ? new Date(s.last_scan_at).toLocaleString() : "—"}</td>
                <td>
                  <div className="row">
                    <button
                      className="btn danger"
                      onClick={() => { if (confirm("Supprimer cette source ?")) removeMutation.mutate(s.id); }}
                    >
                      Supprimer
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}