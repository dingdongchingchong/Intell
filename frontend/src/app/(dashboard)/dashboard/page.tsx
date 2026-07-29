import { getServerSession } from "next-auth";
import { redirect } from "next/navigation";
import { authOptions } from "@/lib/auth-options";
import { dashboardAPI } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { stageLabel } from "@/lib/utils";

export default async function DashboardPage() {
  const session = await getServerSession(authOptions);
  if (!session) redirect("/login");

  let stats = null;
  let error = "";
  try {
    const res = await dashboardAPI.stats();
    stats = res.stats;
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load stats";
  }

  const cards = stats
    ? [
        { label: "Total cases", value: stats.total_cases },
        { label: "Active", value: stats.active_cases },
        { label: "Completed", value: stats.completed_cases },
        { label: "Rush", value: stats.rush_cases },
        { label: "Stalled (10d+)", value: stats.stalled_cases },
      ]
    : [];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="font-display text-3xl tracking-tight">Dashboard</h1>
        <p className="mt-1 text-slate-500">
          Welcome back, {session.user?.name || session.user?.email}
        </p>
      </div>

      {error && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
          API unavailable: {error}. Start the Rust API with{" "}
          <code className="rounded bg-amber-100 px-1">cargo run -p caseflow-api --bin server</code>
        </div>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
        {cards.map((c) => (
          <Card key={c.label}>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-slate-500">
                {c.label}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-semibold tabular-nums">{c.value}</div>
            </CardContent>
          </Card>
        ))}
      </div>

      {stats && (
        <Card>
          <CardHeader>
            <CardTitle>Cases by stage</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {stats.by_stage.map((row) => (
                <div key={row.stage} className="flex items-center justify-between text-sm">
                  <span>{stageLabel(row.stage)}</span>
                  <span className="font-semibold tabular-nums">{row.count}</span>
                </div>
              ))}
              {!stats.by_stage.length && (
                <p className="text-sm text-slate-500">No cases yet.</p>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
