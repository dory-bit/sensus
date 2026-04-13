#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod google_calendar;
#[cfg(test)]
mod tests;

use chrono::{Utc, Local};
use rusqlite::params;
use db::{
    add_quest, get_all_quests, init_db, quest_exists, save_stats, toggle_quest_status, DbState, Quest,
    UserStats, check_and_update_daily_streak,
};
use google_calendar::GoogleCalendarClient;
use std::sync::Mutex;
use tauri::State;


#[tauri::command]
fn load_user_data(state: State<'_, DbState>) -> UserStats {
    let conn = state.0.lock().unwrap();
    
    let today = Local::now().date_naive().to_string();
    
    // Try to get stats, if fail, return defaults
    match db::get_stats(&conn) {
        Ok(stats) => {
            // Check for daily reset
            let last_update = stats.last_update.clone();
        if last_update.split('T').next().unwrap_or("") != today {
            let _ = db::delete_completed_quests(&conn);
            let now = Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE stats SET last_update = ?1, hp = 100, stm = 100 WHERE id = 1",
                params![now],
            );
        }

            let _ = db::check_and_update_daily_streak(&conn);
            stats
        },
        Err(_) => UserStats {
            lvl: 1, xp: 0, hp: 100, stm: 100, int: 10, spr: 10, last_update: today,
        }
    }
}

#[tauri::command]
fn get_streak(state: State<'_, DbState>) -> i32 {
    let conn = state.0.lock().unwrap();
    db::get_current_streak(&conn)
}

#[tauri::command]
fn update_streak_status(state: State<'_, DbState>, success: bool) -> Result<i32, String> {
    let conn = state.0.lock().unwrap();
    db::update_streak(&conn, success).map_err(|e| e.to_string())
}


