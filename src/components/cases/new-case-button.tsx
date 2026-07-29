"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { browserApi } from "@/lib/api";
import { STAGES } from "@/lib/utils";

export function NewCaseButton() {
  const { data } = useSession();
  const token = (data as { accessToken?: string } | null)?.accessToken;
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [form, setForm] = useState({
    subject: "",
    client: "",
    investigation_type: "Premises Liability",
    stage: "not_started",
    priority: "normal",
  });

  async function create() {
    if (!form.subject.trim() || !form.client.trim()) {
      toast.error("Subject and client are required");
      return;
    }
    setLoading(true);
    try {
      await browserApi("/cases", token, {
        method: "POST",
        body: JSON.stringify(form),
      });
      toast.success("Case created");
      setOpen(false);
      setForm({
        subject: "",
        client: "",
        investigation_type: "Premises Liability",
        stage: "not_started",
        priority: "normal",
      });
      router.refresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Create failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <>
      <Button onClick={() => setOpen(true)}>
        <Plus className="h-4 w-4" />
        New Case
      </Button>
      {open && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/50 p-4">
          <div className="w-full max-w-lg rounded-xl bg-white p-6 shadow-xl">
            <h2 className="font-display text-xl">New case</h2>
            <div className="mt-4 space-y-3">
              <Input
                placeholder="Subject *"
                value={form.subject}
                onChange={(e) => setForm({ ...form, subject: e.target.value })}
              />
              <Input
                placeholder="Client *"
                value={form.client}
                onChange={(e) => setForm({ ...form, client: e.target.value })}
              />
              <Input
                placeholder="Investigation type"
                value={form.investigation_type}
                onChange={(e) =>
                  setForm({ ...form, investigation_type: e.target.value })
                }
              />
              <select
                className="h-10 w-full rounded-md border border-slate-200 px-3 text-sm"
                value={form.stage}
                onChange={(e) => setForm({ ...form, stage: e.target.value })}
              >
                {STAGES.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.label}
                  </option>
                ))}
              </select>
              <select
                className="h-10 w-full rounded-md border border-slate-200 px-3 text-sm"
                value={form.priority}
                onChange={(e) => setForm({ ...form, priority: e.target.value })}
              >
                <option value="normal">Normal</option>
                <option value="rush">Rush</option>
                <option value="urgent">Urgent</option>
              </select>
            </div>
            <div className="mt-6 flex justify-end gap-2">
              <Button variant="outline" onClick={() => setOpen(false)}>
                Cancel
              </Button>
              <Button onClick={create} disabled={loading}>
                {loading ? "Creating…" : "Create"}
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
