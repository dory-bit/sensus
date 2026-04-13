#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::*;
    use rusqlite::{params, Connection};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to open in-memory db");
        
        conn.execute(
            "CREATE TABLE stats (id INTEGER PRIMARY KEY, lvl INTEGER, xp INTEGER, hp INTEGER, stm INTEGER, int INTEGER, spr INTEGER, last_update TEXT)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE quests (id INTEGER PRIMARY KEY AUTOINCREMENT, task_text TEXT NOT NULL, is_completed BOOLEAN NOT NULL DEFAULT 0, parent_id INTEGER, xp INTEGER DEFAULT 10, position INTEGER DEFAULT 0, due_date TEXT, FOREIGN KEY(parent_id) REFERENCES quests(id))",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE activity_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp TEXT NOT NULL, activity_type TEXT NOT NULL, value INTEGER NOT NULL)",
            [],
        ).unwrap();
        conn.execute(
            "CREATE TABLE streaks (id INTEGER PRIMARY KEY, current_streak INTEGER DEFAULT 0, longest_streak INTEGER DEFAULT 0, last_check_date TEXT)",
            [],
        ).unwrap();
        
        conn.execute(
            "INSERT INTO stats (lvl, xp, hp, stm, int, spr, last_update) VALUES (1, 0, 100, 100, 10, 10, '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute("INSERT INTO streaks (id, current_streak, longest_streak) VALUES (1, 0, 0)", []).unwrap();
        
        conn
    }

    #[test]
    fn test_stats_logic() {
        let conn = setup_test_db();
        let mut stats = get_stats(&conn);
        stats.xp = 50;
        save_stats(&conn, &stats).unwrap();
        let updated = get_stats(&conn);
        assert_eq!(updated.xp, 50);
    }

    #[test]
    fn test_quest_lifecycle() {
        let conn = setup_test_db();
        let qid = add_quest(&conn, "Test Quest", -1, 20).unwrap();
        
        toggle_quest_status(&conn, qid, true).unwrap();
        let quests = get_all_quests(&conn).unwrap();
        assert!(quests.iter().any(|q| q.id == qid && q.is_completed));
        
        cancel_quest(&conn, qid).unwrap();
        let quests_after = get_all_quests(&conn).unwrap();
        assert!(!quests_after.iter().any(|q| q.id == qid));
    }

    #[test]
    fn test_activity_and_total() {
        let conn = setup_test_db();
        log_activity(&conn, "water", 300).unwrap();
        log_activity(&conn, "water", 200).unwrap();
        log_activity(&conn, "food", 500).unwrap();
        
        let water_total = get_//...
