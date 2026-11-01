// splits a parameter string into text and {{token}} segments, mirroring engine/template.rs
export type Segment =
  { kind: "text"; text: string } | { kind: "token"; token: string; raw: string };

const KNOWN = /^(input|item|index|now|output:[^{}]+|var:[^{}]+)$/;

export function parseTokens(s: string): Segment[] {
  const out: Segment[] = [];
  let rest = s;
  while (rest.length > 0) {
    const start = rest.indexOf("{{");
    if (start < 0) break;
    const end = rest.indexOf("}}", start + 2);
    if (end < 0) break;
    const token = rest.slice(start + 2, end).trim();
    if (start > 0) out.push({ kind: "text", text: rest.slice(0, start) });
    const raw = rest.slice(start, end + 2);
    out.push(KNOWN.test(token) ? { kind: "token", token, raw } : { kind: "text", text: raw });
    rest = rest.slice(end + 2);
  }
  if (rest) out.push({ kind: "text", text: rest });
  return mergeText(out);
}

function mergeText(segments: Segment[]): Segment[] {
  return segments.reduce<Segment[]>((acc, seg) => {
    const prev = acc[acc.length - 1];
    if (seg.kind === "text" && prev?.kind === "text") {
      acc[acc.length - 1] = { kind: "text", text: prev.text + seg.text };
    } else {
      acc.push(seg);
    }
    return acc;
  }, []);
}

export function isSingleToken(s: string) {
  const segs = parseTokens(s);
  return segs.length === 1 && segs[0]?.kind === "token";
}
