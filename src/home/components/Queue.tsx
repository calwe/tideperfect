import { events, Track } from "@/bindings";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useEffect, useState } from "react";
import { QueueItem } from "./QueueItem";
import { Player } from "./Player";

export function Queue() {
  const [queue, setQueue] = useState<Track[]>([]);

  useEffect(() => {
    const unlisten = events.updatedQueueEvent.listen((event) => {
      setQueue(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex flex-col h-full border-l border-border">
      <div className="px-4 py-3 border-b border-border">
        <h2 className="text-lg font-semibold">Queue</h2>
        <p className="text-xs text-muted-foreground">
          {queue.length} {queue.length === 1 ? "track" : "tracks"}
        </p>
      </div>

      <ScrollArea className="flex-1">
        {queue.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-muted-foreground text-sm">
            Queue is empty
          </div>
        ) : (
          <div>
            {queue.map((track, index) => (
              <QueueItem key={`${track.id}-${index}`} track={track} index={index} />
            ))}
          </div>
        )}
      </ScrollArea>

      <Player />
    </div>
  );
}
