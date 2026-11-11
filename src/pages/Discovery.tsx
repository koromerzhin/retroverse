import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { formatNumber } from "../lib/format";
import { useI18n } from "../i18n";
import { SearchInput, EmptyState } from "../components";

export default function Discovery() {
  const qc = useQueryClient();
  const { t } = useI18n();
  const { data: duplicates } = useQuery({ queryKey: ["duplicates"], queryFn: () => api.findDuplicates() });
  const { data: collections } = useQuery({ queryKey: ["collections"], queryFn: () => api.listCollections() });
  const { data: gamesRes } = useQuery({ queryKey: ["games"], queryFn: () => api.listGames(undefined, undefined, undefined, undefined, undefined, 100000, 0) });
  const games = gamesRes?.items;

  const [search, setSearch] = useState("");
  const [collName, setCollName] = useState("");
  const createMut = useMutation({
    mutationFn: () => api.createCollection(collName.trim()),
    onSuccess: () => { setCollName(""); qc.invalidateQueries({ queryKey: ["collections"] }); },
  });

  const missing = (games || []).filter((g: any) => g.files.some((f: any) => f.missing === 1) || g.files.length === 0);
  const filteredMissing = missing.filter((g: any) => !search || g.game.title.toLowerCase().includes(search.toLowerCase()));

  return (
    <>
      <h1 className="page-title">{t("discovery.title")}</h1>
      <p className="page-sub">{t("discovery.subtitle")}</p>

      <div className="row" style={{ marginBottom: 16, alignItems: "center", flexWrap: "wrap", gap: 8 }}>
        <SearchInput value={search} onChange={setSearch} placeholder="Rechercher…" maxWidth={260} />
      </div>

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Doublons détectés</h3>
        {(duplicates || []).length === 0 ? (
          <EmptyState icon="🔍" message="Aucun doublon." />
        ) : (
          (duplicates || []).map((d: any, i: number) => (
            <div key={i} style={{ marginBottom: 10 }}>
              <strong>{d.title}</strong> — {d.games.length} exemplaires
              <ul style={{ margin: "4px 0", paddingLeft: 20, fontSize: 13, color: "var(--text-dim)" }}>
                {d.games.map((g: any) => <li key={g[0]}>{g[1]} ({g[0].slice(0,8)})</li>)}
              </ul>
            </div>
          ))
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Collections</h3>
        <div className="row" style={{ marginBottom: 12, gap: 8 }}>
          <input className="input" placeholder="Nouvelle collection" value={collName} onChange={(e) => setCollName(e.target.value)} style={{ maxWidth: 260 }} />
          <button className="btn primary" onClick={() => createMut.mutate()} disabled={!collName.trim() || createMut.isPending}>Créer</button>
        </div>
        {(collections || []).length === 0 ? (
          <EmptyState icon="📂" message="Aucune collection." />
        ) : (
          <ul style={{ paddingLeft: 20, fontSize: 13 }}>
            {(collections || []).map((c: any) => <li key={c.id}>{c.name}</li>)}
          </ul>
        )}
      </div>

      <div className="card" style={{ marginTop: 16 }}>
        <h3 style={{ marginTop: 0 }}>Jeux à redécouvrir</h3>
        {filteredMissing.length === 0 ? (
          <EmptyState icon="🎮" message="Rien à signaler." />
        ) : (
          <p style={{ color: "var(--text-dim)", fontSize: 13 }}>{formatNumber(filteredMissing.length)} jeu(x) avec fichiers manquants ou sans fichier.</p>
        )}
      </div>
    </>
  );
}
