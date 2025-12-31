import { commands, events } from "@/bindings"
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
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const handleLoginClick = async () => {
    await commands.login();
  }

  events.loggedIn.listen((_) => {
    console.log("logged in, redirecting");
    queryClient.invalidateQueries({ queryKey: ["loggedIn"] });
    navigate('/');
  })

  return ( 
    <div className="h-screen flex justify-center items-center">
      <Button onClick={handleLoginClick}>Login</Button>
    </div>
  )
}

