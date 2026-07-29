import Link from "next/link";
import { getServerSession } from "next-auth";
import { redirect } from "next/navigation";
import { authOptions } from "@/lib/auth-options";
import { casesAPI } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { stageColor, stageLabel } from "@/lib/utils";
import { NewCaseButton } from "@/components/cases/new-case-button";

export default async function CasesPage() {
  const session = await getServerSession(authOptions);
  if (!session) redirect("/login");

  let cases: Awaited<ReturnType<typeof casesAPI.list>>["cases"] = [];
  let total = 0;
  let error = "";
  try {
    const res = await casesAPI.list("?page_size=100");
    cases = res.cases;
    total = res.total;
  } catch (e) {
    error = e instanceof Error ? e.message : "Failed to load cases";
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="font-display text-3xl tracking-tight">Cases</h1>
          <p className="text-slate-500">{total} total</p>
        </div>
        <NewCaseButton />
      </div>

      {error && (
        <div className="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800">
          {error}
        </div>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Case list</CardTitle>
        </CardHeader>
        <CardContent className="overflow-x-auto">
          <table className="w-full min-w-[720px] text-left text-sm">
            <thead className="border-b text-xs uppercase text-slate-500">
              <tr>
                <th className="pb-3 pr-3 font-medium">Case #</th>
                <th className="pb-3 pr-3 font-medium">Subject</th>
                <th className="pb-3 pr-3 font-medium">Client</th>
                <th className="pb-3 pr-3 font-medium">Type</th>
                <th className="pb-3 pr-3 font-medium">Stage</th>
                <th className="pb-3 font-medium">Priority</th>
              </tr>
            </thead>
            <tbody>
              {cases.map((c) => (
                <tr key={c.id} className="border-b border-slate-100 last:border-0">
                  <td className="py-3 pr-3 font-medium">
                    <Link href={`/cases/${c.id}`} className="text-blue-700 hover:underline">
                      {c.case_number}
                    </Link>
                  </td>
                  <td className="py-3 pr-3">{c.subject}</td>
                  <td className="py-3 pr-3">{c.client}</td>
                  <td className="py-3 pr-3">{c.investigation_type}</td>
                  <td className="py-3 pr-3">
                    <span
                      className="inline-flex rounded-full px-2 py-0.5 text-xs font-medium text-white"
                      style={{ background: stageColor(c.stage) }}
                    >
                      {stageLabel(c.stage)}
                    </span>
                  </td>
                  <td className="py-3 capitalize">{c.priority}</td>
                </tr>
              ))}
                  {!cases.length && !error && (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-slate-500">
                    No cases yet. Create one to get started.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </CardContent>
      </Card>
    </div>
  );
}
