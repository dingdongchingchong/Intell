"use client";

import { FormEvent, useEffect, useState } from "react";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { browserApi } from "@/lib/api";
import type { UserPublic } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function UsersPage() {
  const { data } = useSession();
  const role = (data?.user as { role?: string } | undefined)?.role;
  const token = (data as { accessToken?: string } | null)?.accessToken;
  const [users, setUsers] = useState<UserPublic[]>([]);
  const [form, setForm] = useState({
    name: "",
    email: "",
    username: "",
    password: "",
    role: "investigator",
  });

  async function load() {
    try {
      const res = await browserApi<{ users: UserPublic[] }>("/users", token);
      setUsers(res.users);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to load users");
    }
  }

  useEffect(() => {
    if (token && role === "admin") load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, role]);

  async function onCreate(e: FormEvent) {
    e.preventDefault();
    try {
      await browserApi("/users", token, {
        method: "POST",
        body: JSON.stringify(form),
      });
      toast.success("User created");
      setForm({
        name: "",
        email: "",
        username: "",
        password: "",
        role: "investigator",
      });
      load();
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Create failed");
    }
  }

  if (role && role !== "admin") {
    return (
      <div className="rounded-lg border border-amber-200 bg-amber-50 p-4 text-amber-900">
        Admin access required.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-3xl tracking-tight">Users</h1>
        <p className="text-slate-500">Manage CaseFlow accounts</p>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Team</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {users.map((u) => (
              <div
                key={u.id}
                className="flex items-center justify-between rounded-lg border border-slate-100 px-3 py-2 text-sm"
              >
                <div>
                  <div className="font-medium">{u.name}</div>
                  <div className="text-slate-500">
                    @{u.username} · {u.email}
                  </div>
                </div>
                <span className="rounded-full bg-slate-100 px-2 py-0.5 text-xs capitalize">
                  {u.role}
                </span>
              </div>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Add user</CardTitle>
          </CardHeader>
          <CardContent>
            <form onSubmit={onCreate} className="space-y-3">
              <Input
                placeholder="Full name"
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                required
              />
              <Input
                placeholder="Email"
                type="email"
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
                required
              />
              <Input
                placeholder="Username"
                value={form.username}
                onChange={(e) => setForm({ ...form, username: e.target.value })}
                required
              />
              <Input
                placeholder="Password (min 8)"
                type="password"
                value={form.password}
                onChange={(e) => setForm({ ...form, password: e.target.value })}
                required
                minLength={8}
              />
              <select
                className="h-10 w-full rounded-md border border-slate-200 px-3 text-sm"
                value={form.role}
                onChange={(e) => setForm({ ...form, role: e.target.value })}
              >
                <option value="admin">Admin</option>
                <option value="manager">Manager</option>
                <option value="investigator">Investigator</option>
                <option value="viewer">Viewer</option>
              </select>
              <Button type="submit" className="w-full">
                Create user
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
