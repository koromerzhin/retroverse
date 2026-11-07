import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import type { GameSource } from "../types";

type SourceType = "local" | "remote";

export default function Sources() {
  const qc = useQueryClient();
  const { data: sources, isLoading } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });
  const { data: active } = useQuery<GameSource | null>({ queryKey: ["active-source"], queryFn: () => api.getActiveSource() });

  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogType, setDialogType] = useState<SourceType | null>(null);
  const [editing, setEditing] = useState<GameSource | null>(null);
  const [editKind, setEditKind] = useState<SourceType>("local");

  // Local form
  const [localName, setLocalName] = useState("");
  const [localPath, setLocalPath] = useState("");

  // Remote form
  const [rName, setRName] = useState("");
  const [rHost, setRHost] = useState("");
  const [rPort, setRPort] = useState("22");
  const [rUser, setRUser] = useState("deck");
  const [rKey, setRKey] = useState("/home/koromerzhin/.ssh/id_ed25519");
  const [rPath, setRPath] = useState("");

  // Edit form
  const [editName, setEditName] = useState("");
  const [editPath, setEditPath] = useState("");
  const [editHost, setEditHost] = useState("");
  const [editPort, setEditPort] = useState("22");
  const [editUser, setEditUser] = useState("");
  const [editKey, setEditKey] = useState("");
  const [editRPath, setEditRPath] = useState("");

  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function resetForms() {
    setLocalName(""); setLocalPath("");
    setRName(""); setRHost(""); setRPort("22"); setRUser("deck"); setRKey("/home/koromerzhin/.ssh/id_ed25519"); setRPath("");
    setEditName(""); setEditPath(""); setEditHost(""); setEditPort("22"); setEditUser(""); setEditKey(""); setEditRPath("");
    setError(null);
  }

  function openAdd() {
    setEditing(null);
    setDialogType(null);
    setEditKind("local");
    resetForms();
    setDialogOpen(true);
  }

  async function startEdit(src: GameSource) {
    setEditing(src);
    setEditKind(src.kind === "remote" ? "remote" : "local");
    setDialogType(src.kind === "remote" ? "remote" : "local");
    resetForms();
    setEditName(src.name);
    if (src.kind === "local") {
      setEditPath(src.path);
    } else {
      setEditRPath(src.remote_path || src.path);
      try {
        const c = await api.getRemoteConnection(src.id);
        setEditHost(c.host || "");
        setEditPort(String(c.port || 22));
        setEditUser(c.username || "");
        setEditKey(c.key_path || "");
      } catch {
        setEditHost(""); setEditPort("22"); setEditUser(""); setEditKey("");
      }
    }
    setDialogOpen(true);
  }

  async function handlePickLocal(target: "add" | "edit") {
    const dir = await pickDirectory();
    if (dir) {
      if (target === "add") { setLocalPath(dir); if (!localName) setLocalName(dir.split("/").pop() || "Source"); }
      else { setEditPath(dir); if (!editName) setEditName(dir.split("/").pop() || "Source"); }
    }
  }

  async function handlePickKey(target: "add" | "edit") {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const res = await open({ multiple: false });
    if (typeof res === "string") {
      if (target === "add") setRKey(res); else setEditKey(res);
    }
  }

  function editParseUrl(v: string) {
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
    if (user) setEditUser(user);
    if (host) setEditHost(host);
    if (port) setEditPort(port);
    if (path) setEditRPath(path);
    if (host && !editName) setEditName(host);
  }

  async function saveSource() {
    setSaving(true); setError(null);
    try {
      if (editing && editKind !== editing.kind) {
        // type conversion
        if (editKind === "local") {
          await api.convertSource(editing.id, "local", editName || editing.name, editPath || editing.path, undefined, undefined, undefined, undefined, undefined);
        } else {
          await api.convertSource(editing.id, "remote", editName || editing.name, undefined, editHost, Number(editPort) || 22, editUser, editKey, editRPath || editing.path);
        }
      } else if (editing) {
        if (editing.kind === "local") {
          await api.updateSource(editing.id, editName || editing.name, editPath || editing.path);
        } else {
          await api.updateRemoteSource(editing.id, editName || editing.name, editHost, Number(editPort) || 22, editUser, editKey, editRPath || editing.path);
        }
      } else {
        if (dialogType === "local") {
          await api.addSource(localName.trim() || "Source locale", localPath);
        } else {
          await api.addRemoteSource(rName.trim() || "Source distante", rHost.trim(), Number(rPort) || 22, rUser.trim(), rKey.trim(), rPath.trim());
        }
      }
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
      qc.refetchQueries({ queryKey: ["sources"] });
      setDialogOpen(false);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm("Supprimer cette source ?")) return;
    try {
      await api.removeSource(id);
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
      qc.refetchQueries({ queryKey: ["sources"] });
    } catch (e: any) {
      alert(String(e));
    }
  }

  async function handleActivate(id: string) {
    try {
      await api.setActiveSource(id);
      qc.invalidateQueries({ queryKey: ["sources"] });
      qc.invalidateQueries({ queryKey: ["active-source"] });
      qc.refetchQueries({ queryKey: ["sources"] });
    } catch (e: any) {
      alert(String(e));
    }
  }

  async function handleTest(id: string) {
    try {
      const msg = await api.testSource(id);
      alert(msg);
    } catch (e: any) {
      alert(String(e));
    }
  }

  const isEmpty = !sources || sources.length === 0;

  return (
    <>
      <h1 className="page-title">Sources</h1>
      <p className="page-sub">Gestion des sources de ROMs (locale et distante).</p>

      {active && (
        <div className="card" style={{ marginBottom: 16, background: "var(--bg-3)" }}>
          <span style={{ fontSize: 13, color: "var(--text-dim)" }}>Source active : </span>
          <strong>{active.name}</strong>
          <span className="badge" style={{ marginLeft: 8 }}>{active.kind === "remote" ? "Distant" : "Local"}</span>
          <span style={{ marginLeft: 8, fontFamily: "monospace", fontSize: 12, color: "var(--text-muted)" }}>{active.kind === "remote" ? active.remote_path : active.path}</span>
        </div>
      )}

      <div style={{ marginBottom: 16 }}>
        <button className="btn primary" onClick={openAdd}>Ajouter une source</button>
      </div>

      {isLoading ? (
        <div className="empty"><div className="spinner" /></div>
      ) : isEmpty ? (
        <div className="empty">
          <div className="icon">📁</div>
          <p style={{ fontSize: 14, fontWeight: 600 }}>Aucune source configurée</p>
          <p style={{ color: "var(--text-muted)", fontSize: 13 }}>Une source est nécessaire pour accéder aux ROMs.</p>
          <button className="btn primary" style={{ marginTop: 12 }} onClick={openAdd}>Ajouter une source</button>
        </div>
      ) : (
        <div className="card" style={{ padding: 0 }}>
          <table className="table">
            <thead style={{ position: "sticky", top: 0, background: "var(--bg-2)", zIndex: 1 }}>
              <tr>
                <th>Active</th>
                <th>Nom</th>
                <th>Type</th>
                <th>Emplacement</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {(sources || []).map((s) => (
                <tr key={s.id} style={{ background: s.id === active?.id ? "var(--bg-3)" : undefined }}>
                  <td>
                    {s.id === active?.id ? (
                      <span style={{ color: "var(--success)", fontWeight: 700 }}>✓</span>
                    ) : (
                      <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => handleActivate(s.id)}>Activer</button>
                    )}
                  </td>
                  <td>{s.name}</td>
                  <td><span className="badge">{s.kind === "remote" ? "Distant" : "Local"}</span></td>
                  <td style={{ fontFamily: "monospace", fontSize: 12 }}>{s.kind === "remote" ? s.remote_path : s.path}</td>
                  <td>
                    <div className="row" style={{ gap: 6 }}>
                      <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => handleTest(s.id)}>Tester</button>
                      <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => startEdit(s)}>Modifier</button>
                      <button className="btn danger" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => handleDelete(s.id)}>Supprimer</button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {dialogOpen && (
        <div className="modal-backdrop" onClick={() => setDialogOpen(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 520 }}>
            <h3 style={{ marginTop: 0 }}>{editing ? "Modifier la source" : "Ajouter une source"}</h3>

            {editing && (
              <div style={{ marginTop: 8, marginBottom: 8 }}>
                <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Type</label>
                <select className="select" value={editKind} onChange={(e) => setEditKind(e.target.value as SourceType)}>
                  <option value="local">Local</option>
                  <option value="remote">Distant (SSH)</option>
                </select>
              </div>
            )}

            {!editing && dialogType === null && (
              <div style={{ display: "flex", gap: 12, marginTop: 12 }}>
                <button className="btn" style={{ flex: 1, padding: 16, flexDirection: "column" }} onClick={() => setDialogType("local")}>
                  <div style={{ fontSize: 24, marginBottom: 4 }}>📁</div>
                  <div style={{ fontWeight: 600 }}>Source locale</div>
                  <div style={{ fontSize: 12, color: "var(--text-muted)" }}>Utiliser un dossier présent sur cet ordinateur</div>
                </button>
                <button className="btn" style={{ flex: 1, padding: 16, flexDirection: "column" }} onClick={() => setDialogType("remote")}>
                  <div style={{ fontSize: 24, marginBottom: 4 }}>🌐</div>
                  <div style={{ fontWeight: 600 }}>Source distante (SSH)</div>
                  <div style={{ fontSize: 12, color: "var(--text-muted)" }}>Accéder à un dossier ROMs sur une machine distante</div>
                </button>
              </div>
            )}

            {!editing && dialogType !== null && (
              <button className="btn" style={{ marginBottom: 12, fontSize: 12 }} onClick={() => setDialogType(null)}>← Retour</button>
            )}

            {((editing ? editKind : dialogType) === "local") && (
              <>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Nom</label>
                  <input className="input" value={editing ? editName : localName} onChange={(e) => editing ? setEditName(e.target.value) : setLocalName(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Chemin du dossier</label>
                  <div className="row" style={{ gap: 8 }}>
                    <input className="input" value={editing ? editPath : localPath} onChange={(e) => editing ? setEditPath(e.target.value) : setLocalPath(e.target.value)} />
                    <button className="btn" onClick={() => handlePickLocal(editing ? "edit" : "add")}>Parcourir…</button>
                  </div>
                </div>
              </>
            )}

            {((editing ? editKind : dialogType) === "remote") && (
              <>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Nom</label>
                  <input className="input" value={editing ? editName : rName} onChange={(e) => editing ? setEditName(e.target.value) : setRName(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>URL SFTP (optionnel)</label>
                  <input className="input" placeholder="sftp://user@host:port/chemin" onChange={(e) => editParseUrl(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Hôte (IP ou nom)</label>
                  <input className="input" value={editing ? editHost : rHost} onChange={(e) => editing ? setEditHost(e.target.value) : setRHost(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Port SSH</label>
                  <input className="input" value={editing ? editPort : rPort} onChange={(e) => editing ? setEditPort(e.target.value) : setRPort(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Utilisateur</label>
                  <input className="input" value={editing ? editUser : rUser} onChange={(e) => editing ? setEditUser(e.target.value) : setRUser(e.target.value)} />
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Clé SSH privée</label>
                  <div className="row" style={{ gap: 8 }}>
                    <input className="input" value={editing ? editKey : rKey} onChange={(e) => editing ? setEditKey(e.target.value) : setRKey(e.target.value)} />
                    <button className="btn" onClick={() => handlePickKey(editing ? "edit" : "add")}>Parcourir…</button>
                  </div>
                </div>
                <div style={{ marginTop: 8 }}>
                  <label style={{ fontSize: 12, color: "var(--text-muted)" }}>Chemin distant des ROMs</label>
                  <input className="input" value={editing ? editRPath : rPath} onChange={(e) => editing ? setEditRPath(e.target.value) : setRPath(e.target.value)} />
                </div>
              </>
            )}

            {error && <p style={{ color: "var(--danger)", fontSize: 12, marginTop: 8 }}>{error}</p>}

            {((dialogType === "local" || dialogType === "remote") || editing) && (
              <div className="row" style={{ marginTop: 16, justifyContent: "flex-end", gap: 8 }}>
                <button className="btn" onClick={() => setDialogOpen(false)}>Annuler</button>
                <button className="btn primary" onClick={saveSource} disabled={saving}>
                  {saving ? <span className="spinner" /> : "Enregistrer"}
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </>
  );
}
