import { commands, events } from "@/bindings";
import { Dialog, DialogContent, DialogTitle, DialogDescription, DialogHeader } from "@/components/Dialog";
import { unwrap } from "@/lib/result";
import { useQuery } from "@tanstack/react-query";
import { openUrl } from '@tauri-apps/plugin-opener'
import QRCode from "react-qr-code";

function AuthModal() {
  const { data: loggedIn, isLoading, refetch } = useQuery({
    queryKey: ['loggedIn'],
    queryFn: async () => {
      return unwrap(await commands.isLoggedIn());
    }
  });

  const { data: userCode } = useQuery({
    queryKey: ['userCode'],
    queryFn: async () => {
      return unwrap(await commands.login());
    },
    enabled: loggedIn === false,
  })

  events.loggedIn.listen((_) => refetch());

  const openLink = async () => {
    await openUrl(`https://link.tidal.com/${userCode}`);
  }

  if (isLoading) return;

  return (
    <Dialog open={!loggedIn}>
      <DialogContent showCloseButton={false}>
        <DialogHeader>
          <DialogTitle>Not logged in!</DialogTitle>
          <DialogDescription>
            Visit <span onClick={openLink} className="underline cursor-pointer">https://link.tidal.com/{userCode}</span> to
            login, or scan the QR code.
          </DialogDescription>
        </DialogHeader>
        <div className="flex justify-center">
          <QRCode className="border-3" size={196} value={`https://link.tidal.com/${userCode}`} />
        </div>
      </DialogContent>
    </Dialog>
  )
}

export default AuthModal;
