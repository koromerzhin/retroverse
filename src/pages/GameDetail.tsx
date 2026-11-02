import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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


  const [title, setTitle] = useState("");
  const [copyMsg, setCopyMsg] = useState<string | null>(null);
  const [hashingFile, setHashingFile] = useState<string | null>(null);
  const [hashingAll, setHashingAll] = useState(false);

  useEffect(() => {
    if (data) setTitle(data.game.title);
  }, [data]);

  const sourceMap = new Map(sources?.map((s) => [s.id, s]) || []);
  const isRemote = data?.files.some((f) => sourceMap.get(f.source_id)?.kind === "remote") ?? false;
  const remoteSourceId = data?.files.map((f) => f.source_id).find((sid) => sourceMap.get(sid)?.kind === "remote");

  const hashFile = async (fileId: string) => {
    setHashingFile(fileId);
    try {
      await api.computeHashesForFile(fileId);
      qc.invalidateQueries({ queryKey: ["game", id] });
    } catch (e) {
      console.error(e);
    } finally {
      setHashingFile(null);
    }
  };

  const hashAll = async () => {
    setHashingAll(true);
    try {
      await api.computeHashesForGame(id!);
      qc.invalidateQueries({ queryKey: ["game", id] });
    } catch (e) {
      console.error(e);
    } finally {
      setHashingAll(false);
    }
  };

  const saveMutation = useMutation({
    mutationFn: () => api.correctTitle(id!, title),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["game", id] });
      qc.invalidateQueries({ queryKey: ["games"] });
    },
  });

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
    <>
      <button className="btn" onClick={() => navigate(-1)} style={{ marginBottom: 16 }}>{t("common.back")}</button>
      <h1 className="page-title">{data.game.title}</h1>
      <p className="page-sub">{data.platform?.name ?? "Plateforme inconnue"}</p>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>{t("gameDetail.info")}</h3>
        <div className="row">
          <input className="input" value={title} onChange={(e) => setTitle(e.target.value)} style={{ maxWidth: 400 }} />
          <button className="btn primary" onClick={() => saveMutation.mutate()} disabled={saveMutation.isPending}>
            {saveMutation.isPending ? <span className="spinner" /> : t("gameDetail.saveTitle")}
          </button>
        </div>
        {saveMutation.isSuccess && <p style={{ color: "var(--success)", marginTop: 8 }}>{t("gameDetail.titleUpdated")}</p>}
        {isRemote && (
          <div style={{ marginTop: 12 }}>
            <button className="btn primary" onClick={handleCopy}>Copier localement…</button>
            {copyMsg && <p style={{ color: "var(--text-dim)", marginTop: 8, fontSize: 13 }}>{copyMsg}</p>}
          </div>
        )}
        <div style={{ marginTop: 12, fontSize: 12, color: "var(--text-muted)" }}>
          Créé le {new Date(data.game.created_at).toLocaleString()} · Mis à jour le {new Date(data.game.updated_at).toLocaleString()}
        </div>
      </div>



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

      <div className="card" style={{ marginTop: 16 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
          <h3 style={{ marginTop: 0, marginBottom: 0 }}>{t("gameDetail.hashes")}</h3>
          <button className="btn primary" style={{ fontSize: 11, padding: "2px 8px" }} onClick={hashAll} disabled={hashingAll || !data.files.some((f: any) => !f.hash || !f.sha512 || ((f.extension === "zip" || f.extension === "7z") && !f.archive_hash))}>
            {hashingAll ? "…" : "Calculer toutes les empreintes"}
          </button>
        </div>
        {data.files.map((f: any, idx: number) => (
          <div key={f.id} style={{ marginBottom: 16, paddingBottom: 12, borderBottom: idx < data.files.length - 1 ? "1px solid var(--border)" : "none", fontSize: 12, fontFamily: "monospace", color: "var(--text-dim)" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8, flexWrap: "wrap" }}>
              <span style={{ fontWeight: 600, color: "var(--text)" }}>{f.filename}</span>
              {!f.is_dir && (
                <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => hashFile(f.id)} disabled={hashingFile === f.id || (!!f.hash && !!f.sha512 && !((f.extension === "zip" || f.extension === "7z") && !f.archive_hash))}>
                  {hashingFile === f.id ? "…" : "Calculer les empreintes"}
                </button>
              )}
              <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => {
                const lines: string[] = [];
                const push = (label: string, value: string | null) => { if (value) lines.push(`${label} : ${value}`); };
                if (f.archive_hash || f.archive_md5 || f.archive_sha1 || f.archive_sha512 || f.archive_crc32) {
                  lines.push("Fichier");
                  push("CRC32", f.archive_crc32);
                  push("MD5", f.archive_md5);
                  push("SHA1", f.archive_sha1);
                  push("SHA256", f.archive_hash);
                  push("SHA512", f.archive_sha512);
                } else {
                  lines.push("Fichier");
                  push("CRC32", f.crc32);
                  push("MD5", f.md5);
                  push("SHA1", f.sha1);
                  push("SHA256", f.hash);
                  push("SHA512", f.sha512);
                }
                if (f.zip_inner) {
                  lines.push("");
                  lines.push(`ROM interne : ${f.zip_inner}`);
                  push("CRC32", f.crc32);
                  push("MD5", f.md5);
                  push("SHA1", f.sha1);
                  push("SHA256", f.hash);
                  push("SHA512", f.sha512);
                }
                navigator.clipboard?.writeText(lines.join("\n")).catch(() => {});
              }}>
                Copier
              </button>
            </div>

            {(f.archive_hash || f.archive_md5 || f.archive_sha1 || f.archive_sha512 || f.archive_crc32) ? (
              <table className="table" style={{ fontSize: 11, marginBottom: 8 }}>
                <thead><tr><th colSpan={2} style={{ textAlign: "left" }}>Fichier</th></tr></thead>
                <tbody>
                  {f.archive_crc32 && <tr><td>CRC32</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_crc32}</td></tr>}
                  {f.archive_md5 && <tr><td>MD5</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_md5}</td></tr>}
                  {f.archive_sha1 && <tr><td>SHA1</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_sha1}</td></tr>}
                  {f.archive_hash && <tr><td>SHA256</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_hash}</td></tr>}
                  {f.archive_sha512 && <tr><td>SHA512</td><td style={{ fontFamily: "monospace", wordBreak: "break-all" }}>{f.archive_sha512}</td></tr>}
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
    </>
  );
}