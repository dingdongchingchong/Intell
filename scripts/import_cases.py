#!/usr/bin/env python3
"""
Import investigation CSV rows into CaseFlow via the REST API.

Maps columns to Phase 1 case schema and classifies status using the same
rules as investigation.html (Date Completed/Paid + note keywords).

Usage:
  # Dry-run (no writes)
  python3 scripts/import_cases.py --csv sample.csv --dry-run

  # Import
  python3 scripts/import_cases.py \\
    --csv sample.csv \\
    --api http://127.0.0.1:8080 \\
    --email admin@caseflow.local \\
    --password 'admin123456'

  # Skip existing case numbers (default). Use --upsert to PUT updates.
  python3 scripts/import_cases.py --csv sample.csv --upsert

Environment:
  CASEFLOW_API_URL, CASEFLOW_EMAIL, CASEFLOW_PASSWORD
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Optional


# --- header / CSV helpers -------------------------------------------------

def normalize_header(h: str) -> str:
    s = str(h or "").lstrip("\ufeff")
    for ch in ("\u2018", "\u2019", "\u201a", "\u201b"):
        s = s.replace(ch, "'")
    return re.sub(r"\s+", " ", s).strip().lower()


def get_field(row: dict[str, str], *names: str) -> str:
    keyed = {normalize_header(k): (v if v is not None else "") for k, v in row.items()}
    for name in names:
        want = normalize_header(name)
        if want in keyed:
            return str(keyed[want]).strip()
        # soft match: ignore punctuation differences
        for k, v in keyed.items():
            if re.sub(r"[^a-z0-9]+", "", k) == re.sub(r"[^a-z0-9]+", "", want):
                return str(v).strip()
    return ""


def parse_us_date(value: str) -> Optional[str]:
    """Return ISO date YYYY-MM-DD if value is M/D[/YYYY], else None."""
    s = (value or "").strip()
    m = re.match(r"^(\d{1,2})/(\d{1,2})(?:/(\d{2,4}))?$", s)
    if not m:
        return None
    month, day = int(m.group(1)), int(m.group(2))
    year = m.group(3)
    if year is None:
        year = datetime.now().year
    else:
        year = int(year)
        if year < 100:
            year += 2000
    try:
        return datetime(year, month, day).date().isoformat()
    except ValueError:
        return None


def extract_expenses(text: str) -> Optional[str]:
    """Best-effort first dollar amount as decimal string."""
    if not text:
        return None
    m = re.search(r"\$?\s*(\d{1,6}(?:,\d{3})*(?:\.\d{1,2})?)\b", text)
    if not m:
        return None
    raw = m.group(1).replace(",", "")
    try:
        return f"{float(raw):.2f}"
    except ValueError:
        return None


# --- stage classification (mirrors investigation.html) --------------------

KW_SUCCESS = re.compile(
    r"\b(contacted client|spoke to client|spoke with client|talked to client|"
    r"client called|client responded|client confirmed|client verified|spoke by phone)\b",
    re.I,
)
KW_FAIL = re.compile(
    r"\b(contact attempted|attempted|no response|voicemail|left message|\blm\b|n/a|no contact)\b",
    re.I,
)
KW_SCHEDULED = re.compile(
    r"\b(scheduled(?: for)?|going to scene|inspection on|visit on|"
    r"will conduct inspection|inspection scheduled)\b",
    re.I,
)
KW_INSPECTED = re.compile(
    r"\b(inspection (?:complete|done)|scene inspection complete|investigation complete|"
    r"photos (?:taken|obtained|uploaded)|measurements taken|photos obtained)\b",
    re.I,
)
KW_LUX = re.compile(r"\b(lux measurements?|lux at)\b", re.I)
KW_REPORT = re.compile(
    r"\b(filemail sent|sent to cs|cs report|photos uploaded|report sent)\b",
    re.I,
)


def classify_status(date_completed_raw: str, case_status: str, notes_area: str) -> str:
    dcp = (date_completed_raw or "").strip()
    dcp_low = dcp.lower()
    corpus = " ".join(x for x in (case_status, notes_area, dcp) if x)

    if re.search(r"cancell?ed", corpus, re.I) or re.search(r"cancell?ed", dcp, re.I):
        return "cancelled"
    if re.search(r"transferred to bk", corpus, re.I):
        return "cancelled"

    if parse_us_date(dcp) or re.fullmatch(r"(paid|completed)", dcp_low):
        return "completed"

    if (
        re.match(r"^sent", dcp, re.I)
        or re.search(r"cs report", dcp, re.I)
        or re.search(r"photos uploaded", dcp, re.I)
        or re.search(r"no billing", dcp, re.I)
    ):
        if re.search(r"check pending|invoice pending", dcp, re.I):
            return "report_sent"
        if re.search(r"not billing|no billing", dcp, re.I):
            return "completed"
        return "report_sent"

    stage = "not_started"
    if KW_FAIL.search(corpus) and not KW_SUCCESS.search(corpus):
        stage = "contact_attempted"
    if KW_SUCCESS.search(corpus):
        stage = "client_contacted"
    if KW_SCHEDULED.search(corpus):
        stage = "inspection_scheduled"
    if KW_INSPECTED.search(corpus) or KW_LUX.search(corpus):
        stage = "scene_inspected"
    if KW_REPORT.search(corpus):
        stage = "report_sent"
    return stage


def row_to_payload(row: dict[str, str]) -> Optional[dict[str, Any]]:
    case_number = get_field(row, "Ace Case #", "Ace Case#", "Case #", "Case Number")
    if not case_number:
        return None

    inv_type = get_field(row, "Investigation Type") or "Special Request"
    subject = get_field(row, "Subject") or None
    client = get_field(row, "Client") or None
    client_client = get_field(row, "Client's Client", "Clients Client") or None
    client_contact = get_field(row, "Client Contact") or None
    investigator = get_field(row, "Investigator") or None
    notes_area = get_field(row, "Case Notes/ Area", "Case Notes/Area", "Case Notes") or None
    file_no = get_field(row, "Client File#", "Client File #") or None
    additional = get_field(row, "Additional Info / Expenses", "Additional Info/Expenses") or None
    case_status_raw = get_field(row, "Case Status", "Status")
    date_assigned_raw = get_field(row, "Date")
    date_completed_raw = get_field(row, "Date Completed/ Paid", "Date Completed/Paid")

    status = classify_status(date_completed_raw, case_status_raw, notes_area or "")
    expenses = extract_expenses(additional or "")

    # Preserve freeform status text inside notes when present
    if case_status_raw:
        if notes_area:
            notes_area = f"{notes_area}\n[CSV Status] {case_status_raw}"
        else:
            notes_area = f"[CSV Status] {case_status_raw}"

    payload: dict[str, Any] = {
        "case_number": case_number,
        "investigation_type": inv_type,
        "subject_plaintiff": subject,
        "client_firm": client,
        "client_client": client_client,
        "client_contact": client_contact,
        "investigator": investigator,
        "case_notes_area": notes_area,
        "client_file_number": file_no,
        "additional_info": additional,
        "status": status,
        "date_assigned": parse_us_date(date_assigned_raw),
        "date_completed_paid": parse_us_date(date_completed_raw),
    }
    if expenses is not None:
        payload["expenses"] = expenses
    return payload


# --- HTTP client ----------------------------------------------------------

@dataclass
class ApiClient:
    base: str
    token: Optional[str] = None

    def _url(self, path: str) -> str:
        return self.base.rstrip("/") + path

    def request(
        self,
        method: str,
        path: str,
        body: Any = None,
        auth: bool = True,
    ) -> tuple[int, Any]:
        data = None
        headers = {"Accept": "application/json"}
        if body is not None:
            data = json.dumps(body).encode("utf-8")
            headers["Content-Type"] = "application/json"
        if auth and self.token:
            headers["Authorization"] = f"Bearer {self.token}"

        req = urllib.request.Request(self._url(path), data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                raw = resp.read().decode("utf-8")
                return resp.status, json.loads(raw) if raw else None
        except urllib.error.HTTPError as e:
            raw = e.read().decode("utf-8", errors="replace")
            try:
                parsed = json.loads(raw) if raw else {"error": raw}
            except json.JSONDecodeError:
                parsed = {"error": raw}
            return e.code, parsed

    def login(self, email: str, password: str) -> None:
        status, data = self.request(
            "POST",
            "/api/v1/auth/login",
            {"login": email, "password": password},
            auth=False,
        )
        if status >= 400:
            raise SystemExit(f"login failed ({status}): {data}")
        # AuthResponse shape: { data: { tokens: { access_token }, ... } } or similar
        token = None
        if isinstance(data, dict):
            inner = data.get("data", data)
            if isinstance(inner, dict):
                tokens = inner.get("tokens") or inner.get("token_pair") or inner
                if isinstance(tokens, dict):
                    token = tokens.get("access_token") or tokens.get("accessToken")
                if not token:
                    token = inner.get("access_token")
        if not token:
            raise SystemExit(f"login ok but no access_token in response: {data}")
        self.token = token


def load_rows(csv_path: Path) -> list[dict[str, str]]:
    with csv_path.open(newline="", encoding="utf-8-sig") as f:
        reader = csv.DictReader(f)
        return [dict(r) for r in reader]


def main() -> int:
    parser = argparse.ArgumentParser(description="Import CaseFlow investigation CSV via API")
    parser.add_argument(
        "--csv",
        type=Path,
        default=Path("sample.csv"),
        help="Path to investigation CSV (default: sample.csv)",
    )
    parser.add_argument(
        "--api",
        default=os.environ.get("CASEFLOW_API_URL", "http://127.0.0.1:8080"),
        help="API base URL",
    )
    parser.add_argument(
        "--email",
        default=os.environ.get("CASEFLOW_EMAIL", "admin@caseflow.local"),
    )
    parser.add_argument(
        "--password",
        default=os.environ.get("CASEFLOW_PASSWORD", "admin123456"),
    )
    parser.add_argument("--dry-run", action="store_true", help="Parse/map only; no API writes")
    parser.add_argument(
        "--upsert",
        action="store_true",
        help="PUT update when case_number already exists (default: skip)",
    )
    parser.add_argument("--limit", type=int, default=0, help="Import only first N rows (0=all)")
    parser.add_argument("--sleep", type=float, default=0.0, help="Delay between writes (seconds)")
    parser.add_argument("--verbose", "-v", action="store_true")
    args = parser.parse_args()

    if not args.csv.exists():
        print(f"CSV not found: {args.csv}", file=sys.stderr)
        return 1

    rows = load_rows(args.csv)
    payloads: list[dict[str, Any]] = []
    skipped_empty = 0
    for row in rows:
        payload = row_to_payload(row)
        if payload is None:
            skipped_empty += 1
            continue
        payloads.append(payload)

    if args.limit > 0:
        payloads = payloads[: args.limit]

    status_counts = Counter(p["status"] for p in payloads)
    print(f"CSV rows: {len(rows)}")
    print(f"Mapped cases: {len(payloads)} (skipped empty case #: {skipped_empty})")
    print("Status distribution:")
    for status, count in status_counts.most_common():
        print(f"  {status:22s} {count}")

    if args.dry_run:
        if args.verbose and payloads:
            print("\nSample payload:")
            print(json.dumps(payloads[0], indent=2))
        print("\nDry-run complete — no API calls.")
        return 0

    client = ApiClient(args.api)
    print(f"\nLogging in to {args.api} as {args.email} …")
    client.login(args.email, args.password)

    created = updated = skipped = failed = 0
    for i, payload in enumerate(payloads, 1):
        case_number = payload["case_number"]
        # Check existence
        code, existing = client.request("GET", f"/api/v1/cases/{urllib.parse.quote(case_number)}")
        exists = code == 200

        if exists and not args.upsert:
            skipped += 1
            if args.verbose:
                print(f"[{i}/{len(payloads)}] skip {case_number}")
            continue

        if exists and args.upsert:
            # UpdateCaseRequest — omit case_number
            body = {k: v for k, v in payload.items() if k != "case_number" and v is not None}
            code, resp = client.request(
                "PUT",
                f"/api/v1/cases/{urllib.parse.quote(case_number)}",
                body,
            )
            ok = 200 <= code < 300
            if ok:
                updated += 1
                mark = "updated"
            else:
                failed += 1
                mark = f"FAIL update {code}: {resp}"
        else:
            body = {k: v for k, v in payload.items() if v is not None}
            code, resp = client.request("POST", "/api/v1/cases", body)
            ok = 200 <= code < 300
            if ok:
                created += 1
                mark = "created"
            else:
                failed += 1
                mark = f"FAIL create {code}: {resp}"

        if args.verbose or not ok:
            print(f"[{i}/{len(payloads)}] {case_number}: {mark}")
        elif i % 50 == 0 or i == len(payloads):
            print(f"… {i}/{len(payloads)}")

        if args.sleep:
            time.sleep(args.sleep)

    print(
        f"\nDone. created={created} updated={updated} skipped={skipped} failed={failed}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
