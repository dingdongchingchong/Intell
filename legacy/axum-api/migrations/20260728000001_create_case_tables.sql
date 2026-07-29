-- Phase 1: investigation case tables (CSV + SPA stages)

CREATE TYPE case_status AS ENUM (
    'not_started',
    'contact_attempted',
    'client_contacted',
    'inspection_scheduled',
    'scene_inspected',
    'report_sent',
    'completed',
    'cancelled',
    'stalled'
);

CREATE TYPE case_activity_type AS ENUM (
    'created',
    'assigned',
    'status_change',
    'note_added',
    'scene_visit',
    'client_contact',
    'report_sent',
    'filemail_sent',
    'expense_added',
    'completed',
    'updated'
);

CREATE TABLE cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_number TEXT NOT NULL UNIQUE,

    -- CSV-aligned columns
    investigation_type TEXT NOT NULL,
    subject_plaintiff TEXT,
    client_firm TEXT,
    client_client TEXT,
    client_contact TEXT,
    investigator TEXT,
    case_notes_area TEXT,
    client_file_number TEXT,
    additional_info TEXT,
    expenses NUMERIC(12, 2) NOT NULL DEFAULT 0,

    status case_status NOT NULL DEFAULT 'not_started',

    date_assigned DATE,
    date_completed_paid DATE,

    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cases_status ON cases (status);
CREATE INDEX idx_cases_investigator ON cases (investigator);
CREATE INDEX idx_cases_date_assigned ON cases (date_assigned);
CREATE INDEX idx_cases_date_completed ON cases (date_completed_paid);
CREATE INDEX idx_cases_created_at ON cases (created_at DESC);

CREATE TABLE case_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    activity_type case_activity_type NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_activities_case ON case_activities (case_id, created_at DESC);
CREATE INDEX idx_case_activities_user ON case_activities (user_id);

CREATE TABLE case_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    note TEXT NOT NULL,
    is_internal BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_notes_case ON case_notes (case_id, created_at DESC);
CREATE INDEX idx_case_notes_user ON case_notes (user_id);

CREATE TABLE case_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    assigned_to TEXT NOT NULL,
    assigned_by UUID REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_assignments_case ON case_assignments (case_id, assigned_at DESC);
CREATE INDEX idx_case_assignments_to ON case_assignments (assigned_to);
