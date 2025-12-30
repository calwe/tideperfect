import { Outlet } from "react-router-dom";
import { Queue } from "./home/components/Queue";
import { Lyrics } from "./home/components/Lyrics";

export default function Layout() {
  return (
    <div className="h-screen flex">
      <div className="flex-1 overflow-y-auto">
        <Outlet />
      </div>
      <div className="w-80 flex-shrink-0">
        <Queue />
      </div>
      <div className="w-80 flex-shrink-0">
        <Lyrics />
      </div>
    </div>
  );
}
