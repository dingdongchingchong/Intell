"use client";

import { FormEvent, useState } from "react";
import { signIn } from "next-auth/react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

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
    <div className="relative flex min-h-screen items-center justify-center overflow-hidden px-4">
      <div className="absolute inset-0 bg-gradient-to-br from-slate-950 via-blue-900 to-teal-800" />
      <div className="relative z-10 w-full max-w-md rounded-xl bg-white p-7 shadow-2xl">
        <div className="mb-2 flex items-center gap-3">
          <div className="grid h-10 w-10 place-items-center rounded-lg bg-gradient-to-br from-blue-600 to-blue-500 text-sm font-bold text-white">
            CF
          </div>
          <div>
            <h1 className="text-xl font-bold text-slate-900">CaseFlow</h1>
            <p className="text-xs text-slate-500">Investigation Manager</p>
          </div>
        </div>
        <p className="mb-5 text-sm text-slate-500">
          Sign in for the admin workspace, or{" "}
          <a href="/investigation.html" className="font-medium text-blue-600 underline">
            open Investigation Manager
          </a>
          .
        </p>
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-wide text-slate-500">
              Email or username
            </label>
            <Input
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="border-slate-200 bg-slate-50"
              autoComplete="username"
              required
            />
          </div>
          <div className="space-y-2">
            <label className="text-xs font-semibold uppercase tracking-wide text-slate-500">
              Password
            </label>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="border-slate-200 bg-slate-50"
              autoComplete="current-password"
              required
            />
          </div>
          {error && <p className="text-sm text-red-600">{error}</p>}
          <Button
            type="submit"
            className="w-full bg-blue-600 hover:bg-blue-500"
            disabled={loading}
          >
            {loading ? "Signing in…" : "Sign in"}
          </Button>
        </form>
      </div>
    </div>
  );
}
