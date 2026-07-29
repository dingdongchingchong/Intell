"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { signOut, useSession } from "next-auth/react";
import {
  LayoutDashboard,
  Briefcase,
  Columns3,
  Users,
  LogOut,
  Scale,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const nav = [
  { href: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { href: "/cases", label: "Cases", icon: Briefcase },
  { href: "/kanban", label: "Kanban", icon: Columns3 },
  { href: "/users", label: "Users", icon: Users, roles: ["admin"] },
];

export function AppShell({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const { data } = useSession();
  const role = (data?.user as { role?: string } | undefined)?.role;

  return (
    <div className="min-h-screen bg-slate-50 text-slate-900">
      <div className="flex min-h-screen">
        <aside className="hidden w-60 shrink-0 border-r border-slate-200 bg-white md:flex md:flex-col">
          <div className="flex items-center gap-2 border-b border-slate-200 px-5 py-5">
            <Scale className="h-6 w-6 text-blue-700" />
            <div>
              <div className="font-semibold tracking-tight">CaseFlow</div>
              <div className="text-xs text-slate-500">Investigation CMS</div>
            </div>
          </div>
          <nav className="flex flex-1 flex-col gap-1 p-3">
            {nav
              .filter((item) => !item.roles || (role && item.roles.includes(role)))
              .map((item) => {
                const Icon = item.icon;
                const active = pathname.startsWith(item.href);
                return (
                  <Link
                    key={item.href}
                    href={item.href}
                    className={cn(
                      "flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                      active
                        ? "bg-slate-900 text-white"
                        : "text-slate-600 hover:bg-slate-100 hover:text-slate-900"
                    )}
                  >
                    <Icon className="h-4 w-4" />
                    {item.label}
                  </Link>
                );
              })}
          </nav>
          <div className="border-t border-slate-200 p-3">
            <div className="mb-2 truncate px-2 text-xs text-slate-500">
              {data?.user?.name || data?.user?.email}
            </div>
            <Button
              variant="outline"
              size="sm"
              className="w-full"
              onClick={() => signOut({ callbackUrl: "/login" })}
            >
              <LogOut className="h-4 w-4" />
              Sign out
            </Button>
          </div>
        </aside>
        <main className="flex-1 overflow-auto">
          <div className="border-b border-slate-200 bg-white px-4 py-3 md:hidden">
            <div className="font-semibold">CaseFlow</div>
          </div>
          <div className="mx-auto max-w-7xl p-4 md:p-8">{children}</div>
        </main>
      </div>
    </div>
  );
}
