import { commands } from "@/bindings";
import SongCard from "@/home/components/SongCard";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router-dom"

export default function Album() {
  const { albumId } = useParams();

  const { data: tracks } = useQuery({
    queryKey: [`tracks-${albumId}`],
    queryFn: async () => unwrap(await commands.albumTracks(albumId!)),
  });

  return (
      <div className="flex flex-wrap gap-5">
        {tracks?.map(track => <SongCard track={track} />)}
      </div>
  )
}
