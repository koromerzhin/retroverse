import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { useI18n } from "../i18n";

export default function Duplicates() {
  const { t } = useI18n();
  const { data } = useQuery({ queryKey: ["duplicates"], queryFn: () => api.findDuplicates() });

  return (
    <>
      <h1 className="page-title">{t("nav.duplicates")}</h1>
      <div className="card" style={{ marginTop: 16 }}>
        {(data || []).length === 0 ? (
          <p style={{ color: "var(--text-muted)" }}>{t("duplicates.none")}</p>
        ) : (
          <ul style={{ fontSize: 13, color: "var(--text-dim)", paddingLeft: 16 }}>
            {(data || []).map((g: any, i: number) => (
              <li key={i} style={{ marginBottom: 8 }}>
                <strong>{g.title}</strong> ({g.platform_id ?? "—"}) — {(g.games || []).length} entrées
                <ul style={{ fontSize: 12, color: "var(--text-muted)", paddingLeft: 16 }}>
                  {(g.games || []).map((gg: any) => <li key={gg[0]}>{gg[1]} <span style={{ fontFamily: "monospace", fontSize: 10 }}>({gg[0]})</span></li>)}
                </ul>
              </li>
            ))}
          </ul>
        )}
        <p style={{ color: "var(--text-dim)", fontSize: 12, marginTop: 8 }}>{(data || []).length} {t("duplicates.groups")}</p>
      </div>
    </>
  );
}
