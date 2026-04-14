use chrono::{Local, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Quest {
    pub id: i32,
    pub task_text: String,
    pub is_completed: bool,
    pub parent_id: Option<i32>,
    pub xp: i32,
    pub position: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStats {
    pub lvl: i32,
    pub xp: i32,
    pub hp: i32,
    pub stm: i32,
    pub int: i32,
    pub spr: i32,
    pub last_update: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Medication {
    pub id: i32,
    pub name: String,
    pub is_taken: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SleepLog {
    pub id: i32,
    pub quality: String,
    pub date: String,
}

pub struct DbState(pub Mutex<Connection>);

fn get_db_path() -> PathBuf {
    PathBuf::from(r"C:\Users\Lukinha Gaming\Documents\ia\sensus_final.db")
}

pub fn init_db() -> Connection {
    let db_path = get_db_path();
    println!("Sensus DB: Inicializando banco em {:?}", db_path);

    // DEBUG: Write the path to a file to verify at runtime
    let _ = std::fs::write(
        r"C:\Users\Lukinha Gaming\Documents\ia\db_debug.txt",
        format!("{:?}", db_path),
    );

    let conn = Connection::open(db_path).expect("Erro ao abrir banco de dados");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS stats (
            id INTEGER PRIMARY KEY,
            lvl INTEGER,
            xp INTEGER,
            hp INTEGER,
            stm INTEGER,
            int INTEGER,
            spr INTEGER,
            last_update TEXT
        )",
        [],
    )
    .expect("Erro ao criar tabela de stats");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS quests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_text TEXT NOT NULL,
            is_completed BOOLEAN NOT NULL DEFAULT 0,
            parent_id INTEGER,
            xp INTEGER DEFAULT 10,
            position INTEGER DEFAULT 0,
            due_date TEXT,
            FOREIGN KEY(parent_id) REFERENCES quests(id)
        )",
        [],
    )
    .expect("Erro ao criar tabela de quests");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS activity_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            activity_type TEXT NOT NULL,
            value INTEGER NOT NULL
        )",
        [],
    )
    .expect("Erro ao criar tabela de logs");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS streaks (
            id INTEGER PRIMARY KEY,
            current_streak INTEGER DEFAULT 0,
            longest_streak INTEGER DEFAULT 0,
            last_check_date TEXT
        )",
        [],
    )
    .expect("Erro ao criar tabela de streaks");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS medications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            is_taken BOOLEAN NOT NULL DEFAULT 0,
            last_taken_date TEXT
        )",
        [],
    )
    .expect("Erro ao criar tabela de medications");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sleep_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            quality TEXT NOT NULL,
            date TEXT NOT NULL
        )",
        [],
    )
    .expect("Erro ao criar tabela de sleep_logs");

    // Migrações
    let table_info: Vec<String> = conn
        .prepare("PRAGMA table_info(quests)")
        .expect("Erro ao ler info da tabela")
        .query_map([], |row| row.get(1))
        .expect("Erro no query_map")
        .filter_map(|res| res.ok())
        .collect();

    if !table_info.contains(&"position".to_string()) {
        conn.execute(
            "ALTER TABLE quests ADD COLUMN position INTEGER DEFAULT 0",
            [],
        )
        .ok();
    }
    if !table_info.contains(&"due_date".to_string()) {
        conn.execute("ALTER TABLE quests ADD COLUMN due_date TEXT", [])
            .ok();
    }

    // Migração para a tabela stats (last_update)
    let stats_info: Vec<String> = conn
        .prepare("PRAGMA table_info(stats)")
        .expect("Erro ao ler info da tabela stats")
        .query_map([], |row| row.get(1))
        .expect("Erro no query_map stats")
        .filter_map(|res| res.ok())
        .collect();

    if !stats_info.contains(&"last_update".to_string()) {
        println!("Sensus DB: Migrando tabela stats para adicionar last_update...");
        conn.execute("ALTER TABLE stats ADD COLUMN last_update TEXT", [])
            .ok();
    }

    let count: i32 = conn
        .query_row("SELECT count(*) FROM stats", [], |r| r.get(0))
        .unwrap_or(0);
    if count == 0 {
        conn.execute(
            "INSERT INTO stats (lvl, xp, hp, stm, int, spr, last_update) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (1, 0, 100, 100, 10, 10, Utc::now().to_rfc3339()),
        )
        .expect("Erro ao inserir stats iniciais");
    }

    // Garantir streak inicial
    conn.execute(
        "INSERT OR IGNORE INTO streaks (id, current_streak, longest_streak) VALUES (1, 0, 0)",
        [],
    )
    .ok();

    conn
}

