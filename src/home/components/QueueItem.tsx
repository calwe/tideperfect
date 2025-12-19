import type { Track } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import QualityBadge from "./QualityBadge";

interface QueueItemProps {
  track: Track;
  index: number;
}

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function QueueItem({ track, index }: QueueItemProps) {
  return (
    <div className="flex items-center gap-3 px-3 py-2 hover:bg-accent/50 transition-colors border-b border-border/50">
      <span className="text-xs text-muted-foreground w-6 text-right shrink-0">
        {index + 1}
      </span>

      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">
          {track.title}
        </p>
        <div className="flex items-center gap-2 mt-0.5">
          {track.explicit && (
            <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
              E
            </Badge>
          )}
          <QualityBadge quality={track.audioQuality} />
        </div>
      </div>

      <span className="text-xs text-muted-foreground shrink-0">
        {formatDuration(track.duration)}
      </span>
    </div>
  );
}
