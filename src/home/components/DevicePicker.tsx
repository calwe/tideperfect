import { commands } from "@/bindings";
import { Select, SelectItem, SelectLabel, SelectContent, SelectGroup, SelectTrigger, SelectValue } from "@/components/ui/select";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

export default function DevicePicker() {
  const [selectedValue, setSelectedValue] = useState<string | undefined>();

  const { data: devices, isLoading } = useQuery({
    queryKey: ["devices"],
    queryFn: async () => {
      const result = unwrap(await commands.devices());

      if (result.length > 0 && !selectedValue) {
        const initial = result[0];
        setSelectedValue(initial);
      }
      return result;
    },
  });

  const handleValueChange = async (value: string) => {
    setSelectedValue(value);
    await commands.setOutputDevice(value);
  };

  if (isLoading || !devices) {
    return (
      <Select disabled>
        <SelectTrigger className="w-70">
          <SelectValue placeholder="Loading devices..." />
        </SelectTrigger>
      </Select>
    )
  }

  return (
    <Select value={selectedValue} onValueChange={handleValueChange} >
      <SelectTrigger className="w-70">
        <SelectValue placeholder="Select an output device" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>Devices</SelectLabel>
          {devices?.map((device) => (
            <SelectItem key={device} value={device}>
              {device}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  )
}
