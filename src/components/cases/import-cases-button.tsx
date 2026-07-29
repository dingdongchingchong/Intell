"use client";

import { useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { useSession } from "next-auth/react";
import { toast } from "sonner";
import { FileUp, Loader2 } from "lucide-react";
import * as XLSX from "xlsx";
import Papa from "papaparse";
import { Button } from "@/components/ui/button";
import { browserApi } from "@/lib/api";
import {
  mapSpreadsheetRow,
  validateImportHeaders,
  type MappedCaseRow,
  type RawSpreadsheetRow,
} from "@/lib/csv-import";

type ImportResult = {
  created: number;
  updated: number;
  skipped: number;
  errors: { row: number; case_number?: string; message: string }[];
};

async function parseFile(file: File): Promise<RawSpreadsheetRow[]> {
  const name = file.name.toLowerCase();
  if (name.endsWith(".xlsx") || name.endsWith(".xls")) {
    const buf = await file.arrayBuffer();
    const wb = XLSX.read(buf, { type: "array", cellDates: true });
    const sheet = wb.Sheets[wb.SheetNames[0]];
    if (!sheet) throw new Error("Workbook has no sheets");
    const rows = XLSX.utils.sheet_to_json<RawSpreadsheetRow>(sheet, {
      defval: "",
      raw: false,
    });
    return rows;
  }

  // CSV / TSV / plain text
  const text = await file.text();
  return new Promise((resolve, reject) => {
    Papa.parse<RawSpreadsheetRow>(text, {
      header: true,
      skipEmptyLines: "greedy",
      transformHeader: (h) => String(h || "").replace(/^\uFEFF/, "").trim(),
      complete: (results) => {
        if (results.errors?.length && !results.data?.length) {
          reject(
            new Error(
              results.errors[0]?.message || "CSV parse failed"
            )
          );
          return;
        }
        resolve(results.data || []);
      },
      error: (err: Error) => reject(err),
    });
  });
}

export function ImportCasesButton() {
  const { data } = useSession();
  const token = (data as { accessToken?: string } | null)?.accessToken;
  const router = useRouter();
  const inputRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function onFile(file: File | undefined) {
    if (!file) return;
    const lower = file.name.toLowerCase();
    if (
      !lower.endsWith(".csv") &&
      !lower.endsWith(".xlsx") &&
      !lower.endsWith(".xls") &&
      !/csv|sheet|excel|spreadsheet|text\/plain/i.test(file.type || "")
    ) {
      toast.error("Please choose a .csv or .xlsx file");
      return;
    }

    setLoading(true);
    try {
      const rawRows = await parseFile(file);
      validateImportHeaders(rawRows);
      const cases: MappedCaseRow[] = [];
      for (let i = 0; i < rawRows.length; i++) {
        const mapped = mapSpreadsheetRow(rawRows[i], i);
        if (mapped) cases.push(mapped);
      }
      if (!cases.length) {
        throw new Error("No case rows found after parsing.");
      }

      const result = await browserApi<ImportResult>("/cases/import", token, {
        method: "POST",
        body: JSON.stringify({ cases, update_existing: true }),
      });

      const errN = result.errors?.length || 0;
      toast.success(
        `Import done: ${result.created} created, ${result.updated} updated` +
          (result.skipped ? `, ${result.skipped} skipped` : "") +
          (errN ? `, ${errN} errors` : "")
      );
      if (errN && result.errors[0]) {
        toast.message(
          `First error (row ${result.errors[0].row}): ${result.errors[0].message}`
        );
      }
      router.refresh();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Import failed");
    } finally {
      setLoading(false);
      if (inputRef.current) inputRef.current.value = "";
    }
  }

  return (
    <>
      <input
        ref={inputRef}
        type="file"
        accept=".csv,.xlsx,.xls,text/csv,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet,application/vnd.ms-excel"
        className="hidden"
        onChange={(e) => onFile(e.target.files?.[0])}
      />
      <Button
        type="button"
        variant="outline"
        disabled={loading}
        onClick={() => inputRef.current?.click()}
      >
        {loading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <FileUp className="h-4 w-4" />
        )}
        Import CSV / Excel
      </Button>
    </>
  );
}
