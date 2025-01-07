-- 创建项目表
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    color TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- 创建标签表
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at DATETIME NOT NULL
);

-- 创建类别表
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    productivity_score REAL NOT NULL DEFAULT 0.0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- 创建应用使用记录表
CREATE TABLE IF NOT EXISTS app_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    window_title TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    duration INTEGER NOT NULL,
    category_id INTEGER,
    productivity_score REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- 创建番茄钟记录表
CREATE TABLE IF NOT EXISTS pomodoro_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    status TEXT NOT NULL,
    notes TEXT,
    project_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
);

-- 创建番茄钟标签关联表
CREATE TABLE IF NOT EXISTS pomodoro_tags (
    pomodoro_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (pomodoro_id, tag_id),
    FOREIGN KEY(pomodoro_id) REFERENCES pomodoro_records(id) ON DELETE CASCADE,
    FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 创建每日总结表
CREATE TABLE IF NOT EXISTS daily_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date DATE NOT NULL UNIQUE,
    total_work_time INTEGER NOT NULL,
    productive_time INTEGER NOT NULL,
    completed_pomodoros INTEGER NOT NULL,
    interrupted_pomodoros INTEGER NOT NULL,
    most_used_app TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
); 