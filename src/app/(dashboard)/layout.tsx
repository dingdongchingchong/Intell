import { AppShell } from "@/components/layout/app-shell";

export default function DashboardGroupLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <AppShell>{children}</AppShell>;
}
