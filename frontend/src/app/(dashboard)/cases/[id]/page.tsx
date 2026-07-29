import { getServerSession } from "next-auth";
import { redirect, notFound } from "next/navigation";
import { authOptions } from "@/lib/auth-options";
import { casesAPI } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { stageColor, stageLabel } from "@/lib/utils";
import { StageSelect } from "@/components/cases/stage-select";

export default async function CaseDetailPage({
  params,
}: {
  params: { id: string };
}) {
  const session = await getServerSession(authOptions);
  if (!session) redirect("/login");

  let caseData;
  try {
    const res = await casesAPI.get(params.id);
    caseData = res.case;
  } catch {
    notFound();
  }

  return (
    <div className="space-y-6">
      <div>
        <p className="text-sm text-slate-500">{caseData.case_number}</p>
        <h1 className="font-display text-3xl tracking-tight">{caseData.subject}</h1>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Details</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <Row label="Client" value={caseData.client} />
            <Row label="Type" value={caseData.investigation_type} />
            <Row label="Priority" value={caseData.priority} />
            <Row label="Opened" value={caseData.opened_date} />
            <div className="flex items-center justify-between gap-4 pt-2">
              <span className="text-slate-500">Stage</span>
              <span
                className="rounded-full px-2 py-0.5 text-xs font-medium text-white"
                style={{ background: stageColor(caseData.stage) }}
              >
                {stageLabel(caseData.stage)}
              </span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Update stage</CardTitle>
          </CardHeader>
          <CardContent>
            <StageSelect caseId={caseData.id} current={caseData.stage} />
          </CardContent>
        </Card>
      </div>

      {caseData.case_notes && (
        <Card>
          <CardHeader>
            <CardTitle>Notes</CardTitle>
          </CardHeader>
          <CardContent className="whitespace-pre-wrap text-sm text-slate-700">
            {caseData.case_notes}
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-4">
      <span className="text-slate-500">{label}</span>
      <span className="font-medium capitalize">{value}</span>
    </div>
  );
}