pub fn get_stats(conn: &Connection) -> Result<UserStats> {
    conn.query_row(
        "SELECT lvl, xp, hp, stm, int, spr, last_update FROM stats WHERE id = 1",
        [],
        |row| {
            Ok(UserStats {
                lvl: row.get(0)?,
                xp: row.get(1)?,
                hp: row.get(2)?,
                stm: row.get(3)?,
                int: row.get(4)?,
                spr: row.get(5)?,
                last_update: row.get(6)?,
            })
        },
    )
}

pub fn save_stats(conn: &Connection, stats: &UserStats) -> Result<()> {
    conn.execute(
        "UPDATE stats SET lvl = ?1, xp = ?2, hp = ?3, stm = ?4, int = ?5, spr = ?6, last_update = ?7 WHERE id = 1",
        (
            stats.lvl, stats.xp, stats.hp, stats.stm, stats.int, stats.spr, &stats.last_update,
        ),
    )?;
    Ok(())
}

pub fn add_quest(conn: &Connection, text: &str, parent_id: i32, xp: i32) -> Result<i32> {
    let trimmed_text = text.trim();
    println!(
        "Sensus DB: Tentando inserir quest: text='{}', parent_id={}, xp={}",
        trimmed_text, parent_id, xp
    );
    let pid = if parent_id == -1 {
        None
    } else {
        Some(parent_id)
    };

    conn.execute(
        "INSERT INTO quests (task_text, parent_id, xp) VALUES (?1, ?2, ?3)",
        params![trimmed_text, pid, xp],
    )?;
    let id = conn.last_insert_rowid() as i32;
    println!("Sensus DB: Quest inserida com sucesso! ID: {}", id);
    Ok(id)
}

pub fn get_all_quests(conn: &Connection) -> Result<Vec<Quest>> {
    println!("Sensus DB: Buscando missões do dia...");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut stmt = conn.prepare(
        "SELECT id, task_text, is_completed, parent_id, xp, position 
         FROM quests 
         WHERE due_date IS NULL OR due_date <= ?1 
         ORDER BY position ASC, id ASC",
    )?;
    let quest_iter = stmt.query_map([today], |row| {
        Ok(Quest {
            id: row.get(0)?,
            task_text: row.get(1)?,
            is_completed: row.get(2)?,
            parent_id: row.get(3)?,
            xp: row.get(4)?,
            position: row.get(5)?,
        })
    })?;

    let mut quests = Vec::new();
    for quest in quest_iter {
        quests.push(quest?);
    }
    println!("Sensus DB: {} missões encontradas para hoje.", quests.len());
    Ok(quests)
}

pub fn update_quest_position(conn: &Connection, id: i32, position: i32) -> Result<()> {
    conn.execute(
        "UPDATE quests SET position = ?1 WHERE id = ?2",
        params![position, id],
    )?;
    Ok(())
}

pub fn log_activity(conn: &Connection, activity_type: &str, value: i32) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_logs (timestamp, activity_type, value) VALUES (?1, ?2, ?3)",
        params![Utc::now().to_rfc3339(), activity_type, value],
    )?;
    Ok(())
}

pub fn get_activity_logs(conn: &Connection) -> Result<Vec<(String, String, i32)>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, activity_type, value FROM activity_logs ORDER BY timestamp ASC",
    )?;
    let logs = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|res| res.ok())
        .collect();
    Ok(logs)
}

