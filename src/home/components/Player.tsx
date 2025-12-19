import { commands, events, Track } from "@/bindings";
import { Button } from "@/components/ui/button";
import { useEffect, useState } from "react";
import { Play, Pause, SkipForward, SkipBack, Volume2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";

export function Player() {
  const [currentTrack, setCurrentTrack] = useState<Track | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentPosition, setCurrentPosition] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    const unlistenTrack = events.currentTrackEvent.listen((event) => {
      setCurrentTrack(event.payload);
      setDuration(event.payload?.duration || 0);
      setCurrentPosition(0);
    });

    const unlistenState = events.playbackStateEvent.listen((event) => {
      setIsPlaying(event.payload);
    });

    const unlistenPosition = events.playbackPositionEvent.listen((event) => {
      setCurrentPosition(event.payload);
    });

    return () => {
      unlistenTrack.then((fn) => fn());
      unlistenState.then((fn) => fn());
      unlistenPosition.then((fn) => fn());
    };
  }, []);

  const togglePlayPause = async () => {
    if (isPlaying) {
      await commands.pause();
    } else {
      await commands.resume();
    }
  };

  const skipNext = async () => {
    await commands.skipNext();
  };

  // const skipPrevious = async () => {
  //   await commands.skipPrevious();
  // };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  if (!currentTrack) {
    return (
      <div className="border-t border-border p-4">
        <div className="flex items-center gap-3">
          <div className="w-16 h-16 bg-muted rounded shrink-0" />
          <div className="flex-1">
            <p className="text-sm text-muted-foreground">No track playing</p>
          </div>
        </div>
      </div>
    );
  }

  const progress = duration > 0 ? (currentPosition / duration) * 100 : 0;

  return (
    <div className="border-t border-border p-4 flex flex-col gap-3">
      <div className="flex items-center gap-3">
        <div className="w-16 h-16 bg-muted rounded shrink-0" />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{currentTrack.title}</p>
          <div className="flex items-center gap-2 mt-1">
            {currentTrack.explicit && (
              <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
                E
              </Badge>
            )}
            <span className="text-xs text-muted-foreground">
              {currentTrack.audioQuality}
            </span>
          </div>
        </div>
      </div>

      {/* Progress bar */}
      <div className="space-y-1">
        <div
          className="h-1 bg-muted rounded-full cursor-pointer group"
        >
          <div
            className="h-full bg-primary rounded-full transition-all group-hover:bg-primary/80"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>{formatTime(currentPosition)}</span>
          <span>{formatTime(duration)}</span>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center justify-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
        >
          <SkipBack className="h-4 w-4" />
        </Button>
        <Button
          variant="default"
          size="icon"
          className="h-9 w-9"
          onClick={togglePlayPause}
        >
          {isPlaying ? (
            <Pause className="h-5 w-5" fill="currentColor" />
          ) : (
            <Play className="h-5 w-5" fill="currentColor" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={skipNext}
        >
          <SkipForward className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
