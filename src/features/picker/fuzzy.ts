// small subsequence matcher: enough for ~200 actions, no dependency, accent-insensitive
function fold(s: string) {
  return s.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
}

export function fuzzyScore(query: string, target: string): number {
  const q = fold(query.trim());
  if (!q) return 1;
  const t = fold(target);
  const direct = t.indexOf(q);
  if (direct >= 0) {
    // whole substring: prefer matches at the start of the text or of a word
    const atWord = direct === 0 || t[direct - 1] === " ";
    return 100 - direct + (atWord ? 50 : 0);
  }
  let score = 0;
  let ti = 0;
  let streak = 0;
  for (const ch of q) {
    if (ch === " ") continue;
    const found = t.indexOf(ch, ti);
    if (found < 0) return 0;
    streak = found === ti ? streak + 1 : 0;
    score += 1 + streak * 2 + (found === 0 || t[found - 1] === " " ? 3 : 0);
    ti = found + 1;
  }
  return score;
}

// long texts like descriptions only count on a plain substring hit, scattered letters there are noise
export function substringScore(query: string, target: string): number {
  const q = fold(query.trim());
  return q && fold(target).includes(q) ? 20 : 0;
}

export function fuzzyFilter<T>(
  items: T[],
  query: string,
  fields: (item: T) => string[],
  extra: (item: T) => string[] = () => [],
): T[] {
  if (!query.trim()) return items;
  return items
    .map((item) => ({
      item,
      score: Math.max(
        ...fields(item).map((f) => fuzzyScore(query, f)),
        ...extra(item).map((f) => substringScore(query, f)),
      ),
    }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.item);
}
