import { useMemo, useState, type MouseEvent } from "react";

interface Props {
  /** Latency samples in ms, chronological (oldest first). */
  data: number[];
}

const WIDTH = 280;
const HEIGHT = 60;
const PAD_TOP = 8;
// Matches the emerald "connected" status dot elsewhere in the list, so the
// chart reads as the same signal rather than introducing a new color.
const LINE_COLOR = "#10b981";

export function LatencyChart({ data }: Props) {
  const [hoverIndex, setHoverIndex] = useState<number | null>(null);

  const chart = useMemo(() => {
    if (data.length < 2) return null;
    const max = Math.max(...data, 0.1) * 1.2;
    const usableHeight = HEIGHT - PAD_TOP;
    const step = WIDTH / (data.length - 1);
    const points = data.map((v, i) => ({
      x: i * step,
      y: PAD_TOP + usableHeight - (v / max) * usableHeight,
      v,
    }));
    const line = points.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(" ");
    const area = `${line} L ${WIDTH},${HEIGHT} L 0,${HEIGHT} Z`;
    return { points, line, area, step };
  }, [data]);

  if (!chart) {
    return (
      <div className="flex h-[60px] items-center justify-center text-[10px] text-muted-foreground">
        Collecting samples…
      </div>
    );
  }

  const { points, line, area, step } = chart;
  const latest = points[points.length - 1];
  const hovered = hoverIndex != null ? points[hoverIndex] : null;

  function handleMove(e: MouseEvent<SVGRectElement>) {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * WIDTH;
    setHoverIndex(Math.max(0, Math.min(points.length - 1, Math.round(x / step))));
  }

  return (
    <div className="relative">
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} className="w-full" style={{ height: HEIGHT }}>
        {[0, 0.5, 1].map((f) => (
          <line
            key={f}
            x1={0}
            x2={WIDTH}
            y1={PAD_TOP + (HEIGHT - PAD_TOP) * f}
            y2={PAD_TOP + (HEIGHT - PAD_TOP) * f}
            className="text-border"
            stroke="currentColor"
            strokeWidth={1}
            opacity={0.5}
          />
        ))}
        <path d={area} fill={LINE_COLOR} opacity={0.1} stroke="none" />
        <path d={line} fill="none" stroke={LINE_COLOR} strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" />
        <circle cx={latest.x} cy={latest.y} r={2.5} fill={LINE_COLOR} />
        {hovered && (
          <>
            <line
              x1={hovered.x}
              x2={hovered.x}
              y1={PAD_TOP}
              y2={HEIGHT}
              stroke={LINE_COLOR}
              strokeWidth={1}
              opacity={0.35}
            />
            <circle cx={hovered.x} cy={hovered.y} r={3} fill={LINE_COLOR} />
          </>
        )}
        <rect
          width={WIDTH}
          height={HEIGHT}
          fill="transparent"
          onMouseMove={handleMove}
          onMouseLeave={() => setHoverIndex(null)}
        />
      </svg>

      <span className="pointer-events-none absolute top-0 right-0 text-[9px] font-medium text-muted-foreground">
        {latest.v.toFixed(2)}ms
      </span>

      {hovered && (
        <div
          className="pointer-events-none absolute -top-[18px] rounded bg-popover px-1.5 py-0.5 text-[9px] text-popover-foreground shadow-sm ring-1 ring-border"
          style={{ left: `${(hovered.x / WIDTH) * 100}%`, transform: "translateX(-50%)" }}
        >
          {hovered.v.toFixed(2)}ms
        </div>
      )}
    </div>
  );
}
