import { forwardRef, useCallback, useRef, type HTMLAttributes, type PointerEvent } from "react";

import type { GlassVariant } from "../../theme/tokens";
import "./glass.css";

interface GlassProps extends HTMLAttributes<HTMLDivElement> {
  variant?: GlassVariant;
  refract?: boolean;
}

export const Glass = forwardRef<HTMLDivElement, GlassProps>(function Glass(
  { variant = "regular", refract = false, className = "", onPointerMove, onPointerLeave, ...rest },
  ref,
) {
  const frame = useRef(0);

  // the specular spot follows the pointer; one write per frame, css does the rest
  const move = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      onPointerMove?.(e);
      if (!refract) return;
      const el = e.currentTarget;
      const { clientX, clientY } = e;
      cancelAnimationFrame(frame.current);
      frame.current = requestAnimationFrame(() => {
        const r = el.getBoundingClientRect();
        el.style.setProperty("--mx", `${((clientX - r.left) / r.width) * 100}%`);
        el.style.setProperty("--my", `${((clientY - r.top) / r.height) * 100}%`);
      });
    },
    [onPointerMove, refract],
  );

  const leave = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      onPointerLeave?.(e);
      e.currentTarget.style.removeProperty("--mx");
      e.currentTarget.style.removeProperty("--my");
    },
    [onPointerLeave],
  );

  return (
    <div
      ref={ref}
      className={`glass glass--${variant}${refract ? " glass--refract" : ""} ${className}`}
      onPointerMove={move}
      onPointerLeave={leave}
      {...rest}
    />
  );
});
