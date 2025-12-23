import { commands, TrackDTO } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import QualityBadge from "./QualityBadge";

interface SongCardProps {
  track: TrackDTO,
}

export default function SongCard({ track }: SongCardProps) {
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
        <Button onClick={queue}>Queue</Button>
      </div>
    </Card>
  )
}
