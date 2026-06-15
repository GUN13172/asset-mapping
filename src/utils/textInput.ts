export const normalizeSmartPunctuation = (value: string) =>
  value
    .replace(/[“”„‟＂]/g, '"')
    .replace(/[‘’‚‛＇]/g, "'");
