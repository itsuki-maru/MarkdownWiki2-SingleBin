-- Add migration script here
CREATE TABLE IF NOT EXISTS user_model (
    id TEXT PRIMARY KEY NOT NULL,
    username CHARACTER VARYING(256) NOT NULL UNIQUE,
    public_name CHARACTER VARYING(256) NOT NULL,
    password CHARACTER VARYING(256) NOT NULL,
    create_at TEXT NOT NULL,
    is_superuser BOOLEAN NOT NULL,
    failed_count INTEGER NOT NULL,
    next_challenge_time TEXT NOT NULL,
    is_locked BOOLEAN NOT NULL,
    is_private BOOLEAN NOT NULL,
    is_basic_authed BOOLEAN DEFAULT FALSE NOT NULL,
    is_basic_authed_at TEXT NOT NULL,
    totp_secret CHARACTER VARYING(256) NOT NULL,
    totp_temp_secret CHARACTER VARYING(256) NOT NULL
);

CREATE TABLE IF NOT EXISTS wiki_model (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    create_at TEXT NOT NULL,
    update_at TEXT NOT NULL,
    is_public BOOLEAN NOT NULL,
    is_edit_request BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES user_model(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS image_model (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    uuid_filename TEXT NOT NULL,
    create_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user_model(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS temporary_urls (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    wiki_id TEXT NOT NULL,
    url TEXT NOT NULL,
    expiration TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    create_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS edit_request_wiki_model (
    id TEXT PRIMARY KEY NOT NULL,
    wiki_owner_id TEXT NOT NULL,
    request_user_id TEXT NOT NULL,
    request_wiki_id TEXT NOT NULL,
    edit_request_title TEXT NOT NULL,
    edit_request_body TEXT NOT NULL,
    create_at TEXT NOT NULL,
    request_message TEXT,
    status TEXT NOT NULL CHECK (status IN ('REJECT', 'REQUESTNOW', 'DRAFT', 'APPLIED')),
    FOREIGN KEY (wiki_owner_id) REFERENCES user_model(id) ON DELETE CASCADE,
    FOREIGN KEY (request_user_id) REFERENCES user_model(id) ON DELETE CASCADE,
    FOREIGN KEY (request_wiki_id) REFERENCES wiki_model(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS application_settings (
    id TEXT PRIMARY KEY NOT NULL,
    setting_key VARCHAR(255) NOT NULL UNIQUE,
    setting_value VARCHAR(255) NOT NULL,
    description TEXT,
    create_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);