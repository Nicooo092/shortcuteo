import { useEffect, useState, type CSSProperties } from "react";

import { blobs } from "../../theme/tokens";

export function Ambient({ animate }: { animate: boolean }) {
  const [hidden, setHidden] = useState(document.hidden);

  // webview2 reports hidden when the window is minimized or covered by the tray
  useEffect(() => {
    const update = () => setHidden(document.hidden);
    document.addEventListener("visibilitychange", update);
    return () => document.removeEventListener("visibilitychange", update);
  }, []);

  const cls = `ambient${hidden ? " is-paused" : ""}${animate ? "" : " is-static"}`;
  return (
    <div className={cls} aria-hidden>
      {blobs.map((b, i) => (
        <div
          key={i}
          className="ambient__blob"
          style={
            {
              left: b.x,
              top: b.y,
              width: b.size,
              height: b.size,
              background: b.color,
              opacity: b.opacity,
              filter: `blur(${b.blur}px)`,
              "--dur": `${b.duration}s`,
              animationDelay: `${-i * 4}s`,
            } as CSSProperties
          }
        />
      ))}
    </div>
  );
}
