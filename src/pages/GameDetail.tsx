import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api, pickDirectory } from "../lib/api";
import { convertFileSrc } from "@tauri-apps/api/core";
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
  const { data: allGenres } = useQuery<any[]>({ queryKey: ["genres-full"], queryFn: () => api.listGenresFull() });
  const { data: ssNoResult } = useQuery<boolean>({ queryKey: ["ss-no-result-one", id], queryFn: async () => { const ids = await api.ssNoResultGameIds(); return ids.includes(id!); }, enabled: !!id });
  const { data: presStatus } = useQuery<string>({ queryKey: ["preservation-status", id], queryFn: () => api.getPreservationStatus(id!), enabled: !!id });
  const { data: gameClassifications } = useQuery<any[]>({ queryKey: ["game-classifications", id], queryFn: () => api.listGameClassifications(id!), enabled: !!id });
  const { data: relatedGames } = useQuery<any[]>({ queryKey: ["related-games", id], queryFn: () => api.findRelatedGames(id!), enabled: !!id });
  const { data: achievements, refetch: refetchAch } = useQuery<any[]>({ queryKey: ["achievements", id], queryFn: () => api.listGameAchievements(id!), enabled: !!id });
  const { data: enrichHistory } = useQuery<any[]>({ queryKey: ["enrich-history", id], queryFn: () => api.listEnrichmentHistory(id!), enabled: !!id });
  const { data: developers } = useQuery<string[]>({ queryKey: ["developers"], queryFn: () => api.listDevelopers() });
  const { data: publishers } = useQuery<string[]>({ queryKey: ["publishers"], queryFn: () => api.listPublishers() });


  const [title, setTitle] = useState("");
  const [copyMsg, setCopyMsg] = useState<string | null>(null);
  const [hashingFile, setHashingFile] = useState<string | null>(null);
  const [hashingAll, setHashingAll] = useState(false);
  const [savingInfo, setSavingInfo] = useState(false);
  const [searchingSS, setSearchingSS] = useState(false);
  const [raSearching, setRaSearching] = useState(false);
  const [raResults, setRaResults] = useState<any[]>([]);
  const [raFetching, setRaFetching] = useState(false);
  const [sgdbSearching, setSgdbSearching] = useState(false);
  const [sgdbResults, setSgdbResults] = useState<any[]>([]);
  const [sgdbFetching, setSgdbFetching] = useState(false);
  const [tab, setTab] = useState<"info" | "files" | "hashes" | "media" | "versions" | "achievements">("info");
  const [infoPublisher, setInfoPublisher] = useState("");
  const [infoDeveloper, setInfoDeveloper] = useState("");
  const [infoGenres, setInfoGenres] = useState<string[]>([]);
  const [infoReleaseEU, setInfoReleaseEU] = useState("");
  const [infoReleaseUS, setInfoReleaseUS] = useState("");
  const [infoReleaseJP, setInfoReleaseJP] = useState("");
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
    const ssGenres = details?.genres || [];
    const localGenres = allGenres || [];
    let matched: string[] = [];
    if (Array.isArray(details?._genres)) {
      matched = details._genres;
    } else {
      matched = ssGenres.map((g: any) => {
        const local = localGenres.find((lg: any) => String(lg.ss_id) === String(g.id));
        if (local) return local.name_fr || local.name_en || local.name_de || local.name_es || local.name_it || local.name_pt || "";
        return "";
      }).filter(Boolean);
      if (!matched.length) {
        matched = ssGenres.map((g: any) => (g.noms || []).map((n: any) => n.text).join("/")).filter(Boolean);
      }
    }
    setInfoGenres(matched);
    setInfoPublisher(details.editeur?.text ?? details.publisher ?? "");
    setInfoDeveloper(details.developpeur?.text ?? details.developer ?? "");
    const dates = details.dates || [];
    setInfoReleaseEU(dates.find((d: any) => d.region === "eu")?.text ?? "");
    setInfoReleaseUS(dates.find((d: any) => d.region === "us")?.text ?? "");
    setInfoReleaseJP(dates.find((d: any) => d.region === "jp")?.text ?? "");
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
      <h1 className="page-title">{data.game.title}{presStatus && presStatus !== "unknown" && <span className="badge" style={{ fontSize: 11, marginLeft: 8, verticalAlign: "middle" }}>{presStatus === "verified" ? "✓ vérifié" : presStatus === "bad_dump" ? "✗ bad dump" : presStatus}</span>}</h1>
      <p className="page-sub">{data.platform?.name ?? "Plateforme inconnue"}</p>

      <div className="row" style={{ marginBottom: 16, gap: 8 }}>
        <button className={`btn ${tab === "info" ? "primary" : ""}`} onClick={() => setTab("info")}>Informations</button>
        <button className={`btn ${tab === "files" ? "primary" : ""}`} onClick={() => setTab("files")}>Fichiers</button>
        <button className={`btn ${tab === "hashes" ? "primary" : ""}`} onClick={() => setTab("hashes")}>Empreintes</button>
        <button className={`btn ${tab === "media" ? "primary" : ""}`} onClick={() => setTab("media")}>{t("game.media")}</button>
        <button className={`btn ${tab === "versions" ? "primary" : ""}`} onClick={() => setTab("versions")}>Versions</button>
        <button className={`btn ${tab === "achievements" ? "primary" : ""}`} onClick={() => setTab("achievements")}>Succès</button>
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
                <select className="select" multiple value={infoGenres} onChange={(e) => setInfoGenres(Array.from(e.target.selectedOptions, (o) => o.value))} style={{ width: "100%", minHeight: 80 }}>
                  {(allGenres || []).map((g: any) => {
                    const name = g.name_fr || g.name_en || g.name_de || g.name_es || g.name_it || g.name_pt || "—";
                    const parent = g.parent_id ? (allGenres || []).find((x: any) => x.id === g.parent_id) : null;
                    const parentName = parent ? (parent.name_fr || parent.name_en || parent.name_de || parent.name_es || parent.name_it || parent.name_pt || "") : "";
                    const label = parentName ? `  ${name} (${parentName})` : name;
                    return <option key={g.id} value={name}>{label}</option>;
                  })}
                  {infoGenres.filter((g) => !(allGenres || []).some((lg: any) => (lg.name_fr || lg.name_en) === g)).map((g) => (
                    <option key={g} value={g}>{g}</option>
                  ))}
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
              <td style={{ fontWeight: 600, verticalAlign: "top" }}>Classification(s)</td>
              <td>
                {(gameClassifications && gameClassifications.length > 0) ? (
                  <table className="table" style={{ fontSize: 11, width: "100%" }}>
                    <thead><tr><th>Organisme</th><th>Classification</th></tr></thead>
                    <tbody>
                      {gameClassifications.map((c: any) => (
                        <tr key={c.id}><td>{c.system_name ?? "—"}</td><td>{c.value}</td></tr>
                      ))}
                    </tbody>
                  </table>
                ) : (
                  <span style={{ color: "var(--text-muted)", fontSize: 12 }}>Aucune classification</span>
                )}
              </td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600, verticalAlign: "top" }}>Date(s) de sortie</td>
              <td>
                <table className="table" style={{ fontSize: 11, width: "100%" }}>
                  <thead><tr><th>Pays</th><th>Date de sortie</th></tr></thead>
                  <tbody>
                    <tr><td>Europe</td><td><input className="input" value={infoReleaseEU} onChange={(e) => setInfoReleaseEU(e.target.value)} style={{ width: "100%" }} type="date" /></td></tr>
                    <tr><td>USA</td><td><input className="input" value={infoReleaseUS} onChange={(e) => setInfoReleaseUS(e.target.value)} style={{ width: "100%" }} type="date" /></td></tr>
                    <tr><td>Japon</td><td><input className="input" value={infoReleaseJP} onChange={(e) => setInfoReleaseJP(e.target.value)} style={{ width: "100%" }} type="date" /></td></tr>
                  </tbody>
                </table>
              </td>
            </tr>
            <tr>
              <td style={{ fontWeight: 600, verticalAlign: "top" }}>Synopsis</td>
              <td><textarea className="input" value={infoSynopsis} onChange={(e) => setInfoSynopsis(e.target.value)} style={{ width: "100%", minHeight: 120 }} /></td>
            </tr>
          </tbody>
        </table>
        <div style={{ marginTop: 12, display: "flex", gap: 8 }}>
          {data.files.length > 0 && (
            <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={async () => {
              try {
                const { openPath } = await import("@tauri-apps/plugin-opener");
                const dir = data.files[0].path.substring(0, data.files[0].path.lastIndexOf("/"));
                await openPath(dir);
              } catch (e: any) {
                api.showNotification("RETROVERSE", String(e)).catch(() => {});
              }
            }}>
              Ouvrir le dossier
            </button>
          )}
          {data.files.length > 0 && (
            <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} onClick={async () => {
              try {
                const { openPath } = await import("@tauri-apps/plugin-opener");
                await openPath(data.files[0].path);
              } catch (e: any) {
                api.showNotification("RETROVERSE", String(e)).catch(() => {});
              }
            }}>
              {t("game.openFile")}
            </button>
          )}
          <button className="btn" style={{ fontSize: 11, padding: "4px 8px" }} disabled={searchingSS || !!ssNoResult || data.files.some((f: any) => !f.hash || !f.sha512 || ((f.extension === "zip" || f.extension === "7z") && !f.archive_hash))} onClick={async () => {
            if (!id) return;
            setSearchingSS(true);
            try {
              await api.enrichGame(id, "screenscraper");
              qc.invalidateQueries({ queryKey: ["game", id] });
              qc.invalidateQueries({ queryKey: ["game-metadata", id] });
              qc.invalidateQueries({ queryKey: ["game-classifications", id] });
              qc.invalidateQueries({ queryKey: ["ss-no-result-one", id] });
              qc.invalidateQueries({ queryKey: ["ss-no-result"] });
              api.showNotification("RETROVERSE", "Métadonnées récupérées.").catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            } finally {
              setSearchingSS(false);
            }
          }}>
            {searchingSS ? <><span className="spinner" /> Recherche en cours...</> : ssNoResult ? "Aucun résultat ScreenScraper" : "Rechercher sur ScreenScraper"}
          </button>
          {(enrichHistory || []).length > 0 && (
            <div style={{ width: "100%", marginTop: 8 }}>
              <div style={{ fontSize: 12, fontWeight: 600, marginBottom: 4 }}>Historique d'enrichissement</div>
              <ul style={{ margin: 0, padding: 0, listStyle: "none", fontSize: 12 }}>
                {(enrichHistory || []).map((h: any) => (
                  <li key={h.id} style={{ display: "flex", justifyContent: "space-between", padding: "2px 0" }}>
                    <span>{new Date(h.created_at).toLocaleString()} — {h.provider}{h.reverted ? " (annulé)" : ""}</span>
                    {!h.reverted && (
                      <button className="btn" style={{ fontSize: 10, padding: "2px 6px" }} onClick={async () => {
                        try {
                          await api.revertEnrichment(h.id);
                          qc.invalidateQueries({ queryKey: ["game", id] });
                          qc.invalidateQueries({ queryKey: ["game-metadata", id] });
                          qc.invalidateQueries({ queryKey: ["enrich-history", id] });
                          api.showNotification("RETROVERSE", "Enrichissement annulé.").catch(() => {});
                        } catch (e) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
                      }}>Annuler</button>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          )}
          <button className="btn primary" style={{ fontSize: 11, padding: "4px 8px" }} disabled={savingInfo} onClick={async () => {
            if (!id) return;
            setSavingInfo(true);
            try {
              const meta = details ? {
                ...details,
                editeur: { text: infoPublisher },
                developpeur: { text: infoDeveloper },
                _genres: infoGenres,
                dates: [
                  infoReleaseEU ? { region: "eu", text: infoReleaseEU } : null,
                  infoReleaseUS ? { region: "us", text: infoReleaseUS } : null,
                  infoReleaseJP ? { region: "jp", text: infoReleaseJP } : null,
                ].filter(Boolean),
                synopsis: infoSynopsis,
              } : {
                editeur: { text: infoPublisher },
                developpeur: { text: infoDeveloper },
                _genres: infoGenres,
                dates: [
                  infoReleaseEU ? { region: "eu", text: infoReleaseEU } : null,
                  infoReleaseUS ? { region: "us", text: infoReleaseUS } : null,
                  infoReleaseJP ? { region: "jp", text: infoReleaseJP } : null,
                ].filter(Boolean),
                synopsis: infoSynopsis,
              };
              await api.saveGameInfo(id, title || data.game.title, JSON.stringify(meta));
              await api.setGameMetadata(id, "description", infoSynopsis).catch(() => {});
              qc.invalidateQueries({ queryKey: ["game", id] });
              qc.invalidateQueries({ queryKey: ["game-metadata", id] });
              api.showNotification("RETROVERSE", "Informations enregistrées.").catch(() => {});
            } catch (e) {
              api.showNotification("RETROVERSE", String(e)).catch(() => {});
            } finally {
              setSavingInfo(false);
            }
          }}>
            {savingInfo ? <><span className="spinner" /> Enregistrement en cours...</> : "Enregistrer toutes les informations"}
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

      {tab === "media" && (
      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Médias</h3>
        {details?.cover_url || (metadata || []).some((m: any) => m.kind === "cover") ? (
          <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
            {(() => {
              const ma = (metadata || []).find((m: any) => m.kind === "cover");
              let src = ma?.path ? convertFileSrc(ma.path) : (details?.cover_url || null);
              if (!src && Array.isArray(details?.medias)) {
                const m = details.medias.find((x: any) => x?.type === "box-2D" || x?.type === "box-3D");
                src = m?.url || null;
              }
              return src ? <img src={src} alt="cover" style={{ maxWidth: 240, borderRadius: 8 }} /> : null;
            })()}
            {(details?.medias || []).filter((m: any) => m?.url).slice(0, 6).map((m: any, i: number) => (
              <img key={i} src={m.url} alt={m.type || "media"} style={{ maxWidth: 160, borderRadius: 8 }} />
            ))}
          </div>
        ) : (
          <p style={{ color: "var(--text-muted)" }}>{t("game.noMedia")}</p>
        )}
        <div style={{ marginTop: 12, display: "flex", gap: 8, flexWrap: "wrap" }}>
          <button className="btn" disabled={sgdbSearching} onClick={async () => {
            if (!id) return;
            setSgdbSearching(true);
            try {
              const r = await api.sgdbSearchGame(data?.game?.title || "");
              setSgdbResults(r || []);
            } catch (e: any) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
            finally { setSgdbSearching(false); }
          }}>{sgdbSearching ? "Recherche…" : "Rechercher sur SteamGridDB"}</button>
        </div>
        {sgdbResults.length > 0 && (
          <ul style={{ fontSize: 12, paddingLeft: 18, marginTop: 8 }}>
            {sgdbResults.map((r: any) => (
              <li key={r.id} style={{ marginBottom: 4 }}>
                {r.name}
                <button className="btn" style={{ fontSize: 10, marginLeft: 8, padding: "2px 6px" }} disabled={sgdbFetching} onClick={async () => {
                  if (!id) return;
                  setSgdbFetching(true);
                  try {
                    const n = await api.sgdbFetchArtworks(id, r.id);
                    qc.invalidateQueries({ queryKey: ["game-metadata", id] });
                    api.showNotification("RETROVERSE", `${n} artwork(s) récupéré(s).`).catch(() => {});
                  } catch (e: any) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
                  finally { setSgdbFetching(false); }
                }}>Récupérer</button>
              </li>
            ))}
          </ul>
        )}
      </div>
      )}
      {tab === "versions" && (
      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Versions / Régions</h3>
        <p style={{ fontSize: 12, color: "var(--text-muted)" }}>Fichiers liés à ce jeu :</p>
        <table className="table" style={{ width: "100%", fontSize: 12 }}>
          <thead><tr><th>Fichier</th><th>Taille</th><th>CRC32</th><th>MD5</th><th>SHA1</th></tr></thead>
          <tbody>
            {(data?.files || []).map((f: any) => (
              <tr key={f.id}>
                <td style={{ wordBreak: "break-all" }}>{f.filename}</td>
                <td>{f.size}</td>
                <td style={{ fontFamily: "monospace", fontSize: 10 }}>{f.crc32 || "—"}</td>
                <td style={{ fontFamily: "monospace", fontSize: 10 }}>{f.md5 ? f.md5.slice(0, 8) + "…" : "—"}</td>
                <td style={{ fontFamily: "monospace", fontSize: 10 }}>{f.sha1 ? f.sha1.slice(0, 8) + "…" : "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {(relatedGames || []).length > 0 && (
          <>
            <h4 style={{ marginTop: 16 }}>Jeux liés (même titre, autre plateforme)</h4>
            <ul style={{ fontSize: 12, paddingLeft: 18 }}>
              {(relatedGames || []).map((g: any) => (
                <li key={g.id}>
                  <a href={`/game/${g.id}`} onClick={(e) => { e.preventDefault(); navigate(`/game/${g.id}`); }}>{g.title}</a>
                  {g.platform_id ? ` — ${g.platform_id}` : ""}
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
      )}
      {tab === "achievements" && (
      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Succès RetroAchievements</h3>
        <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
          <button className="btn" disabled={raSearching} onClick={async () => {
            if (!id) return;
            setRaSearching(true);
            try {
              const r = await api.raSearchGame(data?.game?.title || "");
              setRaResults(r || []);
            } catch (e: any) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
            finally { setRaSearching(false); }
          }}>{raSearching ? "Recherche…" : "Rechercher sur RetroAchievements"}</button>
        </div>
        {raResults.length > 0 && (
          <ul style={{ fontSize: 12, paddingLeft: 18, marginBottom: 12 }}>
            {raResults.map((r: any) => (
              <li key={r.id} style={{ marginBottom: 4 }}>
                {r.title} ({r.console})
                <button className="btn" style={{ fontSize: 10, marginLeft: 8, padding: "2px 6px" }} disabled={raFetching} onClick={async () => {
                  if (!id) return;
                  setRaFetching(true);
                  try {
                    const n = await api.raFetchAchievements(id, r.id);
                    await refetchAch();
                    api.showNotification("RETROVERSE", `${n} succès récupérés.`).catch(() => {});
                  } catch (e: any) { api.showNotification("RETROVERSE", String(e)).catch(() => {}); }
                  finally { setRaFetching(false); }
                }}>Récupérer</button>
              </li>
            ))}
          </ul>
        )}
        {(achievements || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>Aucun succès. Recherchez le jeu sur RetroAchievements pour récupérer les succès.</p>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: 10 }}>
            {(achievements || []).map((a: any) => (
              <div key={a.id} style={{ border: "1px solid var(--border)", borderRadius: 8, padding: 8, opacity: a.unlocked ? 1 : 0.5 }}>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  {a.badge && <img src={`https://media.retroachievements.org/Badge/${a.badge}${a.unlocked ? "" : "_lock"}.png`} alt="" style={{ width: 48, height: 48 }} onError={(e: any) => { e.target.style.display = "none"; }} />}
                  <div>
                    <div style={{ fontWeight: 600, fontSize: 13 }}>{a.title}</div>
                    <div style={{ fontSize: 11, color: "var(--text-muted)" }}>{a.points} pts</div>
                  </div>
                </div>
                {a.description && <p style={{ fontSize: 11, marginTop: 6, color: "var(--text-dim)" }}>{a.description}</p>}
              </div>
            ))}
          </div>
        )}
      </div>
      )}
    </div>
  );
}