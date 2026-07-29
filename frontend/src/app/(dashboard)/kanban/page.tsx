"use client";

import { useEffect, useMemo, useState } from "react";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { browserApi } from "@/lib/api";
import type { Case } from "@/lib/types";
import { STAGES, stageColor } from "@/lib/utils";

export default function KanbanPage() {
  const { data } = useSession();
  const token = (data as { accessToken?: string } | null)?.accessToken;
  const [cases, setCases] = useState<Case[]>([]);
  const [dragId, setDragId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function load() {
    setLoading(true);
    try {
      const res = await browserApi<{ cases: Case[] }>("/cases?page_size=200", token);
      setCases(res.cases);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Failed to load");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    if (token) load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  const columns = useMemo(() => {
    const cols = STAGES.map((s) => ({
      ...s,
      cases: cases.filter((c) => c.stage === s.id),
    }));
    cols.sort((a, b) => {
      const ae = a.cases.length === 0 ? 1 : 0;
      const be = b.cases.length === 0 ? 1 : 0;
      return ae - be;
    });
    return cols;
  }, [cases]);

  async function onDrop(stage: string) {
    if (!dragId) return;
    const current = cases.find((c) => c.id === dragId);
    if (!current || current.stage === stage) {
      setDragId(null);
      return;
    }
    setCases((prev) =>
      prev.map((c) => (c.id === dragId ? { ...c, stage } : c))
    );
    setDragId(null);
    try {
      await browserApi(`/cases/${dragId}/stage`, token, {
        method: "PATCH",
        body: JSON.stringify({ stage }),
      });
      toast.success("Stage updated");
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Update failed");
      load();
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-3xl tracking-tight">Kanban</h1>
        <p className="text-slate-500">Drag cards between stages</p>
      </div>
      {loading ? (
        <p className="text-sm text-slate-500">Loading…</p>
      ) : (
        <div className="flex gap-4 overflow-x-auto pb-4">
          {columns.map((col) => (
            <div
              key={col.id}
              className="w-72 shrink-0 rounded-xl border border-slate-200 bg-slate-100/70"
              onDragOver={(e) => e.preventDefault()}
              onDrop={() => onDrop(col.id)}
            >
              <div className="sticky top-0 flex items-center justify-between border-b border-slate-200 bg-white/90 px-3 py-3 backdrop-blur">
                <div className="flex items-center gap-2 text-sm font-semibold">
                  <span
                    className="h-2.5 w-2.5 rounded-full"
                    style={{ background: col.color }}
                  />
                  {col.label}
                </div>
                <span className="rounded-full bg-slate-100 px-2 text-xs">
                  {col.cases.length}
                </span>
              </div>
              <div className="space-y-2 p-2 min-h-[120px]">
                {col.cases.map((c) => (
                  <div
                    key={c.id}
                    draggable
                    onDragStart={() => setDragId(c.id)}
                    className="cursor-grab rounded-lg border border-slate-200 bg-white p-3 shadow-sm active:cursor-grabbing"
                  >
                    <div className="text-xs font-semibold text-blue-700">
                      {c.case_number}
                    </div>
                    <div className="mt-1 text-sm font-medium">{c.subject}</div>
                    <div className="mt-2 text-xs text-slate-500">{c.client}</div>
                    {c.is_rush && (
                      <span
                        className="mt-2 inline-block rounded px-1.5 py-0.5 text-[10px] font-semibold text-white"
                        style={{ background: stageColor("cancelled") }}
                      >
                        RUSH
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
