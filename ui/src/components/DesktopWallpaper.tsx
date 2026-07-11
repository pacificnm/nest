/**
 * Resolution-independent desktop wallpaper (SVG).
 * Replaces a raster asset so the shell stays sharp on 1080p, 4K, and ultrawide displays.
 */
export function DesktopWallpaper() {
  return (
    <div className="nest-desktop-wallpaper" aria-hidden="true">
      <svg
        className="nest-desktop-wallpaper__svg"
        viewBox="0 0 1920 1080"
        preserveAspectRatio="xMidYMid slice"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id="nest-bg-base" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#0a0e14" />
            <stop offset="42%" stopColor="#121820" />
            <stop offset="100%" stopColor="#1b2430" />
          </linearGradient>

          <linearGradient id="nest-bg-horizon" x1="0%" y1="100%" x2="0%" y2="0%">
            <stop offset="0%" stopColor="#0d1218" stopOpacity="0.95" />
            <stop offset="55%" stopColor="#141b24" stopOpacity="0.4" />
            <stop offset="100%" stopColor="#1b2430" stopOpacity="0" />
          </linearGradient>

          <radialGradient id="nest-glow-primary" cx="72%" cy="18%" r="55%">
            <stop offset="0%" stopColor="#4f8ef7" stopOpacity="0.38" />
            <stop offset="45%" stopColor="#3d6ec4" stopOpacity="0.12" />
            <stop offset="100%" stopColor="#1b2430" stopOpacity="0" />
          </radialGradient>

          <radialGradient id="nest-glow-secondary" cx="18%" cy="78%" r="48%">
            <stop offset="0%" stopColor="#58a6ff" stopOpacity="0.2" />
            <stop offset="55%" stopColor="#2f5f9e" stopOpacity="0.06" />
            <stop offset="100%" stopColor="#0d1218" stopOpacity="0" />
          </radialGradient>

          <radialGradient id="nest-glow-accent" cx="88%" cy="72%" r="35%">
            <stop offset="0%" stopColor="#6ea8fe" stopOpacity="0.16" />
            <stop offset="100%" stopColor="#0a0e14" stopOpacity="0" />
          </radialGradient>

          <linearGradient id="nest-wave-fill" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#4f8ef7" stopOpacity="0.08" />
            <stop offset="50%" stopColor="#6ea8fe" stopOpacity="0.14" />
            <stop offset="100%" stopColor="#3d6ec4" stopOpacity="0.05" />
          </linearGradient>

          <filter id="nest-soft-blur" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="48" />
          </filter>

          <filter id="nest-fine-grain" x="0%" y="0%" width="100%" height="100%">
            <feTurbulence
              type="fractalNoise"
              baseFrequency="0.85"
              numOctaves="4"
              stitchTiles="stitch"
              result="noise"
            />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0.04
                      0 0 0 0 0.06
                      0 0 0 0 0.1
                      0 0 0 0.035 0"
              in="noise"
              result="grain"
            />
            <feBlend in="SourceGraphic" in2="grain" mode="overlay" />
          </filter>
        </defs>

        <rect width="1920" height="1080" fill="url(#nest-bg-base)" />
        <rect width="1920" height="1080" fill="url(#nest-bg-horizon)" />
        <rect width="1920" height="1080" fill="url(#nest-glow-primary)" />
        <rect width="1920" height="1080" fill="url(#nest-glow-secondary)" />
        <rect width="1920" height="1080" fill="url(#nest-glow-accent)" />

        <g filter="url(#nest-soft-blur)" opacity="0.9">
          <ellipse cx="1380" cy="220" rx="520" ry="280" fill="#4f8ef7" fillOpacity="0.12" />
          <ellipse cx="340" cy="820" rx="440" ry="240" fill="#58a6ff" fillOpacity="0.09" />
        </g>

        <path
          d="M0 680 C 280 620, 420 760, 720 700 S 1180 580, 1520 640 S 1840 720, 1920 660 L 1920 1080 L 0 1080 Z"
          fill="url(#nest-wave-fill)"
          opacity="0.85"
        />
        <path
          d="M0 780 C 360 720, 520 860, 860 790 S 1320 700, 1680 760 S 1880 820, 1920 800 L 1920 1080 L 0 1080 Z"
          fill="#4f8ef7"
          fillOpacity="0.04"
        />

        <g opacity="0.35" stroke="#6ea8fe" strokeWidth="1" fill="none">
          <path d="M0 420 Q 480 360, 960 400 T 1920 380" opacity="0.25" />
          <path d="M0 520 Q 520 460, 1040 500 T 1920 470" opacity="0.15" />
        </g>

        <rect
          width="1920"
          height="1080"
          fill="url(#nest-bg-base)"
          fillOpacity="0.08"
          filter="url(#nest-fine-grain)"
          style={{ mixBlendMode: "overlay" }}
        />

        <rect width="1920" height="1080" fill="url(#nest-bg-horizon)" opacity="0.55" />
      </svg>
    </div>
  );
}
