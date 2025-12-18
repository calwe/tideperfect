import { commands } from "@/bindings";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { isError, unwrap } from "@/lib/result";
import { Button } from "@/components/ui/button";
import SongCard from "./components/SongCard";
import AlbumCard from "./components/AlbumCard";
import DevicePicker from "./components/DevicePicker";

export default function Home() {
  const navigate = useNavigate();

  const songs = ['392563576', '60166839', '2570681', '391366623']

  const { data: usernameResult, isLoading } = useQuery({
    queryKey: ['username'],
    queryFn: commands.getUsername,
  });

  const { data: albums } = useQuery({
    queryKey: ['albums'],
    queryFn: async () => {
      return unwrap(await commands.favouriteAlbums());
    }
  });

  const stopPlayback = async () => {
    await commands.stopTrack();
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <p>Loading...</p>
      </div>
    );
  }

  if (!usernameResult || isError(usernameResult)) {
    navigate('/login', { replace: true });
    return null;
  }

  return (
    <div className="h-screen m-5 flex flex-col gap-5">
      <div className="flex gap-5">
        <DevicePicker />
        <Button onClick={stopPlayback}>
          Stop Playback
        </Button>
      </div>
      <div className="flex flex-wrap gap-5">
        {albums?.map(album => <AlbumCard id={album.id} title={album.title} quality={album.quality}/>)}
      </div>
    </div>
  );
}
