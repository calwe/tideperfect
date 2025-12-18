import { commands } from "@/bindings"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useState } from "react"
import { redirect, useNavigate } from "react-router-dom"
import { useQueryClient } from "@tanstack/react-query"

export default function LoginCard() {
  const [code, setCode] = useState("");
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const handleLoginClick = async () => {
    const response = await commands.startAuthorization();
    if (response.status == 'ok') {
      setCode(response.data)
    }
  }

  const handleAuth = async () => {
    const response = await commands.authorize();
    if (response.status == 'ok') {
      queryClient.invalidateQueries({ queryKey: ['username'] });
      navigate("/")
    }
  }

  if (code == "") {
    return ( 
      <div>
        <Button onClick={handleLoginClick}>Login</Button>
      </div>
    )
  } else {
    return (
      <div className="flex flex-col items-center">
        <h1>If asked, use code {code}</h1>
        <Button onClick={handleAuth}>Click once logged in</Button>
      </div>
    )
  }
}

