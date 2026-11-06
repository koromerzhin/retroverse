import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import { useI18n } from "../i18n";
import type { GameSource } from "../types";

export default function Sources() {
  const qc = useQueryClient();
  const { t } = useI18n();
  const { data: sources, isLoading } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });
  const { data: active } = useQuery<GameSource | null>({ queryKey: ["active-source"], queryFn: () => api.getActiveSource() });

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

  // edit state
  const [editing, setEditing] = useState<GameSource | null>(null);
  const [editName, setEditName] = useState("");
  const [editPath, setEditPath] = useState("");
  const [editHost, setEditHost] = useState("");
  const [editPort, setEditPort] = useState("22");
  const [editUser, setEditUser] = useState("");
  const [editKey, setEditKey] = useState("");
  const [editRPath, setEditRPath] = useState("");

  const [testMsg, setTestMsg] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function parseUrl(v: string) {
    setRUrl(v);
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

  const addLocal = useMutation({
    mutationFn: () => api.addSource(localName.trim() || "Nouvelle source", localPath),
    onSuccess: () => {
      setLocalName(""); setLocalPath("");
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
    },
    onError: (e: any) => setError(String(e)),
  });

  const addRemote = useMutation({
    mutationFn: () =>
      api.addRemoteSource(rName.trim() || "Source distante", rHost.trim(), Number(rPort) || 22, rUser.trim(), rKey.trim(), rPath.trim()),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
      setError(null);
    },
    onError: (e: any) => setError(String(e)),
  });

  const removeMutation = useMutation({
    mutationFn: (id: string) => api.removeSource(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
    },
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => api.setActiveSource(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
      api.showNotification("RETROVERSE", "Source activée.").catch(() => {});
    },
    onError: (e: any) => api.showNotification("RETROVERSE", String(e)).catch(() => {}),
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
    // test with form values (not saved) — use a temp approach: save then test? Simpler: test saved source only.
    setTestMsg("Test disponible après ajout de la source.");
  }
  async function handleTestSource(id: string) {
    try {
      const msg = await api.testSource(id);
      api.showNotification("RETROVERSE", msg).catch(() => {});
    } catch (e: any) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    }
  }

  async function startEdit(s: GameSource) {
    setEditing(s);
    setEditName(s.name);
    if (s.kind === "local") {
      setEditPath(s.path);
    } else {
      setEditRPath(s.remote_path || s.path);
      try {
        const c = await api.getRemoteConnection(s.id);
        setEditHost(c.host || "");
        setEditPort(String(c.port || 22));
        setEditUser(c.username || "");
        setEditKey(c.key_path || "");
      } catch {
        setEditHost(""); setEditPort("22"); setEditUser(""); setEditKey("");
      }
    }
  }

  async function saveEdit() {
    if (!editing) return;
    try {
      if (editing.kind === "local") {
        await api.updateSource(editing.id, editName || editing.name, editPath || editing.path);
      } else {
        // need host/user/key; if empty, fetch from db? fallback: keep existing by re-entering
        await api.updateRemoteSource(editing.id, editName || editing.name, editHost, Number(editPort) || 22, editUser, editKey, editRPath || editing.path);
      }
      qc.invalidateQueries({ queryKey: ["sources"] });
      setEditing(null);
      api.showNotification("RETROVERSE", "Source mise à jour.").catch(() => {});
    } catch (e: any) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    }
  }

  return (
    <>
      <h1 className="page-title">{t("sources.title")}</h1>
      <p className="page-sub">{t("sources.subtitle")}</p>

      {active && (
        <div className="card" style={{ marginBottom: 16, background: "var(--bg-3)" }}>
          <span style={{ fontSize: 13, color: "var(--text-dim)" }}>Source active : </span>
          <strong>{active.name}</strong>
          <span className="badge" style={{ marginLeft: 8 }}>{active.kind === "remote" ? "Distant" : "Local"}</span>
          <span style={{ marginLeft: 8, fontFamily: "monospace", fontSize: 12, color: "var(--text-muted)" }}>{active.kind === "remote" ? active.remote_path : active.path}</span>
        </div>
      )}

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
              <th>Active</th>
              <th>Nom</th>
              <th>Type</th>
              <th>Chemin</th>
              <th>Dernier scan</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sources!.map((s) => (
              <tr key={s.id} style={{ background: s.id === active?.id ? "var(--bg-3)" : undefined }}>
                <td>
                  {s.id === active?.id ? (
                    <span style={{ color: "var(--success)", fontWeight: 700 }}>✓</span>
                  ) : (
                    <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => activateMutation.mutate(s.id)}>Activer</button>
                  )}
                </td>
                <td>{s.name}</td>
                <td><span className="badge">{s.kind === "remote" ? "Distant" : "Local"}</span></td>
                <td style={{ fontFamily: "monospace", fontSize: 12 }}>{s.kind === "remote" ? `${s.remote_path}` : s.path}</td>
                <td>{s.last_scan_at ? new Date(s.last_scan_at).toLocaleString() : "—"}</td>
                <td>
                  <div className="row" style={{ gap: 6 }}>
                    <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => handleTestSource(s.id)}>Tester</button>
                    <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => startEdit(s)}>Modifier</button>
                    <button
                      className="btn danger"
                      style={{ fontSize: 11, padding: "2px 8px" }}
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

      {editing && (
        <div className="modal-backdrop" onClick={() => setEditing(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3 style={{ marginTop: 0 }}>Modifier la source</h3>
            <div style={{ marginTop: 8 }}>
              <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Nom</label>
              <input className="input" value={editName} onChange={(e) => setEditName(e.target.value)} />
            </div>
            {editing.kind === "local" ? (
              <>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Chemin</label>
                  <input className="input" value={editPath} onChange={(e) => setEditPath(e.target.value)} />
                </div>
              </>
            ) : (
              <>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Hôte</label>
                  <input className="input" value={editHost} onChange={(e) => setEditHost(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Port</label>
                  <input className="input" value={editPort} onChange={(e) => setEditPort(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Utilisateur</label>
                  <input className="input" value={editUser} onChange={(e) => setEditUser(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Clé SSH</label>
                  <input className="input" value={editKey} onChange={(e) => setEditKey(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Chemin distant</label>
                  <input className="input" value={editRPath} onChange={(e) => setEditRPath(e.target.value)} />
                </div>
              </>
            )}
            <div className="row" style={{ marginTop: 16, justifyContent: "flex-end", gap: 8 }}>
              <button className="btn" onClick={() => setEditing(null)}>Annuler</button>
              <button className="btn primary" onClick={saveEdit}>Enregistrer</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
