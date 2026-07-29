"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { browserApi } from "@/lib/api";
import { STAGES } from "@/lib/utils";

export function StageSelect({
  caseId,
  current,
}: {
  caseId: string;
  current: string;
}) {
  const { data } = useSession();
  const token = (data as { accessToken?: string } | null)?.accessToken;
  const [stage, setStage] = useState(current);
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  async function save() {
    setLoading(true);
    try {
      await browserApi(`/cases/${caseId}/stage`, token, {
        method: "PATCH",
        body: JSON.stringify({ stage }),
      });
      toast.success("Stage updated");
      router.refresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Update failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex flex-col gap-3 sm:flex-row">
      <select
        className="h-10 flex-1 rounded-md border border-slate-200 px-3 text-sm"
        value={stage}
        onChange={(e) => setStage(e.target.value)}
      >
        {STAGES.map((s) => (
          <option key={s.id} value={s.id}>
            {s.label}
          </option>
        ))}
      </select>
      <Button onClick={save} disabled={loading || stage === current}>
        {loading ? "Saving…" : "Save"}
      </Button>
    </div>
  );
}
