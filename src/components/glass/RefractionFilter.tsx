// displacement map: neutral grey in the middle, red/green ramps at the edges, so only the
// border of a glass element bends the backdrop. stretched to each element by objectBoundingBox.
const map = `<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256">
<defs>
<linearGradient id="r" x1="0" x2="1" y1="0" y2="0">
<stop offset="0" stop-color="#000"/><stop offset=".14" stop-color="#800000"/>
<stop offset=".86" stop-color="#800000"/><stop offset="1" stop-color="#f00"/>
</linearGradient>
<linearGradient id="g" x1="0" x2="0" y1="0" y2="1">
<stop offset="0" stop-color="#000"/><stop offset=".2" stop-color="#008000"/>
<stop offset=".8" stop-color="#008000"/><stop offset="1" stop-color="#0f0"/>
</linearGradient>
</defs>
<rect width="256" height="256" fill="url(#r)"/>
<rect width="256" height="256" fill="url(#g)" style="mix-blend-mode:screen"/>
</svg>`;

const href = `data:image/svg+xml,${encodeURIComponent(map)}`;

export function RefractionFilter() {
  return (
    <svg width="0" height="0" aria-hidden style={{ position: "absolute" }}>
      <filter
        id="sc-refract"
        x="0"
        y="0"
        width="1"
        height="1"
        filterUnits="objectBoundingBox"
        primitiveUnits="objectBoundingBox"
        colorInterpolationFilters="sRGB"
      >
        <feImage
          href={href}
          x="0"
          y="0"
          width="1"
          height="1"
          preserveAspectRatio="none"
          result="map"
        />
        <feDisplacementMap
          in="SourceGraphic"
          in2="map"
          scale="0.06"
          xChannelSelector="R"
          yChannelSelector="G"
        />
      </filter>
    </svg>
  );
}
