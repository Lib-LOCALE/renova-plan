import { EN } from './i18n-en';

export type Lang = 'fr' | 'en';

function initial(): Lang {
  try {
    const saved = localStorage.getItem('renovaplan.lang');
    if (saved === 'fr' || saved === 'en') return saved;
    return navigator.language.toLowerCase().startsWith('fr') ? 'fr' : 'en';
  } catch {
    return 'fr';
  }
}

class I18n {
  lang = $state<Lang>(initial());
  toggle() {
    this.lang = this.lang === 'fr' ? 'en' : 'fr';
    try {
      localStorage.setItem('renovaplan.lang', this.lang);
    } catch {
      /* préférence non conservée */
    }
  }
}

export const i18n = new I18n();

/** Le français sert de clé : un texte sans traduction reste lisible. */
export const t = (s: string): string => (i18n.lang === 'en' ? (EN[s] ?? s) : s);
