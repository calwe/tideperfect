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
}

export default function AlbumCard({ id, title, quality }: AlbumCardProps) {
  const navigate = useNavigate();

  const onClick = async () => {
    navigate(`/album/${id}`);
  }

  return (
    <Card className="p-4 flex flex-col gap-4" onClick={onClick} >
      <h1>{title} ({id})</h1>
      <div className="flex justify-around">
        <QualityBadge quality={quality} />
      </div>
    </Card>
  )
}