pub fn add_medication(conn: &Connection, name: &str) -> Result<i32> {
    conn.execute(
        "INSERT INTO medications (name, is_taken) VALUES (?1, 0)",
        params![name],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_medications(conn: &Connection) -> Result<Vec<Medication>> {
    let mut stmt = conn.prepare("SELECT id, name, is_taken FROM medications")?;
    let meds = stmt
        .query_map([], |row| {
            Ok(Medication {
                id: row.get(0)?,
                name: row.get(1)?,
                is_taken: row.get(2)?,
            })
        })?
        .filter_map(|res| res.ok())
        .collect();
    Ok(meds)
}

pub fn toggle_medication(conn: &Connection, id: i32, is_taken: bool) -> Result<()> {
    let val = if is_taken { 1 } else { 0 };

    // Log to file for debugging since we can't see stdout in .exe
    let log_msg = format!(
        "{} - Toggle Med ID {}: {}\n",
        chrono::Utc::now().to_rfc3339(),
        id,
        val
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(r"C:\Users\Lukinha Gaming\Documents\ia\meds_debug.log")
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(log_msg.as_bytes())
        });

    conn.execute(
        "UPDATE medications SET is_taken = ?1, last_taken_date = ?2 WHERE id = ?3",
        params![val, Utc::now().to_rfc3339(), id],
    )?;
    Ok(())
}

pub fn delete_medication(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM medications WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn reset_meds_daily(conn: &Connection) -> Result<usize> {
    conn.execute("UPDATE medications SET is_taken = 0", [])
}

pub fn log_sleep(conn: &Connection, quality: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO sleep_logs (quality, date) VALUES (?1, ?2)",
        params![quality, Local::now().format("%Y-%m-%d").to_string()],
    )?;
    Ok(())
}

pub fn get_last_sleep(conn: &Connection) -> Result<Option<String>> {
    conn.query_row(
        "SELECT quality FROM sleep_logs ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    )
    .map(Some)
    .or(Ok(None))
}

pub fn update_streak(conn: &Connection, success: bool) -> Result<i32> {
    let mut stats: (i32, i32) = conn.query_row(
        "SELECT current_streak, longest_streak FROM streaks WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    if success {
        stats.0 += 1;
        if stats.0 > stats.1 {
            stats.1 = stats.0;
        }
    } else {
        stats.0 = 0;
    }

    conn.execute(
        "UPDATE streaks SET current_streak = ?1, longest_streak = ?2 WHERE id = 1",
        params![stats.0, stats.1],
    )?;
    Ok(stats.0)
}

pub fn check_and_update_daily_streak(conn: &Connection) -> Result<i32> {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let streak_data: (i32, i32, Option<String>) = conn
        .query_row(
            "SELECT current_streak, longest_streak, last_check_date FROM streaks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap_or((0, 0, None));

    let current_streak = streak_data.0;
    let longest_streak = streak_data.1;
    let last_check_date = streak_data.2;

    let mut new_streak = current_streak;

    if let Some(last_date) = last_check_date {
        if last_date == today {
            new_streak = current_streak;
        } else {
            let last_date_parsed = chrono::NaiveDate::parse_from_str(&last_date, "%Y-%m-%d")
                .map_err(|_| rusqlite::Error::InvalidQuery);

            if let Ok(last_d) = last_date_parsed {
                let today_parsed = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
                if today_parsed.signed_duration_since(last_d).num_days() == 1 {
                    new_streak = current_streak + 1;
                } else {
                    new_streak = 0;
                }
            } else {
                new_streak = 0;
            }
        }
    } else {
        new_streak = 1;
    }

    let new_longest = if new_streak > longest_streak {
        new_streak
    } else {
        longest_streak
    };

    conn.execute(
        "UPDATE streaks SET current_streak = ?1, longest_streak = ?2, last_check_date = ?3 WHERE id = 1",
        params![new_streak, new_longest, today],
    )?;

    Ok(new_streak)
}

pub fn get_current_streak(conn: &Connection) -> i32 {
    conn.query_row(
        "SELECT current_streak FROM streaks WHERE id = 1",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0)
}

pub fn delete_completed_quests(conn: &Connection) -> Result<usize> {
    conn.execute("DELETE FROM quests WHERE is_completed = 1", [])
}

pub fn cancel_quest(conn: &Connection, id: i32) -> Result<()> {
    // Deleta recursivamente todas as sub-missões para evitar erro de FOREIGN KEY
    conn.execute(
        "WITH RECURSIVE descendants(id) AS (
            SELECT id FROM quests WHERE id = ?1
            UNION ALL
            SELECT q.id FROM quests q JOIN descendants d ON q.parent_id = d.id
        )
        DELETE FROM quests WHERE id IN descendants",
        params![id],
    )?;
    Ok(())
}

pub fn reschedule_quest(conn: &Connection, id: i32, new_date: &str) -> Result<()> {
    conn.execute(
        "UPDATE quests SET due_date = ?1 WHERE id = ?2",
        params![new_date, id],
    )?;
    Ok(())
}

pub fn get_daily_total(conn: &Connection, activity_type: &str) -> Result<i32> {
    conn.query_row(
        "SELECT SUM(value) FROM activity_logs WHERE activity_type = ?1 AND date(timestamp, 'localtime') = date('now', 'localtime')",
        params![activity_type],
        |row| {
            let val: Option<i32> = row.get(0)?;
            Ok(val.unwrap_or(0))
        },
    )
}

pub fn get_sleep_logs(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT date, quality FROM sleep_logs ORDER BY date ASC")?;
    let logs = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|res| res.ok())
        .collect();
    Ok(logs)
}

pub fn quest_exists(conn: &Connection, text: &str) -> bool {
    let trimmed_text = text.trim();
    let mut stmt = conn
        .prepare("SELECT count(*) FROM quests WHERE task_text = ?1")
        .expect("Erro ao preparar statement");
    let count: i32 = stmt
        .query_row([trimmed_text], |row| row.get(0))
        .unwrap_or(0);
    count > 0
}

pub fn toggle_quest_status(conn: &Connection, id: i32, completed: bool) -> Result<()> {
    let log_msg = format!(
        "{} - Toggle Quest ID {}: {}\n",
        chrono::Utc::now().to_rfc3339(),
        id,
        completed
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(r"C:\Users\Lukinha Gaming\Documents\ia\sensus_debug.log")
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(log_msg.as_bytes())
        });

    conn.execute(
        "UPDATE quests SET is_completed = ?1 WHERE id = ?2",
        params![completed, id],
    )?;
    Ok(())
}
