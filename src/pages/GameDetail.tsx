import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import { formatBytes } from "../lib/format";
import { useI18n } from "../i18n";
import type { LibraryEntry, GameSource } from "../types";

export default function GameDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { t } = useI18n();
  const qc = useQueryClient();
  const { data, isLoading } = useQuery<LibraryEntry>({
    queryKey: ["game", id],
    queryFn: () => api.getGame(id!),
    enabled: !!id,
  });
  const { data: sources } = useQuery<GameSource[]>({
    queryKey: ["sources"],
    queryFn: () => api.listSources(),
  });
  const { data: metadata } = useQuery<any[]>({
    queryKey: ["game-metadata", id],
    queryFn: () => api.getGameMetadata(id!),
    enabled: !!id,
  });
  const { data: genres } = useQuery<string[]>({ queryKey: ["genres"], queryFn: () => api.listGenres() });
  const { data: developers } = useQuery<string[]>({ queryKey: ["developers"], queryFn: () => api.listDevelopers() });
  const { data: publishers } = useQuery<string[]>({ queryKey: ["publishers"], queryFn: () => api.listPublishers() });


  const [title, setTitle] = useState("");
  const [copyMsg, setCopyMsg] = useState<string | null>(null);
  const [hashingFile, setHashingFile] = useState<string | null>(null);
  const [hashingAll, setHashingAll] = useState(false);
  const [tab, setTab] = useState<"info" | "files" | "hashes">("info");
  const [infoPublisher, setInfoPublisher] = useState("");
  const [infoDeveloper, setInfoDeveloper] = useState("");
  const [infoGenre, setInfoGenre] = useState("");
  const [infoClassifications, setInfoClassifications] = useState("");
  const [infoReleaseDates, setInfoReleaseDates] = useState("");
  const [infoSynopsis, setInfoSynopsis] = useState("");

  useEffect(() => {
    if (data) setTitle(data.game.title);
  }, [data]);

  const sourceMap = new Map(sources?.map((s) => [s.id, s]) || []);
  const detailsRecord = (metadata || []).find((m: any) => m.field === "details");
  const details = data?.game?.metadata_json ? JSON.parse(data.game.metadata_json) : (detailsRecord ? JSON.parse(detailsRecord.value) : null);
  const descRecord = (metadata || []).find((m: any) => m.field === "description");
  const synopsis = descRecord ? descRecord.value : null;
  useEffect(() => {
    if (!details) return;
    setInfoGenre(details.genres ? details.genres.map((g: any) => (g.noms || []).map((n: any) => n.text).join("/")).join(", ") : "");
    setInfoPublisher(details.editeur?.text ?? details.publisher ?? "");
    setInfoDeveloper(details.developpeur?.text ?? details.developer ?? "");
    setInfoClassifications(details.classifications ? details.classifications.map((c: any) => `${c.type} ${c.text}`).join(", ") : "");
    setInfoReleaseDates(details.dates ? details.dates.map((d: any) => `${d.region}: ${d.text}`).join(", ") : "");
    setInfoSynopsis(synopsis ?? "");
  }, [details, synopsis]);
  const isRemote = data?.files.some((f) => sourceMap.get(f.source_id)?.kind === "remote") ?? false;
  const remoteSourceId = data?.files.map((f) => f.source_id).find((sid) => sourceMap.get(sid)?.kind === "remote");

  const hashFile = async (fileId: string) => {
    setHashingFile(fileId);
    try {
      const n = await api.computeHashesForFile(fileId);
      api.showNotification("RETROVERSE", `Empreinte calculée pour ${n} fichier(s).`).catch(() => {});
      qc.invalidateQueries({ queryKey: ["game", id] });
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setHashingFile(null);
    }
  };

  const hashAll = async () => {
    setHashingAll(true);
    try {
      const n = await api.computeHashesForGame(id!);
      api.showNotification("RETROVERSE", `Empreintes calculées : ${n} fichier(s).`).catch(() => {});
      qc.invalidateQueries({ queryKey: ["game", id] });
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setHashingAll(false);
    }
  };

  async function handleCopy() {
    if (!remoteSourceId || !data) return;
    const dest = await pickDirectory();
    if (!dest) return;
    setCopyMsg(null);
    try {
      const res = await api.copyRemoteFiles(remoteSourceId, data.files.map((f) => f.id), dest);
      setCopyMsg(`Copié : ${res.files} fichier(s), ${res.bytes} octets.${res.errors.length ? " Erreurs: " + res.errors.join("; ") : ""}`);
    } catch (e: any) {
      setCopyMsg(String(e));
    }
  }

  if (isLoading || !data) {
    return <div className="empty"><div className="spinner" /></div>;
  }

  return (
    <div>
      <button className="btn" onClick={() => navigate(-1)} style={{ marginBottom: 16 }}>{t("common.back")}</button>
      <h1 className="page-title">{data.game.title}</h1>
      <p className="page-sub">{data.platform?.name ?? "Plateforme inconnue"}</p>

      <div className="row" style={{ marginBottom: 16, gap: 8 }}>
        <button className={`btn ${tab === "info" ? "primary" : ""}`} onClick={() => setTab("info")}>Informations</button>
        <button className={`btn ${tab === "files" ? "primary" : ""}`} onClick={() => setTab("files")}>Fichiers</button>
        <button className={`btn ${tab === "hashes" ? "primary" : ""}`} onClick={() => setTab("hashes")}>Empreintes</button>
      </div>

      {tab === "info" && (
      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("gameDetail.info")}</h3>
        {isRemote && (
          <div style={{ marginTop: 12 }}>
            <button className="btn primary" onClick={handleCopy}>Copier localement…</button>
            {copyMsg && <p style={{ color: "var(--text-dim)", marginTop: 8, fontSize: 13 }}>{copyMsg}</p>}
          </div>
        )}
        <table className="table" style={{ fontSize: 12, marginTop: 12 }}>
          <tbody>
            <tr>
              <td style={{ width: 160, fontWeight: 600 }}>Titre</td>
              <td><input className="input" value={title} onChange={(e) => setTitle(e.target.value)} style={{ width: "100%" }} /></td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600 }}>Genre</td>
              <td>
                <select className="select" value={infoGenre} onChange={(e) => setInfoGenre(e.target.value)} style={{ width: "100%" }}>
                  <option value="">—</option>
                  {(genres || []).map((g) => <option key={g} value={g}>{g}</option>)}
                  {!genres?.includes(infoGenre) && infoGenre && <option value={infoGenre}>{infoGenre}</option>}
                </select>
              </td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600 }}>Éditeur(s)</td>
              <td>
                <select className="select" value={infoPublisher} onChange={(e) => setInfoPublisher(e.target.value)} style={{ width: "100%" }}>
                  <option value="">—</option>
                  {(publishers || []).map((p) => <option key={p} value={p}>{p}</option>)}
                  {!publishers?.includes(infoPublisher) && infoPublisher && <option value={infoPublisher}>{infoPublisher}</option>}
                </select>
              </td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600 }}>Développeur(s)</td>
              <td>
                <select className="select" value={infoDeveloper} onChange={(e) => setInfoDeveloper(e.target.value)} style={{ width: "100%" }}>
                  <option value="">—</option>
                  {(developers || []).map((d) => <option key={d} value={d}>{d}</option>)}
                  {!developers?.includes(infoDeveloper) && infoDeveloper && <option value={infoDeveloper}>{infoDeveloper}</option>}
                </select>
              </td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600 }}>Classification(s)</td>
              <td><input className="input" value={infoClassifications} onChange={(e) => setInfoClassifications(e.target.value)} style={{ width: "100%" }} /></td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600 }}>Date(s) de sortie</td>
              <td><input className="input" value={infoReleaseDates} onChange={(e) => setInfoReleaseDates(e.target.value)} style={{ width: "100%" }} /></td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600, verticalAlign: "top" }}>Synopsis</td>
              <td><textarea className="input" value={infoSynopsis} onChange={(e) => setInfoSynopsis(e.target.value)} style={{ width: "100%", minHeight: 120 }} /></td>
            </tr>
          </tbody>
        </table>
        <div style={{ marginTop: 12, display: "flex", gap: 8 }}>
          <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={async () => {
            if (!id) return;
            try {
              await api.enrichGame(id, "screenscraper");
              qc.invalidateQueries({ queryKey: ["game-metadata", id] });
              api.showNotification("RETROVERSE", "Métadonnées récupérées.").catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            }
          }}>
            Rechercher sur ScreenScraper
          </button>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} onClick={async () => {
            if (!id) return;
            try {
              const meta = details ? { ...details, editeur: { text: infoPublisher }, developpeur: { text: infoDeveloper } } : { editeur: { text: infoPublisher }, developpeur: { text: infoDeveloper } };
              await api.saveGameInfo(id, title || data.game.title, JSON.stringify(meta));
              qc.invalidateQueries({ queryKey: ["game", id] });
              api.showNotification("RETROVERSE", "Informations enregistrées.").catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            }
          }}>
            Enregistrer toutes les informations
          </button>
        </div>
      </div>
      )}

      {tab === "files" && (
      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("gameDetail.files")} ({data.files.length})</h3>
        <table className="table">
          <thead>
            <tr>
              <th>Nom</th>
              <th>Extension</th>
              <th>Taille</th>
              <th>Chemin</th>
            </tr>
          </thead>
          <tbody>
            {data.files.map((f) => (
              <tr key={f.id}>
                <td>{f.filename}</td>
                <td><span className="badge">{f.extension}</span></td>
                <td>{formatBytes(f.size)}</td>
                <td style={{ fontFamily: "monospace", fontSize: 11, color: "var(--text-muted)" }}>{f.path}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      )}



      {tab === "hashes" && (
      <div className="card" style={{ marginTop: 16 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
          <h3 style={{ marginTop: 0, marginBottom: 0 }}>{t("gameDetail.hashes")}</h3>
          <button className="btn primary" style={{ fontSize: 11, padding: "2px 8px" }} onClick={hashAll} disabled={hashingAll || data.files.every((f: any) => f.extension === "scummvm") || !data.files.some((f: any) => !f.hash || !f.sha512 || ((f.extension === "zip" || f.extension === "7z") && !f.archive_hash))}>
            {hashingAll ? "…" : "Calculer toutes les empreintes"}
          </button>
        </div>
        {data.files.map((f: any, idx: number) => (
          <div key={f.id} style={{ marginBottom: 16, paddingBottom: 12, borderBottom: idx < data.files.length - 1 ? "1px solid var(--border)" : "none", fontSize: 12, fontFamily: "monospace", color: "var(--text-dim)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8, flexWrap: "wrap" }}>
              <span style={{ fontWeight: 600, color: "var(--text)" }}>{f.filename}</span>

              {!f.is_dir && (
                <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => hashFile(f.id)} disabled={hashingFile === f.id || f.extension === "scummvm" || (!!f.hash && !!f.sha512 && !((f.extension === "zip" || f.extension === "7z") && !f.archive_hash))}>
                  {hashingFile === f.id ? "…" : "Calculer les empreintes"}
                </button>
              )}

            </div>

            {(f.extension === "zip" || f.extension === "7z") ? (
              <table className="table" style={{ fontSize: 11, marginBottom: 8 }}>
                <thead><tr><th colSpan={2} style={{ textAlign: "left" }}>Fichier (archive)</th></tr></thead>
                <tbody>
                  {f.archive_crc32 ? <tr><td>CRC32</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_crc32}</td></tr> : <tr><td>CRC32</td><td style={{ color: "var(--text-muted)" }}>non calculé</td></tr>}
                  {f.archive_md5 ? <tr><td>MD5</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_md5}</td></tr> : <tr><td>MD5</td><td style={{ color: "var(--text-muted)" }}>non calculé</td></tr>}
                  {f.archive_sha1 ? <tr><td>SHA1</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_sha1}</td></tr> : <tr><td>SHA1</td><td style={{ color: "var(--text-muted)" }}>non calculé</td></tr>}
                  {f.archive_hash ? <tr><td>SHA256</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_hash}</td></tr> : <tr><td>SHA256</td><td style={{ color: "var(--text-muted)" }}>non calculé</td></tr>}
                  {f.archive_sha512 ? <tr><td>SHA512</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_sha512}</td></tr> : <tr><td>SHA512</td><td style={{ color: "var(--text-muted)" }}>non calculé</td></tr>}
                </tbody>
              </table>
            ) : (
              <table className="table" style={{ fontSize: 11, marginBottom: 8 }}>
                <thead><tr><th colSpan={2} style={{ textAlign: "left" }}>Fichier</th></tr></thead>
                <tbody>
                  {f.crc32 && <tr><td>CRC32</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.crc32}</td></tr>}
                  {f.md5 && <tr><td>MD5</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.md5}</td></tr>}
                  {f.sha1 && <tr><td>SHA1</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.sha1}</td></tr>}
                  {f.hash && <tr><td>SHA256</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.hash}</td></tr>}
                  {f.sha512 && <tr><td>SHA512</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.sha512}</td></tr>}
                </tbody>
              </table>
            )}
            {f.zip_inner && (
              <>
                <div style={{ fontWeight: 600, color: "var(--text)", marginBottom: 4, whiteSpace: "nowrap" }}>ROM interne : {f.zip_inner}</div>
                <table className="table" style={{ fontSize: 11, marginBottom: 8 }}>
                <tbody>
                  {f.crc32 && <tr><td>CRC32</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.crc32}</td></tr>}
                  {f.md5 && <tr><td>MD5</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.md5}</td></tr>}
                  {f.sha1 && <tr><td>SHA1</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.sha1}</td></tr>}
                  {f.hash && <tr><td>SHA256</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.hash}</td></tr>}
                  {f.sha512 && <tr><td>SHA512</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.sha512}</td></tr>}
                </tbody>
              </table>
              </>
            )}
          </div>
        ))}
      </div>
      )}
    </div>
  );
}