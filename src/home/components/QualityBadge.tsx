import { Badge } from "@/components/ui/badge";

interface QualityBadgeProps {
  quality: string,
}

export default function QualityBadge({ quality }: QualityBadgeProps) {
  switch (quality) {
    case "LOSSLESS":
      return <Badge className="bg-teal-500">HIGH</Badge>
    case "HIRES_LOSSLESS":
      return <Badge className="bg-amber-300">MAX</Badge>
    default:
      return <Badge>Other</Badge>
  }
}
