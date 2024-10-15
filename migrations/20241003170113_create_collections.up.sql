-- Create collection table
CREATE TABLE collection (
    collection_id SERIAL PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES account(account_id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE (owner_id, name)
);

CREATE UNIQUE INDEX idx_one_default_per_owner
ON collection (owner_id)
WHERE is_default = true;

CREATE OR REPLACE FUNCTION update_collection_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_collection_timestamp
BEFORE UPDATE ON collection
FOR EACH ROW
EXECUTE FUNCTION update_collection_timestamp();

-- Create collection_repositories table
CREATE TABLE collection_repository (
    collection_id INTEGER REFERENCES collection(collection_id) ON DELETE CASCADE,
    repository_id INTEGER REFERENCES repository(repository_id) ON DELETE CASCADE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (collection_id, repository_id)
);
