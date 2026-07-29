/** Ace Investigation Manager spreadsheet → CreateCase API payload */

export type RawSpreadsheetRow = Record<string, unknown>;

export interface MappedCaseRow {
  case_number?: string;
  subject: string;
  investigation_type: string;
  client: string;
  clients_client?: string;
  client_contact?: string;
  client_file?: string;
  opened_date?: string;
  completed_date?: string;
  case_notes?: string;
  additional_info?: string;
  case_status?: string;
  stage?: string;
  priority?: string;
  is_rush?: boolean;
}

export function normalizeHeader(h: unknown): string {
  return String(h ?? "")
    .replace(/^\uFEFF/, "")
    .normalize("NFKD")
    .replace(/[\u2018\u2019\u201A\u201B'`]/g, "'")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim()
    .replace(/\s+/g, " ");
}

function cell(row: RawSpreadsheetRow, ...names: string[]): string {
  const keys = Object.keys(row).filter((k) => normalizeHeader(k) !== "");
  for (const name of names) {
    const want = normalizeHeader(name);
    if (!want) continue;
    const found = keys.find((k) => normalizeHeader(k) === want);
    if (found != null && row[found] != null && String(row[found]).trim() !== "") {
      return String(row[found]).trim();
    }
  }
  for (const name of names) {
    const want = normalizeHeader(name);
    if (!want) continue;
    const found = keys.find((k) => {
      const nk = normalizeHeader(k);
      if (!nk) return false;
      return nk.includes(want) || (want.length >= 4 && want.includes(nk));
    });
    if (found != null && row[found] != null && String(row[found]).trim() !== "") {
      return String(row[found]).trim();
    }
  }
  return "";
}

/** Parse dates like 1/2/2026, 2026-01-02, Excel serials already stringified. */
export function parseFlexibleDate(raw: string): string | undefined {
  const s = raw.trim();
  if (!s || /^cancel/i.test(s)) return undefined;
  // Excel serial number as string
  if (/^\d+(\.\d+)?$/.test(s)) {
    const n = Number(s);
    if (n > 20000 && n < 80000) {
      const utc = new Date(Date.UTC(1899, 11, 30) + n * 86400000);
      return utc.toISOString().slice(0, 10);
    }
  }
  const mdy = s.match(/^(\d{1,2})[\/\-.](\d{1,2})[\/\-.](\d{2,4})$/);
  if (mdy) {
    let y = Number(mdy[3]);
    if (y < 100) y += 2000;
    const m = Number(mdy[1]);
    const d = Number(mdy[2]);
    if (m >= 1 && m <= 12 && d >= 1 && d <= 31) {
      return `${y}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    }
  }
  const iso = s.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (iso) return `${iso[1]}-${iso[2]}-${iso[3]}`;
  return undefined;
}

export function inferStage(status: string, completedRaw: string): string {
  const blob = `${status} ${completedRaw}`.toLowerCase();
  if (/cancel/.test(blob)) return "cancelled";
  if (
    /filemail sent|investigation complete|scene inspection complete|completed\b|complete\b/.test(
      blob
    ) &&
    !/pending|to be|scheduled|attempt/.test(status.toLowerCase())
  ) {
    return "completed";
  }
  if (/report|filemail/.test(blob)) return "report_sent";
  if (/scene inspection|inspection done|inspected/.test(blob)) return "scene_inspected";
  if (/scheduled|schedule/.test(blob)) return "inspection_scheduled";
  if (/contact made|client respond|outreach/.test(blob)) return "client_contacted";
  if (/contact attempt|attempted|lm\/|texted|called/.test(blob)) return "contact_attempted";
  if (parseFlexibleDate(completedRaw) && !/cancel/i.test(completedRaw)) return "completed";
  return "not_started";
}

export function mapSpreadsheetRow(
  row: RawSpreadsheetRow,
  index: number
): MappedCaseRow | null {
  const vals = Object.values(row).filter((v) => v != null && String(v).trim() !== "");
  if (vals.length < 2) return null;

  const caseNumber =
    cell(row, "Ace Case #", "Ace Case#", "Case #", "Case Number", "Case ID") ||
    undefined;
  const subject = cell(row, "Subject", "Name") || "—";
  const client = cell(row, "Client");
  if (!client && subject === "—") return null;

  const additional = cell(
    row,
    "Additional Info / Expenses",
    "Additional Info",
    "Expenses"
  );
  const status = cell(row, "Case Status", "Status");
  const completedRaw = cell(
    row,
    "Date Completed/ Paid",
    "Date Completed/Paid",
    "Completed",
    "Date Completed"
  );
  const openedRaw = cell(row, "Date", "Opened", "Open Date", "Date Opened");
  const notes = cell(row, "Case Notes/ Area", "Case Notes/Area", "Case Notes", "Notes");
  const isRush = /\brush\b/i.test(`${additional} ${notes} ${status}`);

  return {
    case_number: caseNumber || `ROW-${index + 1}`,
    subject,
    investigation_type:
      cell(row, "Investigation Type", "InvestigationType", "Type") || "Unknown",
    client: client || "Unknown",
    clients_client:
      cell(row, "Client's Client", "Clients Client", "Client Client") || undefined,
    client_contact: cell(row, "Client Contact", "Contact") || undefined,
    client_file:
      cell(row, "Client File#", "Client File #", "File #", "File#") || undefined,
    opened_date: parseFlexibleDate(openedRaw),
    completed_date: parseFlexibleDate(completedRaw),
    case_notes: [notes, cell(row, "Investigator") ? `Investigator: ${cell(row, "Investigator")}` : ""]
      .filter(Boolean)
      .join("\n") || undefined,
    additional_info: additional || undefined,
    case_status: status || undefined,
    stage: inferStage(status, completedRaw),
    priority: isRush ? "rush" : "normal",
    is_rush: isRush,
  };
}

export function validateImportHeaders(rows: RawSpreadsheetRow[]): void {
  if (!rows.length) throw new Error("No data rows found.");
  const headerKeys = Object.keys(rows[0] || {}).map(normalizeHeader);
  const hasHint = headerKeys.some(
    (k) =>
      k.includes("ace case") ||
      k.includes("case") ||
      k === "date" ||
      k.includes("subject") ||
      k.includes("investigator")
  );
  if (!hasHint) {
    throw new Error(
      "Unrecognized columns. Expected headers like Date, Ace Case #, Subject, Client…"
    );
  }
}