#[tauri::command]
fn update_user_stats(state: State<'_, DbState>, stats: UserStats, activity: Option<String>, value: Option<i32>) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    
    if let (Some(act), Some(val)) = (activity, value) {
        db::log_activity(&conn, &act, val).map_err(|e| e.to_string())?;
    }
    
    save_stats(&conn, &stats).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_new_quest(state: State<'_, DbState>, text: String, parent_id: i32) -> Result<i32, String> {
    println!(
        "Sensus API: Comando add_new_quest recebido -> text: {}, parent_id: {}",
        text, parent_id
    );
    let conn = state.0.lock().unwrap();
    let xp = if parent_id != -1 { 5 } else { 10 };
    match add_quest(&conn, &text, parent_id, xp) {
        Ok(id) => {
            println!("Sensus API: Sucesso ao adicionar quest. ID: {}", id);
            Ok(id)
        }
        Err(e) => {
            println!("Sensus API: ERRO ao adicionar quest: {}", e);
            Err(format!("Erro no Banco de Dados: {}", e))
        }
    }
}

#[tauri::command]
fn update_quest_position(state: State<'_, DbState>, id: i32, position: i32) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::update_quest_position(&conn, id, position).map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_quest(state: State<'_, DbState>, id: i32) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::cancel_quest(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn reschedule_quest(state: State<'_, DbState>, id: i32, date: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::reschedule_quest(&conn, id, &date).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_activity_logs(state: State<'_, DbState>) -> Result<Vec<(String, String, i32)>, String> {


    let conn = state.0.lock().unwrap();
    db::get_activity_logs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_daily_total(state: State<'_, DbState>, activity_type: String) -> Result<i32, String> {
    let conn = state.0.lock().unwrap();
    db::get_daily_total(&conn, &activity_type).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_sleep_logs(state: State<'_, DbState>) -> Result<Vec<(String, String)>, String> {
    let conn = state.0.lock().unwrap();
    db::get_sleep_logs(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn fetch_quests(state: State<'_, DbState>) -> Result<Vec<Quest>, String> {
    let conn = state.0.lock().unwrap();
    get_all_quests(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_quest(state: State<'_, DbState>, id: i32, completed: bool) -> Result<UserStats, String> {
    let conn = state.0.lock().unwrap();
    
    // 1. Verificar status atual da quest para evitar ganho de XP duplicado ou perda incorreta
    let current_completed: bool = conn.query_row(
        "SELECT is_completed FROM quests WHERE id = ?1",
        [id],
        |row| row.get(0),
    ).map_err(|e| format!("Erro ao buscar status da quest: {}", e))?;
 
    if completed && !current_completed {
        // Ganhou XP: Mudou de não-concluída para concluída
        let quest_xp: i32 = conn.query_row(
            "SELECT xp FROM quests WHERE id = ?1",
            [id],
            |row| row.get(0),
        ).map_err(|e| format!("Erro ao buscar XP da quest: {}", e))?;
 
        let mut stats = db::get_stats(&conn).map_err(|e| format!("Erro ao ler stats: {}", e))?;
        stats.xp += quest_xp;
        if stats.xp >= 100 {
            stats.lvl += 1;
            stats.xp -= 100;
        }
        db::save_stats(&conn, &stats).map_err(|e| format!("Erro ao salvar XP: {}", e))?;
    } else if !completed && current_completed {
        // Perdeu XP: Mudou de concluída para não-concluída
        let quest_xp: i32 = conn.query_row(
            "SELECT xp FROM quests WHERE id = ?1",
            [id],
            |row| row.get(0),
        ).map_err(|e| format!("Erro ao buscar XP da quest: {}", e))?;
 
        let mut stats = db::get_stats(&conn).map_err(|e| format!("Erro ao ler stats: {}", e))?;
        stats.xp = (stats.xp - quest_xp).max(0);
        // Nota: Não reduzimos nível aqui para evitar complexidade, apenas o XP
        db::save_stats(&conn, &stats).map_err(|e| format!("Erro ao remover XP: {}", e))?;
    }
 
    toggle_quest_status(&conn, id, completed).map_err(|e| e.to_string())?;
    
    // Retornar stats atualizados para o frontend sincronizar a UI
    db::get_stats(&conn).map_err(|e| format!("Erro ao ler stats finais: {}", e))
}


#[tauri::command]
async fn sync_google_calendar(state: State<'_, DbState>) -> Result<String, String> {
    println!("Sensus API: Iniciando sincronização com Google Calendar...");
    
    let token_path = r"C:\Users\Lukinha Gaming\Documents\ia\sensus\app\token.json";
    let mut client = GoogleCalendarClient::new(token_path).map_err(|e| format!("Erro ao carregar token: {}", e))?;
    
    let events = loop {
        match client.fetch_events().await {
            Ok(evs) => break evs,
            Err(e) if e.to_string() == "TOKEN_EXPIRED" => {
                println!("Sensus API: Token expirado, tentando renovar...");
                client.refresh_access_token().await.map_err(|re| format!("Erro ao renovar token: {}", re))?;
            }
            Err(e) => return Err(format!("Erro ao buscar eventos: {}", e)),
        }
    };

    let mut added_count = 0;

    let conn = state.0.lock().unwrap();
    for event in events {
        if let Some(summary) = event.summary {
            println!("Sensus API: Analisando evento -> '{}'", summary);
            let exists = quest_exists(&conn, &summary);
            
            if exists {
                println!("Sensus API: Conflito detectado! Evento '{}' já existe.", summary);
                let all_quests = db::get_all_quests(&conn).unwrap_or_default();
                let quest_list: Vec<String> = all_quests.iter().map(|q| q.task_text.clone()).collect();
                println!("Sensus API: Missões atualmente no banco: {:?}", quest_list);
            } else {
                if let Ok(_) = add_quest(&conn, &summary, -1, 10) {
                    added_count += 1;
                    println!("Sensus API: Evento '{}' adicionado com sucesso!", summary);
                }
            }
        } else {
            println!("Sensus API: Ignorando evento sem título.");
        }
    }

    println!("Sensus API: Sincronização concluída. {} novos eventos adicionados.", added_count);
    Ok(format!("Sincronização concluída! {} novos eventos adicionados.", added_count))
}

#[tauri::command]
fn add_medication(state: State<'_, DbState>, name: String) -> Result<i32, String> {
    let conn = state.0.lock().unwrap();
    db::add_medication(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_medications(state: State<'_, DbState>) -> Result<Vec<db::Medication>, String> {
    let conn = state.0.lock().unwrap();
    db::get_medications(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_medication(state: State<'_, DbState>, id: i32, is_taken: bool) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::toggle_medication(&conn, id, is_taken).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_medication(state: State<'_, DbState>, id: i32) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_medication(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn log_sleep(state: State<'_, DbState>, quality: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::log_sleep(&conn, &quality).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_last_sleep(state: State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().unwrap();
    db::get_last_sleep(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn greet(name: &str) -> String {

    format!("Olá, {}! Bem-vindo ao Sensus Tauri.", name)
}

fn main() {
    let connection = init_db();

    tauri::Builder::default()
        .manage(DbState(Mutex::new(connection)))
        .invoke_handler(tauri::generate_handler![
            greet,
            load_user_data,
            update_user_stats,
            add_new_quest,
            fetch_quests,
            toggle_quest,
            sync_google_calendar,
            update_quest_position,
            cancel_quest,
            reschedule_quest,
            get_streak,
            update_streak_status,
            get_activity_logs,
            get_daily_total,
            get_sleep_logs,
            add_medication,
            get_medications,
            toggle_medication,
            delete_medication,
            log_sleep,
            get_last_sleep
        ])


        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
