import type { Tile } from "../bindings/Tile";

export const color = {
  night: "#0A0A18",
  ink: "#F5F6FB",
  inkSecondary: "rgba(245, 246, 251, 0.73)",
  inkTertiary: "rgba(245, 246, 251, 0.5)",
  accent: "#5B8CFF",
  success: "#2FD88A",
  danger: "#FF5A6E",
  closeHover: "#C42B1C",
};

export const blobs = [
  { color: "#3A55FF", size: 620, blur: 90, opacity: 0.55, duration: 22, x: "8%", y: "4%" },
  { color: "#A53BFF", size: 540, blur: 85, opacity: 0.45, duration: 27, x: "62%", y: "12%" },
  { color: "#FF3D86", size: 460, blur: 80, opacity: 0.32, duration: 19, x: "70%", y: "64%" },
  { color: "#13C9BD", size: 500, blur: 88, opacity: 0.38, duration: 24, x: "14%", y: "62%" },
  { color: "#FF9B3D", size: 360, blur: 80, opacity: 0.3, duration: 18, x: "42%", y: "40%" },
];

export const glass = {
  thin: {
    blur: 16,
    saturate: 160,
    background: "rgba(255, 255, 255, 0.06)",
    border: "rgba(255, 255, 255, 0.12)",
    highlight: "inset 0 1px 0 rgba(255, 255, 255, 0.18)",
  },
  regular: {
    blur: 28,
    saturate: 180,
    background: "linear-gradient(180deg, rgba(255, 255, 255, 0.14), rgba(255, 255, 255, 0.04))",
    border: "rgba(255, 255, 255, 0.15)",
    highlight: "inset 0 1px 0 rgba(255, 255, 255, 0.32)",
  },
  thick: {
    blur: 44,
    saturate: 190,
    background: "linear-gradient(180deg, rgba(255, 255, 255, 0.22), rgba(255, 255, 255, 0.08))",
    border: "rgba(255, 255, 255, 0.2)",
    highlight:
      "inset 0 1px 0 rgba(255, 255, 255, 0.5), inset 0 0 24px rgba(255, 255, 255, 0.08), 0 24px 60px rgba(0, 0, 0, 0.45)",
  },
} as const;

export type GlassVariant = keyof typeof glass;

export const radius = { key: 7, field: 12, tile: 14, block: 20, card: 24, pill: 999 };

export const space = [4, 8, 12, 16, 24, 32, 48] as const;

export const font = {
  sans: '"Geist Variable", "Segoe UI Variable", "Segoe UI", system-ui, sans-serif',
  mono: '"Geist Mono Variable", "Cascadia Mono", Consolas, monospace',
};

export const tiles: Record<Tile, { from: string; to: string }> = {
  iris: { from: "#5B8CFF", to: "#8B5CFF" },
  ember: { from: "#FF4F8B", to: "#FF8A3D" },
  lagoon: { from: "#14C8B8", to: "#3AD77A" },
  amber: { from: "#FF9A3D", to: "#FFCC4D" },
  orchid: { from: "#4E4BFF", to: "#B04DFF" },
  glacier: { from: "#22B8FF", to: "#4F7DFF" },
  slate: { from: "rgba(160, 170, 200, 0.42)", to: "rgba(110, 118, 150, 0.32)" },
};

export function tileGradient(tile: Tile) {
  const t = tiles[tile];
  return `linear-gradient(135deg, ${t.from}, ${t.to})`;
}

export const motion = {
  spring: { type: "spring", stiffness: 300, damping: 22 } as const,
  springSoft: { type: "spring", stiffness: 220, damping: 26 } as const,
  springEase: "cubic-bezier(.34, 1.56, .64, 1)",
  press: 0.97,
  hoverScale: 1.02,
  hoverLift: -5,
};

// one place to turn tokens into css variables, so css files never hardcode values
export function applyTokens(root: HTMLElement, opts: { accent?: string; blurScale?: number } = {}) {
  const s = root.style;
  const blurScale = opts.blurScale ?? 1;
  s.setProperty("--night", color.night);
  s.setProperty("--ink", color.ink);
  s.setProperty("--ink-2", color.inkSecondary);
  s.setProperty("--ink-3", color.inkTertiary);
  s.setProperty("--accent", opts.accent ?? color.accent);
  s.setProperty("--success", color.success);
  s.setProperty("--danger", color.danger);
  s.setProperty("--close-hover", color.closeHover);
  for (const [name, g] of Object.entries(glass)) {
    s.setProperty(`--glass-${name}-blur`, `${Math.round(g.blur * blurScale)}px`);
    s.setProperty(`--glass-${name}-saturate`, `${g.saturate}%`);
    s.setProperty(`--glass-${name}-bg`, g.background);
    s.setProperty(`--glass-${name}-border`, g.border);
    s.setProperty(`--glass-${name}-highlight`, g.highlight);
  }
  for (const [name, r] of Object.entries(radius)) {
    s.setProperty(`--r-${name}`, name === "pill" ? "999px" : `${r}px`);
  }
  space.forEach((v, i) => s.setProperty(`--s-${i + 1}`, `${v}px`));
  s.setProperty("--font-sans", font.sans);
  s.setProperty("--font-mono", font.mono);
  s.setProperty("--ease-spring", motion.springEase);
  for (const [name, t] of Object.entries(tiles)) {
    s.setProperty(`--tile-${name}`, `linear-gradient(135deg, ${t.from}, ${t.to})`);
  }
}
