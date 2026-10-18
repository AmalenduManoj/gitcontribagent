CREATE TABLE repositories (
    id              BIGSERIAL PRIMARY KEY,
    owner           TEXT NOT NULL,
    name            TEXT NOT NULL,
    github_repo_id  BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (owner, name)
);

CREATE TABLE issues (
    id              BIGSERIAL PRIMARY KEY,
    repository_id   BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    github_issue_id BIGINT NOT NULL,
    issue_number    INTEGER NOT NULL,
    title           TEXT NOT NULL,
    body            TEXT,
    state           TEXT NOT NULL DEFAULT 'open',
    author          TEXT,
    labels          JSONB NOT NULL DEFAULT '[]'::jsonb,
    html_url        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (repository_id, issue_number)
);

CREATE TABLE issue_analyses (
    id              BIGSERIAL PRIMARY KEY,
    issue_id        BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    summary         TEXT NOT NULL,
    requirements    JSONB NOT NULL DEFAULT '[]'::jsonb,
    affected_areas  JSONB NOT NULL DEFAULT '[]'::jsonb,
    possible_files  JSONB NOT NULL DEFAULT '[]'::jsonb,
    implementation_approach JSONB NOT NULL DEFAULT '[]'::jsonb,
    testing_requirements    JSONB NOT NULL DEFAULT '[]'::jsonb,
    complexity      TEXT NOT NULL CHECK (complexity IN ('low','medium','high')),
    newcomer_suitability    TEXT NOT NULL,
    uncertainties   JSONB NOT NULL DEFAULT '[]'::jsonb,
    raw_response    TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE notifications (
    id              BIGSERIAL PRIMARY KEY,
    analysis_id     BIGINT NOT NULL REFERENCES issue_analyses(id) ON DELETE CASCADE,
    provider        TEXT NOT NULL DEFAULT 'email',
    recipient       TEXT NOT NULL,
    subject         TEXT NOT NULL,
    body            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE github_webhook_events (
    id              BIGSERIAL PRIMARY KEY,
    event_id        TEXT NOT NULL UNIQUE,
    event_type      TEXT NOT NULL,
    payload         JSONB NOT NULL,
    processed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE approval_actions (
    id              BIGSERIAL PRIMARY KEY,
    issue_id        BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    action_type     TEXT NOT NULL CHECK (action_type IN ('post_comment','create_pr','create_branch')),
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','approved','rejected','executed','failed')),
    generated_payload   JSONB,
    approved_by     TEXT,
    approved_at     TIMESTAMPTZ,
    executed_at     TIMESTAMPTZ,
    github_comment_id   BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE github_comments (
    id              BIGSERIAL PRIMARY KEY,
    approval_action_id  BIGINT NOT NULL REFERENCES approval_actions(id) ON DELETE CASCADE,
    github_comment_id   BIGINT NOT NULL,
    issue_id        BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    body            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
