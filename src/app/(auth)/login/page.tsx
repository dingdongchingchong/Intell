"use client";

import { FormEvent, useState } from "react";
import { signIn } from "next-auth/react";
import { useRouter } from "next/navigation";
import { Scale } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("admin");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setLoading(true);
    setError("");
    const res = await signIn("credentials", {
      email,
      password,
      redirect: false,
    });
    setLoading(false);
    if (res?.error) {
      // NextAuth maps thrown authorize() errors to CredentialsSignin unless we surface message
      setError(
        res.error === "CredentialsSignin"
          ? "Invalid credentials, or API is down (run npm run dev:api)"
          : res.error
      );
      return;
    }
    router.push("/investigation.html");
    router.refresh();
  }

  return (
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden bg-slate-950 px-4">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,#1d4ed8_0%,transparent_40%),radial-gradient(circle_at_80%_0%,#0f766e_0%,transparent_35%)] opacity-60" />
      <Card className="relative z-10 w-full max-w-md border-slate-800 bg-slate-900/90 text-white shadow-2xl">
        <CardHeader className="space-y-3">
          <div className="flex items-center gap-2 text-blue-300">
            <Scale className="h-7 w-7" />
            <span className="font-display text-2xl tracking-tight text-white">
              CaseFlow
            </span>
          </div>
          <CardTitle className="text-base font-normal text-slate-300">
            Sign in for admin tools, or{" "}
            <a href="/investigation.html" className="text-blue-300 underline">
              open Investigation Manager
            </a>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2">
              <label className="text-xs uppercase tracking-wide text-slate-400">
                Email or username
              </label>
              <Input
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="border-slate-700 bg-slate-950 text-white"
                autoComplete="username"
                required
              />
            </div>
            <div className="space-y-2">
              <label className="text-xs uppercase tracking-wide text-slate-400">
                Password
              </label>
              <Input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="border-slate-700 bg-slate-950 text-white"
                autoComplete="current-password"
                required
              />
            </div>
            {error && <p className="text-sm text-red-400">{error}</p>}
            <Button
              type="submit"
              className="w-full bg-blue-600 hover:bg-blue-500"
              disabled={loading}
            >
              {loading ? "Signing in…" : "Sign in"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
