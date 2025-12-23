import { commands } from "@/bindings";
import { useQuery } from "@tanstack/react-query";
import { Navigate, useNavigate } from "react-router-dom";
import { isError, unwrap } from "@/lib/result";
import { Button } from "@/components/ui/button";
import SongCard from "./components/SongCard";
import AlbumCard from "./components/AlbumCard";
import DevicePicker from "./components/DevicePicker";

export default function Home() {
  const navigate = useNavigate();

  const songs = ['392563576', '60166839', '2570681', '391366623']

  const { data: loggedIn, isLoading, isFetching } = useQuery({
    queryKey: ['loggedIn'],
    queryFn: async () => {
      let res = unwrap(await commands.isLoggedIn());
      console.log(res);
      return res;
    }
  });

  const { data: albums } = useQuery({
    queryKey: ['albums'],
    queryFn: async () => {
      return unwrap(await commands.favouriteAlbums());
    }
  });

  const playQueue = async () => {
    await commands.play();
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <p>Loading...</p>
      </div>
    );
  }

  if (!loggedIn && !isFetching) {
    console.log("Navigating: ", loggedIn);
    return <Navigate to="/login" replace />;
  }

  return (
    <div className="m-5 flex flex-col gap-5">
      <div className="flex gap-5">
        <Button onClick={playQueue}>
          Play Queue
        </Button>
      </div>
      <div className="flex flex-wrap gap-5">
        {albums?.map(album => <AlbumCard id={album.item.id} title={album.item.title} quality={album.item.audioQuality} image={album.item.cover} />)}
      </div>
    </div>
  );
}
