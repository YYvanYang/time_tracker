-- 为应用使用记录表添加索引
CREATE INDEX IF NOT EXISTS idx_app_usage_start_time ON app_usage(start_time);
CREATE INDEX IF NOT EXISTS idx_app_usage_app_name ON app_usage(app_name);
CREATE INDEX IF NOT EXISTS idx_app_usage_category ON app_usage(category_id);

-- 为番茄钟记录表添加索引
CREATE INDEX IF NOT EXISTS idx_pomodoro_start_time ON pomodoro_records(start_time);
CREATE INDEX IF NOT EXISTS idx_pomodoro_end_time ON pomodoro_records(end_time);
CREATE INDEX IF NOT EXISTS idx_pomodoro_status ON pomodoro_records(status);
CREATE INDEX IF NOT EXISTS idx_pomodoro_project ON pomodoro_records(project_id);

-- 为标签关联表添加索引
CREATE INDEX IF NOT EXISTS idx_pomodoro_tags_tag ON pomodoro_tags(tag_id);

-- 为每日总结表添加索引
CREATE INDEX IF NOT EXISTS idx_daily_summaries_date ON daily_summaries(date); 