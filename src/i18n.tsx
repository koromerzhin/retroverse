import React, { createContext, useContext, useEffect, useState } from "react";
import { api } from "./lib/api";

export type Lang = "fr" | "en" | "es" | "pt" | "de" | "it";

const SUPPORTED: Lang[] = ["fr", "en", "es", "pt", "de", "it"];

function detectLang(sysLocale: string): Lang {
  const code = (sysLocale || "").toLowerCase().slice(0, 2);
  if (SUPPORTED.includes(code as Lang)) return code as Lang;
  // navigator fallback
  const nav = (navigator.language || "").toLowerCase().slice(0, 2);
  if (SUPPORTED.includes(nav as Lang)) return nav as Lang;
  return "en";
}

// Translation dictionaries
import fr from "./locales/fr.json";
import en from "./locales/en.json";
import es from "./locales/es.json";
import pt from "./locales/pt.json";
import de from "./locales/de.json";
import it from "./locales/it.json";

const DICTS: Record<Lang, any> = { fr, en, es, pt, de, it };

interface Ctx {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: string) => string;
}

const I18nContext = createContext<Ctx>({ lang: "en", setLang: () => {}, t: (k) => k });

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>("en");

  useEffect(() => {
    (async () => {
      // load saved
      const saved = localStorage.getItem("retroverse.lang") as Lang | null;
      if (saved && SUPPORTED.includes(saved)) {
        setLangState(saved);
        return;
      }
      // detect system
      try {
        const sys = await api.getSystemLocale();
        setLangState(detectLang(sys));
      } catch {
        setLangState(detectLang(navigator.language));
      }
    })();
  }, []);

  const setLang = (l: Lang) => {
    setLangState(l);
    localStorage.setItem("retroverse.lang", l);
  };

  const t = (key: string): string => {
    const dict = DICTS[lang] || DICTS.en;
    return (dict && dict[key]) || (DICTS.en && DICTS.en[key]) || key;
  };

  return <I18nContext.Provider value={{ lang, setLang, t }}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}

export const LANGS: { code: Lang; label: string }[] = [
  { code: "fr", label: "Français" },
  { code: "en", label: "English" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "de", label: "Deutsch" },
  { code: "it", label: "Italiano" },
];