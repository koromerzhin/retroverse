import { useEffect, useRef, useState } from "react";
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
  const enrichedRef = useRef(false);
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


  const [title, setTitle] = useState("");
  const [copyMsg, setCopyMsg] = useState<string | null>(null);
  const [hashingFile, setHashingFile] = useState<string | null>(null);
  const [hashingAll, setHashingAll] = useState(false);
  const [enriching, setEnriching] = useState(false);

  useEffect(() => {
    if (data) setTitle(data.game.title);
  }, [data]);

  const sourceMap = new Map(sources?.map((s) => [s.id, s]) || []);
  const detailsRecord = (metadata || []).find((m: any) => m.field === "details");
  const details = detailsRecord ? JSON.parse(detailsRecord.value) : null;
  const descRecord = (metadata || []).find((m: any) => m.field === "description");
  const synopsis = descRecord ? descRecord.value : null;
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

  const enrich = async (provider: string) => {
    setEnriching(true);
    try {
      await api.enrichGame(id!, provider);
      qc.invalidateQueries({ queryKey: ["game-metadata", id] });
      api.showNotification("RETROVERSE", "Métadonnées enrichies.").catch(() => {});
    } catch (e) {
      api.showNotification("RETROVERSE", String(e)).catch(() => {});
    } finally {
      setEnriching(false);
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

  useEffect(() => {
    if (!id || enrichedRef.current) return;
    if (metadata && metadata.length > 0) return;
    enrichedRef.current = true;
    api.enrichGame(id, "screenscraper").then(() => {
      qc.invalidateQueries({ queryKey: ["game-metadata", id] });
    }).catch(() => {});
  }, [id, metadata, qc]);

  if (isLoading || !data) {
    return <div className="empty"><div className="spinner" /></div>;
  }

  return (
    <div>
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
        <div style={{ marginTop: 12, fontSize: 13, color: "var(--text-dim)" }}>
          <div><strong>Titre :</strong> {details?.noms ? details.noms.map((n: any) => n.text).join(", ") : data.game.title}</div>
          <div><strong>Genre :</strong> {details?.genres ? details.genres.map((g: any) => (g.noms || []).map((n: any) => n.text).join("/")).join(", ") : ""}</div>
          <div><strong>Éditeur(s) :</strong> {details?.editeur?.text ?? ""}</div>
          <div><strong>Développeur(s) :</strong> {details?.developpeur?.text ?? ""}</div>
          <div><strong>Classification(s) :</strong> {details?.classifications ? details.classifications.map((c: any) => `${c.type} ${c.text}`).join(", ") : ""}</div>
          <div><strong>Date(s) de sortie :</strong> {details?.dates ? details.dates.map((d: any) => `${d.region}: ${d.text}`).join(", ") : ""}</div>
          <div style={{ marginTop: 8, whiteSpace: "pre-wrap" }}><strong>Synopsis :</strong> {synopsis ?? ""}</div>
        </div>
        <div style={{ marginTop: 12 }}>
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} onClick={async () => {
            if (!id) return;
            try {
              await api.enrichGame(id, "screenscraper");
              qc.invalidateQueries({ queryKey: ["game-metadata", id] });
              api.showNotification("RETROVERSE", "Informations enregistrées.").catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            }
          }}>
            Enregistrer toutes les informations
          </button>
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
        <h3 style={{ marginTop: 0 }}>Métadonnées</h3>
        <div className="row" style={{ gap: 8 }}>
          <button className="btn primary" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => enrich("screenscraper")} disabled={enriching}>
            {enriching ? "…" : "Enrichir (ScreenScraper)"}
          </button>
          <button className="btn" style={{ fontSize: 11, padding: "2px 8px" }} onClick={() => enrich("thegamesdb")} disabled={enriching}>
            {enriching ? "…" : "Enrichir (TheGamesDB)"}
          </button>
        </div>
      </div>



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
    </div>
  );
}