import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const STAGES = [
  { id: "not_started", label: "Not Started", color: "#9CA3AF" },
  { id: "contact_attempted", label: "Contact Attempted", color: "#F59E0B" },
  { id: "client_contacted", label: "Client Contacted", color: "#3B82F6" },
  { id: "inspection_scheduled", label: "Inspection Scheduled", color: "#8B5CF6" },
  { id: "scene_inspected", label: "Scene Inspected", color: "#14B8A6" },
  { id: "report_sent", label: "Report Sent", color: "#6366F1" },
  { id: "completed", label: "Completed/Paid", color: "#22C55E" },
  { id: "cancelled", label: "Cancelled", color: "#EF4444" },
] as const;

export function stageLabel(id: string) {
  return STAGES.find((s) => s.id === id)?.label ?? id;
}

export function stageColor(id: string) {
  return STAGES.find((s) => s.id === id)?.color ?? "#6B7280";
}
