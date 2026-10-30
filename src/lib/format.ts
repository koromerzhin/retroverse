export function formatNumber(value: number | null | undefined): string {
  if (value == null) return "0";
  try {
    return new Intl.NumberFormat(undefined, { useGrouping: true, maximumFractionDigits: 0 }).format(value);
  } catch {
    return String(value);
  }
}

export function formatBytes(value: number | null | undefined): string {
  if (value == null) return "0";
  if (value < 1024) return `${value} o`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} Ko`;
  if (value < 1024 * 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(1)} Mo`;
  return `${(value / (1024 * 1024 * 1024)).toFixed(2)} Go`;
}