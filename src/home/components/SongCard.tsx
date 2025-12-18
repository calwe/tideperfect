import { commands } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import QualityBadge from "./QualityBadge";

interface SongCardProps {
  songId: string,
}

export default function SongCard({ songId }: SongCardProps) {
  const { data: track } = useQuery({
    queryKey: [`track-${songId}`],
    queryFn: async () => unwrap(await commands.fetchTrack(songId)),
  });

  const onClick = async () => {
    await commands.playTrack(songId);
  }

  return (
    <Card className="p-4 flex flex-col gap-4">
      <h1>{track?.title} ({songId})</h1>
      <div className="flex justify-around">
        {track?.mediaMetadata?.tags?.map((tag) => <QualityBadge quality={tag} />)}
      </div>
      <Button onClick={onClick}>Play</Button>
    </Card>
  )
}
