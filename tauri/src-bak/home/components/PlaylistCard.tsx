import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useNavigate } from "react-router-dom";

interface PlaylistCardProps {
  id: string,
  title: string,
  image: string,
}

export default function PlaylistCard({ id, title, image }: PlaylistCardProps) {
  const navigate = useNavigate();

  const open = async () => {
    navigate(`/playlist/${id}`);
  }

  const imageUrl = (identifier: string) => {
    const path = identifier.replace(/-/g, '/');
    return `https://resources.tidal.com/images/${path}/320x320.jpg`
  }

  return (
    <Card className="p-4 flex flex-col gap-4 items-center" >
      <h1>{title} ({id})</h1>
      <div className="w-36 h-36 rounded">
        <img className="object-stretch rounded" src={imageUrl(image)} />
      </div>
      <Button onClick={open}>View Tracks</Button>
    </Card>
  )
}
