CREATE TABLE account (
    account_id SERIAL PRIMARY KEY,
    github_id VARCHAR(255) UNIQUE NOT NULL,
    last_login TIMESTAMP WITH TIME ZONE,
    auth_type VARCHAR(50) NOT NULL DEFAULT 'github',
    email VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add an index on github_id for faster lookups
CREATE INDEX idx_accounts_github_id ON account(github_id);
