import { commands } from "@/bindings";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import QualityBadge from "./QualityBadge";
import { useNavigate } from "react-router-dom";

interface AlbumCardProps {
  id: string,
  title: string,
  quality: string,
  image: string,
}

export default function AlbumCard({ id, title, quality, image }: AlbumCardProps) {
  const navigate = useNavigate();

  const onClick = async () => {
    navigate(`/album/${id}`);
  }

  const imageUrl = (identifier: string) => {
    const path = identifier.replace(/-/g, '/');
    return `https://resources.tidal.com/images/${path}/320x320.jpg`
  }

  return (
    <Card className="p-4 flex flex-col gap-4 items-center" onClick={onClick} >
      <h1>{title} ({id})</h1>
      <div className="w-36 h-36 rounded">
        <img className="object-stretch rounded" src={imageUrl(image)} />
      </div>
      <div className="flex justify-around">
        <QualityBadge quality={quality} />
      </div>
    </Card>
  )
}
