//src/export.rs

use crate::error::{Result, TimeTrackerError};
use crate::storage::models::*;
use chrono::{DateTime, Local};
use serde::Serialize;
use std::path::Path;
use std::fs::File;
use std::io::Write;

#[derive(Serialize)]
pub struct ExportData {
    pub version: String,
    pub export_date: DateTime<Local>,
    pub app_usage: Vec<AppUsageRecord>,
    pub pomodoros: Vec<PomodoroRecord>,
    pub projects: Vec<Project>,
    pub tasks: Vec<Task>,
    pub tags: Vec<Tag>,
    pub daily_summaries: Vec<DailySummary>,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    CSV,
    JSON,
    Excel,
    HTML,
}

pub struct DataExporter {
    pub format: ExportFormat,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub include_app_usage: bool,
    pub include_pomodoros: bool,
    pub include_projects: bool,
    pub include_tasks: bool,
    pub include_summaries: bool,
}

impl DataExporter {
    pub fn new(format: ExportFormat) -> Self {
        let end_date = Local::now();
        let start_date = end_date - chrono::Duration::days(30); // 默认导出30天数据

        Self {
            format,
            start_date,
            end_date,
            include_app_usage: true,
            include_pomodoros: true,
            include_projects: true,
            include_tasks: true,
            include_summaries: true,
        }
    }

    pub fn export<P: AsRef<Path>>(&self, storage: &Storage, path: P) -> Result<()> {
        let data = self.collect_data(storage)?;

        match self.format {
            ExportFormat::CSV => self.export_csv(data, path),
            ExportFormat::JSON => self.export_json(data, path),
            ExportFormat::Excel => self.export_excel(data, path),
            ExportFormat::HTML => self.export_html(data, path),
        }
    }

    fn collect_data(&self, storage: &Storage) -> Result<ExportData> {
        let mut data = ExportData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            export_date: Local::now(),
            app_usage: Vec::new(),
            pomodoros: Vec::new(),
            projects: Vec::new(),
            tasks: Vec::new(),
            tags: Vec::new(),
            daily_summaries: Vec::new(),
        };

        if self.include_app_usage {
            data.app_usage = storage.get_app_usage_by_date_range(
                self.start_date,
                self.end_date,
            )?;
        }

        if self.include_pomodoros {
            data.pomodoros = storage.get_pomodoro_records_by_date_range(
                self.start_date,
                self.end_date,
            )?;
        }

        if self.include_projects {
            data.projects = storage.get_projects()?;
        }

        if self.include_tasks {
            data.tasks = storage.get_all_tasks()?;
        }

        if self.include_summaries {
            data.daily_summaries = storage.get_daily_summaries_by_date_range(
                self.start_date,
                self.end_date,
            )?;
        }

