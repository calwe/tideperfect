import { commands, events, TrackDTO } from "@/bindings";
import { useEffect, useState } from "react";
import { unwrap, isOk } from "@/lib/result";
import { ScrollArea } from "@/components/ui/scroll-area";

export function Lyrics() {
  const [currentTrack, setCurrentTrack] = useState<TrackDTO | null>(null);
  const [lyrics, setLyrics] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    console.log("[Lyrics] Component mounted, setting up event listener");
    const unlistenTrack = events.updatedCurrentTrack.listen((event) => {
      console.log("[Lyrics] updatedCurrentTrack event received:", event.payload);
      setCurrentTrack(event.payload);
    });

    return () => {
      console.log("[Lyrics] Component unmounting, cleaning up listener");
      unlistenTrack.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    console.log("[Lyrics] fetchLyrics effect triggered, currentTrack:", currentTrack);
    const fetchLyrics = async () => {
      if (!currentTrack) {
        console.log("[Lyrics] No current track, skipping lyrics fetch");
        setLyrics(null);
        return;
      }

      console.log("[Lyrics] Fetching lyrics for track ID:", currentTrack.id);
      setIsLoading(true);
      try {
        const result = await commands.lyrics(currentTrack.id);
        console.log("[Lyrics] Lyrics command result:", result);
        if (isOk(result)) {
          console.log("[Lyrics] Lyrics fetched successfully:", result.data?.substring(0, 50) + "...");
          setLyrics(result.data);
        } else {
          console.log("[Lyrics] Lyrics command returned error:", result.error);
          setLyrics(null);
        }
      } catch (error) {
        console.error("[Lyrics] Failed to fetch lyrics:", error);
        setLyrics(null);
      } finally {
        setIsLoading(false);
      }
    };

    fetchLyrics();
  }, [currentTrack?.id]);

  if (!currentTrack) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <p className="text-sm text-muted-foreground">No track playing</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <p className="text-sm text-muted-foreground">Loading lyrics...</p>
      </div>
    );
  }

  if (!lyrics) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <p className="text-sm text-muted-foreground">No lyrics available</p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col border-l border-border">
      <div className="p-4 border-b border-border">
        <h2 className="text-sm font-semibold">Lyrics</h2>
        <p className="text-xs text-muted-foreground mt-1">
          {currentTrack.title}
        </p>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-4">
          <pre className="text-sm whitespace-pre-wrap font-sans leading-relaxed">
            {lyrics}
          </pre>
        </div>
      </ScrollArea>
    </div>
  );
}
