import { commands, Track } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import QualityBadge from "./QualityBadge";

interface SongCardProps {
  track: Track,
}

export default function SongCard({ track }: SongCardProps) {
  const onClick = async () => {
    await commands.playTrack(track?.id);
  }

  const queue = async () => {
    await commands.queueTrack(track?.id);
  }

  return (
    <Card className="p-4 flex flex-col gap-4">
      <h1>{track?.title} ({track?.id})</h1>
      <div className="flex justify-around">
        {track?.mediaMetadata?.tags?.map((tag) => <QualityBadge quality={tag} />)}
      </div>
      <div className="flex justify-center gap-4">
        <Button onClick={onClick}>Play</Button>
        <Button onClick={queue}>Queue</Button>
      </div>
    </Card>
  )
}