        Ok(data)
    }

    fn export_csv<P: AsRef<Path>>(&self, data: ExportData, path: P) -> Result<()> {
        let path = path.as_ref();
        std::fs::create_dir_all(path.parent().unwrap())?;

        // 导出应用使用记录
        if self.include_app_usage {
            let mut writer = csv::Writer::from_path(
                path.with_file_name("app_usage.csv")
            )?;

            writer.write_record(&[
                "App Name",
                "Window Title",
                "Start Time",
                "Duration (seconds)",
                "Category",
                "Is Productive",
            ])?;

            for record in data.app_usage {
                writer.write_record(&[
                    &record.app_name,
                    &record.window_title,
                    &record.start_time.to_rfc3339(),
                    &record.duration.as_secs().to_string(),
                    &record.category,
                    &record.is_productive.to_string(),
                ])?;
            }
        }

        // 导出番茄钟记录
        if self.include_pomodoros {
            let mut writer = csv::Writer::from_path(
                path.with_file_name("pomodoros.csv")
            )?;

            writer.write_record(&[
                "Start Time",
                "End Time",
                "Status",
                "Notes",
                "Project",
                "Tags",
            ])?;

            for record in data.pomodoros {
                writer.write_record(&[
                    &record.start_time.to_rfc3339(),
                    &record.end_time.to_rfc3339(),
                    &record.status.to_string(),
                    &record.notes.unwrap_or_default(),
                    &record.project_id.map(|id| id.to_string()).unwrap_or_default(),
                    &record.tags.join(","),
                ])?;
            }
        }

        // 导出每日统计
        if self.include_summaries {
            let mut writer = csv::Writer::from_path(
                path.with_file_name("daily_summaries.csv")
            )?;

            writer.write_record(&[
                "Date",
                "Total Work Time (minutes)",
                "Productive Time (minutes)",
                "Completed Pomodoros",
                "Interrupted Pomodoros",
                "Most Used App",
            ])?;

            for summary in data.daily_summaries {
                writer.write_record(&[
                    &summary.date.format("%Y-%m-%d").to_string(),
                    &(summary.total_work_time.as_secs() / 60).to_string(),
                    &(summary.productive_time.as_secs() / 60).to_string(),
                    &summary.completed_pomodoros.to_string(),
                    &summary.interrupted_pomodoros.to_string(),
                    &summary.most_used_app.unwrap_or_default(),
                ])?;
            }
        }

        Ok(())
    }

    fn export_json<P: AsRef<Path>>(&self, data: ExportData, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(&data)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn export_excel<P: AsRef<Path>>(&self, data: ExportData, path: P) -> Result<()> {
        use rust_xlsxwriter::{Workbook, Format, Chart, ChartType};

        let mut workbook = Workbook::new();

        // 应用使用统计表
        if self.include_app_usage {
            let worksheet = workbook.add_worksheet();
            worksheet.write_string(0, 0, "应用统计")?;

            let header_format = Format::new()
                .set_bold()
                .set_border(rust_xlsxwriter::FormatBorder::Thin);

            let headers = ["应用", "使用时长(小时)", "使用次数", "生产力得分"];
            for (col, header) in headers.iter().enumerate() {
                worksheet.write_string_with_format(1, col as u16, header, &header_format)?;
            }

            // 分组统计应用使用数据
            let mut app_stats: HashMap<String, (f64, u32, f64)> = HashMap::new();
            for record in &data.app_usage {
                let entry = app_stats.entry(record.app_name.clone())
                    .or_insert((0.0, 0, 0.0));
                entry.0 += record.duration.as_secs_f64() / 3600.0;
                entry.1 += 1;
                if record.is_productive {
                    entry.2 += 1.0;
                }
            }

            let mut row = 2;
            for (app, (hours, count, productive)) in app_stats {
                worksheet.write_string(row, 0, &app)?;
                worksheet.write_number(row, 1, hours)?;
                worksheet.write_number(row, 2, count as f64)?;
                worksheet.write_number(row, 3, productive / count as f64 * 100.0)?;
                row += 1;
            }

            // 添加应用使用时长饼图
            let mut chart = Chart::new(ChartType::Pie);
            chart.add_series()
                .set_categories(("Sheet1", 2, 0, row - 1, 0))
                .set_values(("Sheet1", 2, 1, row - 1, 1));
            worksheet.insert_chart(1, 5, &chart)?;
        }

        // 番茄钟统计表
        if self.include_pomodoros {
            let worksheet = workbook.add_worksheet();
            worksheet.write_string(0, 0, "番茄钟统计")?;

            let headers = ["日期", "完成数", "中断数", "专注时长(小时)"];
            let header_format = Format::new()
                .set_bold()
                .set_border(rust_xlsxwriter::FormatBorder::Thin);

            for (col, header) in headers.iter().enumerate() {
                worksheet.write_string_with_format(1, col as u16, header, &header_format)?;
            }

            // 按日期分组统计
            let mut daily_stats: HashMap<String, (u32, u32, f64)> = HashMap::new();
            for record in &data.pomodoros {
                let date = record.start_time.format("%Y-%m-%d").to_string();
                let entry = daily_stats.entry(date)
                    .or_insert((0, 0, Duration::from_secs(0), HashSet::new()));
                
                match record.status {
                    PomodoroStatus::Completed => {
                        entry.0 += 1;
                        entry.2 += record.duration();
                    }
                    PomodoroStatus::Interrupted => {
                        entry.1 += 1;
                    }
                }

                if let Some(project_id) = record.project_id {
                    if let Ok(project) = data.projects.iter().find(|p| p.id == project_id) {
                        entry.3.insert(project.name.clone());
                    }
                }
            }

            for (date, (completed, interrupted, duration, projects)) in daily_stats {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.1}小时</td><td>{}</td></tr>\n",
                    date,
                    completed,
                    interrupted,
                    duration.as_secs_f64() / 3600.0,
                    projects.into_iter().collect::<Vec<_>>().join(", ")
                ));
            }

            html.push_str("</table>\n</section>\n");
        }

        // 项目统计
        if self.include_projects {
            html.push_str("<section>\n");
            html.push_str("<h2>项目统计</h2>\n");
            html.push_str("<div class=\"projects\">\n");

            for project in &data.projects {
                html.push_str("<div class=\"project-card\">\n");
                html.push_str(&format!("<h3>{}</h3>\n", project.name));
                
                if let Some(desc) = &project.description {
                    html.push_str(&format!("<p>{}</p>\n", desc));
                }

                // 统计项目的番茄钟数据
                let project_pomodoros: Vec<_> = data.pomodoros.iter()
                    .filter(|p| p.project_id == Some(project.id))
                    .collect();

                let completed = project_pomodoros.iter()
                    .filter(|p| matches!(p.status, PomodoroStatus::Completed))
                    .count();
                
                let total_time: Duration = project_pomodoros.iter()
                    .map(|p| p.duration())
                    .sum();

                html.push_str("<div class=\"project-stats\">\n");
                html.push_str(&format!("<div>完成番茄数: {}</div>\n", completed));
                html.push_str(&format!("<div>总时长: {:.1}小时</div>\n", 
                    total_time.as_secs_f64() / 3600.0));
                html.push_str("</div>\n");

                html.push_str("</div>\n");
            }

            html.push_str("</div>\n</section>\n");
        }

        // 每日总结
        if self.include_summaries {
            html.push_str("<section>\n");
            html.push_str("<h2>每日总结</h2>\n");
            html.push_str("<table>\n");
            html.push_str(
                "<tr><th>日期</th><th>工作时长</th><th>生产时间</th><th>完成番茄数</th><th>主要应用</th></tr>\n"
            );

            for summary in &data.daily_summaries {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{:.1}小时</td><td>{:.1}小时</td><td>{}</td><td>{}</td></tr>\n",
                    summary.date.format("%Y-%m-%d"),
                    summary.total_work_time.as_secs_f64() / 3600.0,
                    summary.productive_time.as_secs_f64() / 3600.0,
                    summary.completed_pomodoros,
                    summary.most_used_app.as_deref().unwrap_or("-")
                ));
            }

            html.push_str("</table>\n</section>\n");
        }

        html.push_str("</body>\n</html>");

        let mut file = File::create(path)?;
        file.write_all(html.as_bytes())?;
        Ok(())
    }
}

