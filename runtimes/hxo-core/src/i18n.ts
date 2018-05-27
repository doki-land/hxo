import { createSignal, type Signal } from "./reactivity";

export interface I18nMessages {
    [key: string]: string | I18nMessages;
}

export interface I18nData {
    [locale: string]: I18nMessages;
}

export interface I18nInstance {
    locale: Signal<string>;
    t: (key: string) => string;
}

const currentLocale = createSignal("en");

export function setLocale(locale: string) {
    currentLocale.set(locale);
}

export function useI18n(data?: I18nData): I18nInstance {
    const t = (key: string): string => {
        const locale = currentLocale.get();
        const messages = data?.[locale] || {};
        return (messages[key] as string) || key;
    };

    return {
        locale: currentLocale,
        t,
    };
}
