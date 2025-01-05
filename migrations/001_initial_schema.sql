CREATE TABLE IF NOT EXISTS app_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    window_title TEXT,
    start_time DATETIME NOT NULL,
    duration INTEGER NOT NULL,
    category TEXT,
    is_productive BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS pomodoro_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    status TEXT NOT NULL,
    notes TEXT,
    project_id INTEGER,
    FOREIGN KEY(project_id) REFERENCES projects(id)
); 