ALTER TABLE app_usage ADD COLUMN productivity_score REAL DEFAULT 0.0;
ALTER TABLE app_usage ADD COLUMN category_id INTEGER REFERENCES categories(id);

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    is_productive BOOLEAN DEFAULT FALSE
); 