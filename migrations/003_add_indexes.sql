CREATE INDEX IF NOT EXISTS idx_app_usage_start_time ON app_usage(start_time);
CREATE INDEX IF NOT EXISTS idx_app_usage_app_name ON app_usage(app_name);
CREATE INDEX IF NOT EXISTS idx_pomodoro_records_start_time ON pomodoro_records(start_time); 