// 导入数据
pub struct DataImporter {
    format: ExportFormat,
}

impl DataImporter {
    pub fn new(format: ExportFormat) -> Self {
        Self { format }
    }

    pub fn import<P: AsRef<Path>>(&self, path: P, storage: &Storage) -> Result<()> {
        match self.format {
            ExportFormat::JSON => self.import_json(path, storage),
            ExportFormat::CSV => self.import_csv(path, storage),
            _ => Err(TimeTrackerError::Platform(
                "Unsupported import format".into()
            )),
        }
    }

    fn import_json<P: AsRef<Path>>(&self, path: P, storage: &Storage) -> Result<()> {
        let file = File::open(path)?;
        let data: ExportData = serde_json::from_reader(file)?;

        storage.transaction(|tx| {
            // 导入应用使用记录
            for record in data.app_usage {
                tx.execute(
                    "INSERT INTO app_usage (app_name, window_title, start_time, duration, category, is_productive)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        record.app_name,
                        record.window_title,
                        record.start_time,
                        record.duration.as_secs(),
                        record.category,
                        record.is_productive,
                    ],
                )?;
            }

            // 导入番茄钟记录
            for record in data.pomodoros {
                tx.execute(
                    "INSERT INTO pomodoros (start_time, end_time, status, notes, project_id)
                     VALUES (?, ?, ?, ?, ?)",
                    params![
                        record.start_time,
                        record.end_time,
                        record.status.to_string(),
                        record.notes,
                        record.project_id,
                    ],
                )?;
            }

            // 导入每日总结
            for summary in data.daily_summaries {
                tx.execute(
                    "INSERT INTO daily_summaries (date, total_work_time, productive_time, completed_pomodoros, interrupted_pomodoros, most_used_app)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        summary.date,
                        summary.total_work_time.as_secs(),
                        summary.productive_time.as_secs(),
                        summary.completed_pomodoros,
                        summary.interrupted_pomodoros,
                        summary.most_used_app,
                    ],
                )?;
            }

            Ok(())
        })
    }

    fn import_csv<P: AsRef<Path>>(&self, path: P, storage: &Storage) -> Result<()> {
        let dir = path.as_ref().parent().unwrap();
        
        // 导入应用使用记录
        let app_usage_path = dir.join("app_usage.csv");
        if app_usage_path.exists() {
            let mut rdr = csv::Reader::from_path(app_usage_path)?;
            for result in rdr.records() {
                let record = result?;
                storage.create_app_usage_record(
                    &record[0],
                    &record[1],
                    DateTime::parse_from_rfc3339(&record[2])?.with_timezone(&Local),
                    Duration::from_secs(record[3].parse()?),
                    &record[4],
                    record[5].parse()?,
                )?;
            }
        }

        // 导入番茄钟记录
        let pomodoros_path = dir.join("pomodoros.csv");
        if pomodoros_path.exists() {
            let mut rdr = csv::Reader::from_path(pomodoros_path)?;
            for result in rdr.records() {
                let record = result?;
                storage.create_pomodoro_record(
                    DateTime::parse_from_rfc3339(&record[0])?.with_timezone(&Local),
                    DateTime::parse_from_rfc3339(&record[1])?.with_timezone(&Local),
                    record[2].parse()?,
                    if record[3].is_empty() { None } else { Some(record[3].to_string()) },
                    if record[4].is_empty() { None } else { Some(record[4].parse()?) },
                    record[5].split(',').map(|s| s.to_string()).collect(),
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_export_import_json() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = Storage::new(&temp_dir.path().to_path_buf())?;

        // 创建一些测试数据
        storage.create_app_usage_record(
            "test_app",
            "test_window",
            Local::now(),
            Duration::from_secs(3600),
            "test",
            true,
        )?;

        // 导出数据
        let export_path = temp_dir.path().join("export.json");
        let exporter = DataExporter::new(ExportFormat::JSON);
        exporter.export(&storage, &export_path)?;

        // 清空数据库
        storage.clear_all_data()?;

        // 导入数据
        let importer = DataImporter::new(ExportFormat::JSON);
        importer.import(&export_path, &storage)?;

        // 验证数据
        let records = storage.get_all_app_usage()?;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].app_name, "test_app");

        Ok(())
    }

    #[test]
    fn test_export_formats() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = Storage::new(&temp_dir.path().to_path_buf())?;

        // 测试各种导出格式
        let formats = [
            ExportFormat::CSV,
            ExportFormat::JSON,
            ExportFormat::Excel,
            ExportFormat::HTML,
        ];

        for format in formats {
            let exporter = DataExporter::new(format);
            let path = temp_dir.path().join(format!("export.{:?}", format).to_lowercase());
            assert!(exporter.export(&storage, &path).is_ok());
        }

        Ok(())
    }
}
start_time.format("%Y-%m-%d").to_string();
                let entry = daily_stats.entry(date)
                    .or_insert((0, 0, 0.0));
                
                match record.status {
                    PomodoroStatus::Completed => {
                        entry.0 += 1;
                        entry.2 += record.duration().as_secs_f64() / 3600.0;
                    }
                    PomodoroStatus::Interrupted => {
                        entry.1 += 1;
                    }
                }
            }

            let mut row = 2;
            for (date, (completed, interrupted, hours)) in daily_stats {
                worksheet.write_string(row, 0, &date)?;
                worksheet.write_number(row, 1, completed as f64)?;
                worksheet.write_number(row, 2, interrupted as f64)?;
                worksheet.write_number(row, 3, hours)?;
                row += 1;
            }

            // 添加完成情况折线图
            let mut chart = Chart::new(ChartType::Line);
            chart.add_series()
                .set_name("完成数")
                .set_categories(("Sheet2", 2, 0, row - 1, 0))
                .set_values(("Sheet2", 2, 1, row - 1, 1));
            chart.add_series()
                .set_name("中断数")
                .set_categories(("Sheet2", 2, 0, row - 1, 0))
                .set_values(("Sheet2", 2, 2, row - 1, 2));
            worksheet.insert_chart(1, 5, &chart)?;
        }

        workbook.save(path)?;
        Ok(())
    }

    fn export_html<P: AsRef<Path>>(&self, data: ExportData, path: P) -> Result<()> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str("<title>时间追踪报告</title>\n");
        html.push_str(include_str!("../assets/templates/export_style.css"));
        html.push_str("</head>\n<body>\n");

        // 添加概览
        html.push_str("<h1>时间追踪报告</h1>\n");
        html.push_str(&format!(
            "<p>导出时间: {}</p>\n",
            data.export_date.format("%Y-%m-%d %H:%M:%S")
        ));
        html.push_str(&format!(
            "<p>时间范围: {} 至 {}</p>\n",
            self.start_date.format("%Y-%m-%d"),
            self.end_date.format("%Y-%m-%d")
        ));

        // 应用使用统计
        if self.include_app_usage {
            html.push_str("<section>\n");
            html.push_str("<h2>应用使用统计</h2>\n");
            html.push_str("<table>\n");
            html.push_str("<tr><th>应用</th><th>使用时长</th><th>使用次数</th><th>生产力</th></tr>\n");

            let mut app_stats: HashMap<String, (Duration, u32, u32)> = HashMap::new();
            for record in &data.app_usage {
                let entry = app_stats.entry(record.app_name.clone())
                    .or_insert((Duration::from_secs(0), 0, 0));
                entry.0 += record.duration;
                entry.1 += 1;
                if record.is_productive {
                    entry.2 += 1;
                }
            }

            for (app, (duration, count, productive)) in app_stats {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{:.1}小时</td><td>{}</td><td>{:.1}%</td></tr>\n",
                    app,
                    duration.as_secs_f64() / 3600.0,
                    count,
                    productive as f64 / count as f64 * 100.0
                ));
            }

            html.push_str("</table>\n</section>\n");
        }

        // 番茄钟统计
        if self.include_pomodoros {
            html.push_str("<section>\n");
            html.push_str("<h2>番茄钟统计</h2>\n");
            html.push_str("<table>\n");
            html.push_str(
                "<tr><th>日期</th><th>完成数</th><th>中断数</th><th>专注时长</th><th>项目</th></tr>\n"
            );

            let mut daily_stats: HashMap<String, (u32, u32, Duration, HashSet<String>)> = HashMap::new();
            for record in &data.pomodoros {
                let date = record.//src/export.rs

use crate::error::{Result, TimeTrackerError};
use crate::storage::models