import { useEffect, useState, type CSSProperties } from "react";

import { blobs } from "../../theme/tokens";

// every moving pixel behind the glass forces all backdrop blurs to repaint, so the drift only
// runs while someone is actually looking at and using the window
const IDLE_MS = 20_000;

function useAwake() {
  const [awake, setAwake] = useState(!document.hidden && document.hasFocus());

  useEffect(() => {
    let timer = 0;
    const sleep = () => setAwake(false);
    const wake = () => {
      window.clearTimeout(timer);
      if (document.hidden || !document.hasFocus()) return sleep();
      setAwake(true);
      timer = window.setTimeout(sleep, IDLE_MS);
    };
    const events = ["pointermove", "pointerdown", "keydown", "wheel", "focus"] as const;
    events.forEach((e) => window.addEventListener(e, wake, { passive: true }));
    window.addEventListener("blur", sleep);
    document.addEventListener("visibilitychange", wake);
    wake();
    return () => {
      window.clearTimeout(timer);
      events.forEach((e) => window.removeEventListener(e, wake));
      window.removeEventListener("blur", sleep);
      document.removeEventListener("visibilitychange", wake);
    };
  }, []);

  return awake;
}

export function Ambient({ animate }: { animate: boolean }) {
  const awake = useAwake();
  const cls = `ambient${awake ? "" : " is-paused"}${animate ? "" : " is-static"}`;
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
              width: b.size + b.blur * 2,
              height: b.size + b.blur * 2,
              // a radial gradient looks like a blurred disc without a per-frame blur filter
              background: `radial-gradient(closest-side, ${b.color}, transparent)`,
              opacity: b.opacity,
              "--dur": `${b.duration}s`,
              animationDelay: `${-i * 4}s`,
            } as CSSProperties
          }
        />
      ))}
    </div>
  );
}
