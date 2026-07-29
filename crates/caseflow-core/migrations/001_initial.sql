-- CaseFlow enterprise schema
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'manager', 'investigator', 'viewer')),
    name TEXT NOT NULL,
    avatar TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_number TEXT UNIQUE NOT NULL,
    subject TEXT NOT NULL,
    investigation_type TEXT NOT NULL,
    client TEXT NOT NULL,
    clients_client TEXT,
    client_contact TEXT,
    client_file TEXT,
    investigator_id UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_to_id UUID REFERENCES users(id) ON DELETE SET NULL,
    stage TEXT NOT NULL CHECK (stage IN (
        'not_started', 'contact_attempted', 'client_contacted',
        'inspection_scheduled', 'scene_inspected', 'report_sent',
        'completed', 'cancelled'
    )),
    priority TEXT NOT NULL CHECK (priority IN ('normal', 'rush', 'urgent')) DEFAULT 'normal',
    is_rush BOOLEAN NOT NULL DEFAULT false,
    is_rework BOOLEAN NOT NULL DEFAULT false,
    is_death BOOLEAN NOT NULL DEFAULT false,
    opened_date DATE NOT NULL DEFAULT CURRENT_DATE,
    completed_date DATE,
    case_notes TEXT,
    additional_info TEXT,
    case_status TEXT,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ,
    assigned_by UUID REFERENCES users(id) ON DELETE SET NULL,
    stage_changed_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS case_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    is_internal BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    resource_id TEXT,
    details TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cases_assigned_to ON cases(assigned_to_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_cases_stage ON cases(stage) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_cases_investigator ON cases(investigator_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_cases_opened ON cases(opened_date DESC);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE deleted_at IS NULL;
