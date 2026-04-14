const { invoke } = window.__TAURI__.tauri;

let state = { lvl: 1, xp: 0, hp: 100, stm: 100, int: 10, spr: 10 };
let currentBreakdownId = null;
let draggedElement = null;
let activeMetrics = [];

const MENU_DATA = {
    water: { title: "💧 Hidratação", options: [{ label: "300 ml", value: 300, hp: 5, xp: 2 }, { label: "500 ml", value: 500, hp: 10, xp: 5 }, { label: "700 ml", value: 700, hp: 15, xp: 8 }] },
    food: { title: "🍲 Alimentação", options: [{ label: "☕ Café da Manhã", stm: 20, xp: 10 }, { label: "🍲 Almoço", stm: 20, xp: 10 }, { label: "🍪 Lanche", stm: 10, xp: 5 }, { label: "🍽️ Janta", stm: 20, xp: 10 }] },
    mood: { title: "🌈 Bem-estar", options: [{ label: "😊 Feliz", spr: 10, xp: 5 }, { label: "🌟 Inspirado", spr: 15, xp: 10 }, { label: "😐 Neutro", spr: 2, xp: 2 }, { label: "😔 Triste", spr: -5, xp: 5 }, { label: "😠 Irritado", spr: -5, xp: 5 }, { label: "😴 Cansado", spr: -5, xp: 5 }] },
    hygiene: { title: "🧼 Higiene", options: [{ label: "🪥 Escovar Dentes", spr: 5, xp: 5 }, { label: "🚿 Banho", hp: 10, spr: 10, xp: 15 }, { label: "🧴 Skin Care", spr: 5, xp: 5 }] }
};

// Global Error Handler
window.onerror = function(message, source, lineno, colno, error) {
    debugLog(`<span style="color:red">CRITICAL ERROR: ${message} (Line: ${lineno})</span>`);
    return false;
};

function debugLog(msg) {
    console.log(`Sensus: ${msg}`);
}

async function tauriInvoke(cmd, args = {}) {
    try {
        let inv = null;
        if (typeof invoke === 'function') inv = invoke;
        else if (window.__TAURI__ && window.__TAURI__.invoke) inv = window.__TAURI__.invoke;
        else if (window.__TAURI__ && window.__TAURI__.tauri && window.__TAURI__.tauri.invoke) inv = window.__TAURI__.tauri.invoke;
        
        if (!inv) {
            throw new Error("Tauri invoke function not found. Is the app running in a Tauri environment?");
        }
        
        debugLog(`Invoking ${cmd} with args: ${JSON.stringify(args)}`);
        const result = await inv(cmd, args);
        debugLog(`Response from ${cmd}: ${JSON.stringify(result)}`);
        return result;
    } catch (e) { 
        debugLog(`Tauri Invoke Error [${cmd}]: ${e}`); 
        throw e; 
    }
}

function getTitle(lvl) {
    if (lvl <= 5) return "Aprendiz";
    if (lvl <= 10) return "Iniciante";
    if (lvl <= 15) return "Aventureiro Inexperiente";
    if (lvl <= 20) return "Aventureiro Experiente";
    if (lvl <= 25) return "Herói da Vizinhança";
    if (lvl <= 30) return "Herói da Cidade";
    if (lvl <= 35) return "Herói do País";
    if (lvl <= 40) return "Jovem Mestre";
    if (lvl <= 45) return "Mestre";
    return "Mestre Ancião";
}

function updateUI() {
    try {
        const now = new Date();
        const lastUpdate = new Date(state.last_update || now.toISOString());
        const elapsedSeconds = (now - lastUpdate) / 1000;
        const secondsInDay = 86400;
        const decayAmount = (elapsedSeconds / secondsInDay) * 100;

        const currentHP = Math.max(0, Math.min(100, state.hp - decayAmount));
        const currentSTM = Math.max(0, Math.min(100, state.stm - decayAmount));

        const elLvl = document.getElementById("lvl");
        if (elLvl) elLvl.innerText = `LVL ${state.lvl}`;
        const elTitle = document.getElementById("user-title");
        if (elTitle) elTitle.innerText = getTitle(state.lvl);
        const xpText = document.getElementById("xp-text");
        if (xpText) xpText.innerText = `XP: ${state.xp}/100`;
        const xpBar = document.getElementById("xp-bar");
        if (xpBar) xpBar.style.width = `${state.xp}%`;
        const elHp = document.getElementById("hp");
        if (elHp) elHp.innerText = `${Math.round(currentHP)}%`;
        const elStm = document.getElementById("stm");
        if (elStm) elStm.innerText = `${Math.round(currentSTM)}%`;
        const elInt = document.getElementById("int");
        if (elInt) elInt.innerText = state.int;
        const elSpr = document.getElementById("spr");
        if (elSpr) elSpr.innerText = state.spr;
        updateStreakUI();
    } catch (e) {
        debugLog(`Erro crítico no updateUI: ${e}`);
    }
}

async function updateStreakUI() {
    try {
        const streak = await tauriInvoke("get_streak");
        const streakEl = document.getElementById("streak-display");
        if (streakEl) streakEl.innerText = `🔥 ${streak}`;
    } catch (e) { debugLog(`Error updating streak: ${e}`); }
}

async function loadData() {
    try {
        const data = await tauriInvoke("load_user_data");
        if (data) { 
            state = data; 
            updateUI(); 
            await refreshQuests(); 
        }
    } catch (e) { debugLog(`Error loading data: ${e}`); }
}

async function saveData() {
    try { 
        await tauriInvoke("update_user_stats", { stats: state }); 
    } catch (e) { debugLog(`Error saving data: ${e}`); }
}

window.openMenu = function(type) {
    const menu = MENU_DATA[type];
    if (!menu) return;
    document.getElementById("modal-title").innerText = menu.title;
    document.getElementById("modal-options").style.display = "grid";
    document.getElementById("modal-input-area").style.display = "none";
    const optionsDiv = document.getElementById("modal-options");
    optionsDiv.innerHTML = "";
    menu.options.forEach(opt => {
        const btn = document.createElement("button");
        btn.className = "option-btn";
        btn.innerText = opt.label;
        btn.onclick = () => applyBonus(opt);
        optionsDiv.appendChild(btn);
    });
    document.getElementById("menu-modal").classList.add("active");
}

window.closeMenu = function() {
    document.getElementById("menu-modal").classList.remove("active");
    currentBreakdownId = null;
}

async function applyBonus(option) {
    try {
        const now = new Date();
        const lastUpdate = new Date(state.last_update || now.toISOString());
        const elapsedSeconds = (now - lastUpdate) / 1000;
        const decayAmount = (elapsedSeconds / 86400) * 100;

        debugLog(`Applying bonus: ${option.label}`);
        debugLog(`Elapsed: ${Math.round(elapsedSeconds)}s, Decay: ${decayAmount.toFixed(2)}%`);

        let realHP = Math.max(0, Math.min(100, state.hp - decayAmount));
        let realSTM = Math.max(0, Math.min(100, state.stm - decayAmount));
        let realSPR = Math.max(0, Math.min(100, state.spr - decayAmount));

        debugLog(`Real values before bonus -> HP: ${realHP.toFixed(2)}, STM: ${realSTM.toFixed(2)}, SPR: ${realSPR.toFixed(2)}`);

        let newHP = realHP;
        let newSTM = realSTM;
        let newSPR = realSPR;

        if (option.hp) newHP = Math.min(100, realHP + option.hp);
        if (option.stm) newSTM = Math.min(100, realSTM + option.stm);
        if (option.spr) newSPR = Math.min(100, realSPR + option.spr);

        state.hp = Math.round(newHP);
        state.stm = Math.round(newSTM);
        state.spr = Math.round(newSPR);

        debugLog(`New state values -> HP: ${state.hp}, STM: ${state.stm}, SPR: ${state.spr}`);
        
        if (option.xp) state.xp += option.xp;
        if (state.xp >= 100) { 
            state.lvl++; 
            state.xp -= 100; 
            alert("LEVEL UP! 🌟"); 
        }
        
        let activity = "";
        let value = 0;
        if (option.value !== undefined) {
            value = option.value;
            if (option.hp) activity = "water";
            else if (option.stm) activity = "food";
            else if (option.spr) activity = "mood";
        } else {
            if (option.hp) { activity = "water"; value = option.hp; }
            else if (option.stm) { activity = "food"; value = option.stm; }
            else if (option.spr) { activity = "mood"; value = option.spr; }
        }
        
        state.last_update = now.toISOString();
        updateUI(); 
        await tauriInvoke("update_user_stats", { stats: state, activity: activity, value: value }); 
        window.closeMenu();
    } catch (e) {
        debugLog(`Erro ao aplicar bônus: ${e}`);
        alert(`Erro ao aplicar bônus: ${e}`);
    }
}

async function updateStatsManually(hpBonus = 0, stmBonus = 0, sprBonus = 0, xpBonus = 0) {
    try {
        const now = new Date();
        const lastUpdate = new Date(state.last_update || now.toISOString());
        const elapsedSeconds = (now - lastUpdate) / 1000;
        const decayAmount = (elapsedSeconds / 86400) * 100;

        let realHP = Math.max(0, Math.min(100, state.hp - decayAmount));
        let realSTM = Math.max(0, Math.min(100, state.stm - decayAmount));
        let realSPR = Math.max(0, Math.min(100, state.spr - decayAmount));

        state.hp = Math.round(Math.max(0, Math.min(100, realHP + hpBonus)));
        state.stm = Math.round(Math.max(0, Math.min(100, realSTM + stmBonus)));
        state.spr = Math.round(Math.max(0, Math.min(100, realSPR + sprBonus)));
        
        if (xpBonus) state.xp += xpBonus;
        if (state.xp >= 100) { 
            state.lvl++; 
            state.xp -= 100; 
            alert("LEVEL UP! 🌟"); 
        }

        state.last_update = now.toISOString();
        updateUI();
        await tauriInvoke("update_user_stats", { stats: state });
    } catch (e) {
        debugLog(`Erro ao atualizar stats manualmente: ${e}`);
    }
}


window.addQuest = async function() {
    const input = document.getElementById("quest-input");
    const text = input.value.trim();
    if (!text) return;
    try {
        await tauriInvoke("add_new_quest", { text: text, parentId: -1 });
        input.value = "";
        await window.refreshQuests();
    } catch (e) { alert(`Erro: ${e}`); }
}

window.confirmAddSubtask = async function() {
    const input = document.getElementById("modal-text-input");
    const text = input.value.trim();
    if (!text || !currentBreakdownId) return;
    try {
        await tauriInvoke("add_new_quest", { text: text, parentId: currentBreakdownId });
        input.value = "";
        await window.refreshQuests();
        window.closeMenu();
    } catch (e) { alert(`Erro: ${e}`); }
}

window.refreshQuests = async function() {
    try {
        const quests = await tauriInvoke("fetch_quests");
        if (!quests) return;
        const list = document.getElementById("quest-list");
        list.innerHTML = "";
        const questMap = {};
        const roots = [];
        quests.forEach(q => { questMap[q.id] = { ...q, children: [] }; });
        quests.forEach(q => {
            if (q.parent_id !== null) {
                if (questMap[q.parent_id]) questMap[q.parent_id].children.push(questMap[q.id]);
            } else {
                roots.push(questMap[q.id]);
            }
        });
function renderQuest(q, depth = 0, container = document.getElementById("quest-list")) {
    const li = document.createElement("li");
    li.className = `quest-item ${depth > 0 ? "subtask" : ""}`;
    li.style.marginLeft = `${depth * 20}px`;
    li.dataset.id = q.id;
    li.draggable = false; 
    li.style.cursor = "grab";
    
    const contentDiv = document.createElement("div");
    contentDiv.className = "quest-content";
    contentDiv.style.display = "flex";
    contentDiv.style.alignItems = "center";
    contentDiv.style.gap = "12px";
    contentDiv.style.width = "100%";
    
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = q.is_completed;
            checkbox.onclick = async () => {
                try {
                    const completedValue = checkbox.checked;
                    const qid = q.id;
                    debugLog(`Sensus: Requesting toggle_quest for ID ${qid} to ${completedValue}`);
                    
                    const updatedStats = await tauriInvoke("toggle_quest", { id: qid, completed: completedValue });
                    
                    if (!updatedStats) {
                        alert("Erro: Falha na comunicação com o backend ao salvar missão.");
                        checkbox.checked = !checkbox.checked;
                        return;
                    }
                    
                    state = updatedStats;
                    updateUI();
                    await window.refreshQuests();
                } catch (e) { 
                    alert(`Erro ao salvar missão: ${e}`); 
                    checkbox.checked = !checkbox.checked;
                }
            };
    const span = document.createElement("span");
    const timeDisplay = q.due_time ? ` <span style="font-size: 0.8em; opacity: 0.7; margin-left: 8px;">${q.due_time}</span>` : "";
    span.innerHTML = `${q.task_text}${timeDisplay}`;
    if (q.is_completed) span.style.textDecoration = "line-through";
    contentDiv.appendChild(checkbox);
    contentDiv.appendChild(span);
    const optionsBtn = document.createElement("button");
    optionsBtn.className = "options-btn";
    optionsBtn.innerText = "⋮";
    optionsBtn.onclick = () => window.openQuestOptions(q.id, q.task_text);
    contentDiv.appendChild(optionsBtn);
    const hammer = document.createElement("button");
    hammer.className = "hammer-btn";
    hammer.innerText = "🔨";
    hammer.onclick = () => window.openBreakdownMenu(q.id, q.task_text);
    contentDiv.appendChild(hammer);
    li.appendChild(contentDiv);
    
    li.onmousedown = (e) => {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'BUTTON') return;
        
        draggedElement = li;
        li.classList.add("dragging");
        li.style.pointerEvents = "none";
        
        const onMouseMove = (moveEvent) => {
            const target = document.elementFromPoint(moveEvent.clientX, moveEvent.clientY);
            const closestItem = target?.closest(".quest-item");
            
            if (closestItem && closestItem !== draggedElement) {
                const rect = closestItem.getBoundingClientRect();
                const midpoint = rect.top + rect.height / 2;
                
                if (moveEvent.clientY < midpoint) {
                    closestItem.parentNode.insertBefore(draggedElement, closestItem);
                } else {
                    closestItem.parentNode.insertBefore(draggedElement, closestItem.nextSibling);
                }
                closestItem.classList.add("drag-over");
            }
        };
        
        const onMouseUp = async () => {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
            
            li.classList.remove("dragging");
            li.style.pointerEvents = "auto";
            document.querySelectorAll(".quest-item").forEach(item => item.classList.remove("drag-over"));
            
            try {
                const allItems = Array.from(document.querySelectorAll(".quest-item"));
                for (let i = 0; i < allItems.length; i++) {
                    const id = parseInt(allItems[i].dataset.id);
                    if (!isNaN(id)) {
                        await tauriInvoke("update_quest_position", { id: id, position: i });
                    }
                }
                debugLog("Positions updated in DB");
            } catch (err) { 
                debugLog(`Erro ao salvar ordem: ${err}`); 
                await window.refreshQuests();
            }
            draggedElement = null;
        };
        
        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", onMouseUp);
    };

    const childrenContainer = document.createElement("div");
    childrenContainer.className = "children-container";
    q.children.forEach(child => {
        renderQuest(child, depth + 1, childrenContainer);
    });
    li.appendChild(childrenContainer);
    
    container.appendChild(li);
}

        roots.sort((a, b) => a.position - b.position || a.id - b.id).forEach(root => renderQuest(root));
    } catch (e) { debugLog(`Error refreshing quests: ${e}`); }
}

window.openQuestOptions = function(qid, text) {
    currentBreakdownId = qid;
    document.getElementById("modal-title").innerText = `Opções: ${text}`;
    document.getElementById("modal-options").style.display = "grid";
    document.getElementById("modal-input-area").style.display = "none";
    const optionsDiv = document.getElementById("modal-options");
    optionsDiv.innerHTML = "";
    const cancelBtn = document.createElement("button");
    cancelBtn.className = "option-btn";
    cancelBtn.innerText = "❌ Cancelar Missão";
    cancelBtn.onclick = async () => {
        try {
            await tauriInvoke("cancel_quest", { id: qid });
            window.closeMenu();
            await window.refreshQuests();
        } catch (e) { alert(`Erro: ${e}`); }
    };
    const rescheduleBtn = document.createElement("button");
    rescheduleBtn.className = "option-btn";
    rescheduleBtn.innerText = "📅 Reagendar";
            rescheduleBtn.onclick = () => {
                document.getElementById("modal-options").style.display = "none";
                document.getElementById("modal-input-area").style.display = "block";
                document.getElementById("modal-title").innerText = "Nova Data";
                const dateInput = document.getElementById("modal-text-input");
                dateInput.type = "date";
                dateInput.placeholder = "";
                const confirmBtn = document.querySelector(".confirm-btn");
                const originalFn = confirmBtn.onclick;
                confirmBtn.innerText = "Salvar Data 📅";
                confirmBtn.onclick = async () => {
                    const date = dateInput.value;
                    if (!date) {
                        alert("Por favor, selecione uma data.");
                        return;
                    }
                    try {
                        await tauriInvoke("reschedule_quest", { id: qid, date: date });
                        window.closeMenu();
                        await window.refreshQuests();
                    } catch (e) { alert(`Erro: ${e}`); }
                    confirmBtn.innerText = "Adicionar 🔨";
                    confirmBtn.onclick = originalFn;
                };
            };
    optionsDiv.appendChild(cancelBtn);
    optionsDiv.appendChild(rescheduleBtn);
    document.getElementById("menu-modal").classList.add("active");
}

window.openBreakdownMenu = function(qid, text) {
    currentBreakdownId = qid;
    document.getElementById("modal-title").innerText = `🔨 Decompor: ${text}`;
    document.getElementById("modal-options").style.display = "none";
    document.getElementById("modal-input-area").style.display = "block";
    const input = document.getElementById("modal-text-input");
    input.type = "text";
    input.value = "";
    input.placeholder = "Nome da submissão...";
    const confirmBtn = document.querySelector(".confirm-btn");
    confirmBtn.innerText = "Adicionar 🔨";
    confirmBtn.onclick = window.confirmLAddSubtask;
    document.getElementById("menu-modal").classList.add("active");
}

window.confirmLAddSubtask = async function() {
    const input = document.getElementById("modal-text-input");
    const text = input.value.trim();
    if (!text || !currentBreakdownId) return;
    try {
        await tauriInvoke("add_new_quest", { text: text, parentId: currentBreakdownId });
        input.value = "";
        await window.refreshQuests();
        window.closeMenu();
    } catch (e) { alert(`Erro: ${e}`); }
}

window.openMedicationsMenu = async function() {
    document.getElementById("modal-title").innerText = "💊 Meus Remédios";
    document.getElementById("modal-options").style.display = "grid";
    document.getElementById("modal-input-area").style.display = "block";
    
    const optionsDiv = document.getElementById("modal-options");
    optionsDiv.innerHTML = "";
    
    try {
        const meds = await tauriInvoke("get_medications");
        meds.forEach(med => {
            const row = document.createElement("div");
            row.className = "option-btn";
            row.style.display = "flex";
            row.style.alignItems = "center";
            row.style.justifyContent = "space-between";
            row.style.cursor = "default";
            row.style.padding = "10px";
            row.style.backgroundColor = "rgba(255,255,255,0.1)";
            row.style.borderRadius = "8px";
            row.style.marginBottom = "10px";

            const checkbox = document.createElement("input");
            checkbox.type = "checkbox";
            checkbox.checked = med.is_taken;
            checkbox.onclick = async () => {
                try {
                    const takenValue = checkbox.checked ? 1 : 0;
                    const medId = med.id;
                    debugLog(`Attempting to toggle medication ID ${medId} to ${takenValue}`);
                    
                    await tauriInvoke("toggle_medication", { id: medId, isTaken: takenValue });
                    
                    if (checkbox.checked) {
                        await updateStatsManually(5, 5, 0, 5);
                        debugLog(`Medication ${medId} taken: Bonus applied`);
                    } else {
                        debugLog(`Medication ${medId} unchecked`);
                    }
                } catch (e) {
                    console.error(`Error toggling medication ${med.id}:`, e);
                    alert(`Erro ao salvar remédio: ${e}`);
                    checkbox.checked = !checkbox.checked; 
                }
            };

            const label = document.createElement("span");
            label.innerText = med.name;
            label.style.flexGrow = "1";
            label.style.marginLeft = "10px";

            const delBtn = document.createElement("button");
            delBtn.innerText = "❌";
            delBtn.style.background = "none";
            delBtn.style.border = "none";
            delBtn.style.color = "#ff5555";
            delBtn.style.cursor = "pointer";
            delBtn.onclick = async () => {
                await tauriInvoke("delete_medication", { id: med.id });
                window.openMedicationsMenu();
            };

            row.appendChild(checkbox);
            row.appendChild(label);
            row.appendChild(delBtn);
            optionsDiv.appendChild(row);
        });

        const closeBtn = document.createElement("button");
        closeBtn.className = "option-btn";
        closeBtn.innerText = "Confirmar e Fechar ✅";
        closeBtn.style.marginTop = "20px";
        closeBtn.style.backgroundColor = "rgba(163, 190, 140, 0.5)";
        closeBtn.onclick = () => window.closeMenu();
        optionsDiv.appendChild(closeBtn);

    } catch (e) { alert(`Erro ao carregar remédios: ${e}`); }

    const input = document.getElementById("modal-text-input");
    input.placeholder = "Nome do novo remédio...";
    const footerConfirmBtn = document.querySelector("#modal-input-area .confirm-btn");
    if (footerConfirmBtn) {
        footerConfirmBtn.innerText = "Adicionar 💊";
        footerConfirmBtn.onclick = async () => {
            const text = input.value.trim();
            if (!text) return;
            try {
                await tauriInvoke("add_medication", { name: text });
                input.value = "";
                window.openMedicationsMenu();
            } catch (e) { alert(`Erro: ${e}`); }
        };
    }
    
    document.getElementById("menu-modal").classList.add("active");
}

window.openSleepMenu = function() {
    document.getElementById("modal-title").innerText = "😴 Qualidade do Sono";
    document.getElementById("modal-options").style.display = "grid";
    document.getElementById("modal-input-area").style.display = "none";
    
    const optionsDiv = document.getElementById("modal-options");
    optionsDiv.innerHTML = "";
    
    const qualities = [
        { label: "🌟 Excelente", value: "Excelente" },
        { label: "✅ Bom", value: "Bom" },
        { label: "😐 Regular", value: "Regular" },
        { label: "❌ Ruim", value: "Ruim" }
    ];
    
    qualities.forEach(q => {
        const btn = document.createElement("button");
        btn.className = "option-btn";
        btn.innerText = q.label;
        btn.onclick = async () => {
            try {
                await tauriInvoke("log_sleep", { quality: q.value });
                
                let hpB = 0, stmB = 0, sprB = 0, xpB = 0;
                if (q.value === "Excelente") { hpB = 40; stmB = 40; sprB = 10; xpB = 20; }
                else if (q.value === "Bom") { hpB = 25; stmB = 25; sprB = 5; xpB = 10; }
                else if (q.value === "Regular") { hpB = 10; stmB = 10; sprB = 0; xpB = 5; }
                else if (q.value === "Ruim") { hpB = 0; stmB = 0; sprB = -10; xpB = 5; }
                
                await updateStatsManually(hpB, stmB, sprB, xpB);
                alert(`Sono registrado como ${q.value}! 😴`);
                window.closeMenu();
            } catch (e) { alert(`Erro: ${e}`); }
        };
        optionsDiv.appendChild(btn);
    });
    
    document.getElementById("menu-modal").classList.add("active");
}

window.syncCalendar = async function() {

    const btn = document.getElementById("sync-btn");
    const originalText = btn.innerHTML;
    try {
        btn.disabled = true;
        btn.innerHTML = '<span class="btn-icon">⏳</span> Sincronizando...';
        const result = await tauriInvoke("sync_google_calendar");
        alert(result);
        await window.refreshQuests();
    } catch (e) {
        alert(`Erro na sincronização: ${e}`);
    } finally {
        btn.disabled = false;
        btn.innerHTML = originalText;
    }
}

async function sendNotification(title, body) {
    try {
        // 1. Notificação Nativa do Navegador (Mais confiável no WebView2)
        if ("Notification" in window && Notification.permission === "granted") {
            new Notification(title, { 
                body: body, 
                icon: 'src-tauri/icons/icon.png' 
            });
            debugLog("Browser notification sent");
        }
        
        // 2. Notificação do Tauri
        if (window.__TAURI__ && window.__TAURI__.notification) {
            await window.__TAURI__.notification.send({ title, body });
            debugLog("Tauri notification sent");
        }
        
        // 3. Notificação via Ponte Telegram
        await fetch("http://127.0.0.1:8000/send_notification", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ title, body })
        }).catch(e => console.error("Telegram notification failed:", e));
        debugLog("Telegram notification request sent");

    } catch (e) { 
        console.error("Notification error:", e); 
        debugLog(`Notification error: ${e}`);
    }
}

function setupReminders() {
    setInterval(() => { sendNotification("🌈 Momento de Pausa", "Respire fundo. Como você está se sentindo agora?"); }, 6 * 60 * 60 * 1000);
    setInterval(() => {
        const now = new Date();
        const lastUpdate = new Date(state.last_update || now.toISOString());
        const elapsedSeconds = (now - lastUpdate) / 1000;
        const decayPercent = (elapsedSeconds / 86400) * 100;
        const currentHP = Math.max(0, Math.min(100, state.hp - decayPercent));
        const currentSTM = Math.max(0, Math.min(100, state.stm - decayPercent));
        if (currentHP < 30) sendNotification("⚠️ Alerta de Energia", "Seu HP está crítico! Hora de se alimentar para recuperar suas forças. 🍲");
        if (currentSTM < 30) sendNotification("⚠️ Alerta de Hidratação", "Sua Stamina está baixa! Beba água agora para não perder o ritmo. 💧");
    }, 15 * 60 * 1000);
}

function setupWaterTooltip() {
    debugLog("Attempting to setupWaterTooltip");
    const waterBtn = document.getElementById("water-btn");
    const tooltip = document.getElementById("custom-tooltip");
    if (!waterBtn || !tooltip) {
        debugLog("ERROR: waterBtn or tooltip not found");
        return;
    }

    const handleMouseEnter = async () => {
        debugLog("Mouse Enter detected!");
        tooltip.innerText = "Carregando... ⏳";
        tooltip.style.display = "block";
        tooltip.style.zIndex = "9999";
        try {
            debugLog("Requesting daily total from Tauri...");
            const total = await tauriInvoke("get_daily_total", { activityType: "water" });
            debugLog(`Total water received: ${total}`);
            tooltip.innerText = `Hoje você já ingeriu: ${total || 0} ml 💧`;
        } catch (e) { 
            debugLog(`Error fetching water total: ${e}`); 
            tooltip.innerText = "Erro ao carregar dados ❌";
        }
    };

    const handleMouseMove = (e) => {
        tooltip.style.left = `${e.clientX + 15}px`;
        tooltip.style.top = `${e.clientY + 15}px`;
    };

    const handleMouseLeave = () => {
        debugLog("Mouse Leave detected");
        tooltip.style.display = "none";
    };

    waterBtn.addEventListener("mouseenter", handleMouseEnter);
    waterBtn.addEventListener("mousemove", handleMouseMove);
    waterBtn.addEventListener("mouseleave", handleMouseLeave);
    waterBtn.onmouseenter = handleMouseEnter;
    waterBtn.onmousemove = handleMouseMove;
    waterBtn.onmouseleave = handleMouseLeave;
    debugLog("Water Tooltip setup complete");
}

window.openStats = function() {
    document.getElementById("stats-modal").classList.add("active");
    renderActivityChart();
}

window.closeStats = function() {
    document.getElementById("stats-modal").classList.remove("active");
}

async function updateChartMetrics() {
    const checkboxes = document.querySelectorAll(".metric-checkbox input");
    activeMetrics = Array.from(checkboxes)
        .filter(i => i.checked)
        .map(i => i.value);
    
    await renderActivityChart();
}
window.updateChartMetrics = updateChartMetrics;

async function renderActivityChart(range = 'week') {
    try {
        debugLog(`Rendering Activity Chart (Range: ${range})...`);
        const logs = await tauriInvoke("get_activity_logs");
        const sleepLogs = await tauriInvoke("get_sleep_logs");
        
        if ((!logs || logs.length === 0) && (!sleepLogs || sleepLogs.length === 0)) {
            debugLog("No data available to render chart.");
            return;
        }

        const GOALS = { water: 2000, food: 100, mood: 50, sleep: 4 };
        const daysToFetch = range === 'week' ? 7 : 30;
        const now = new Date();
        const labels = [];
        const waterData = [];
        const foodData = [];
        const moodData = [];
        const sleepData = [];
        const sleepMapping = { "Excelente": 4, "Bom": 3, "Regular": 2, "Ruim": 1 };

        for (let i = daysToFetch - 1; i >= 0; i--) {
            const d = new Date();
            d.setDate(now.getDate() - i);
            const dateStr = d.toISOString().split('T')[0];
            labels.push(dateStr);
            
            const dayLogs = (logs || []).filter(l => l[0].startsWith(dateStr));
            
            const wSum = dayLogs.filter(l => l[1] === "water").reduce((sum, l) => sum + l[2], 0);
            waterData.push((wSum / GOALS.water) * 100);
            
            const fSum = dayLogs.filter(l => l[1] === "food").reduce((sum, l) => sum + l[2], 0);
            foodData.push((fSum / GOALS.food) * 100);
            
            const mSum = dayLogs.filter(l => l[1] === "mood").reduce((sum, l) => sum + l[2], 0);
            moodData.push((mSum / GOALS.mood) * 100);
            
            const daySleep = sleepLogs ? sleepLogs.find(s => s[0] === dateStr) : null;
            const sleepVal = daySleep ? (sleepMapping[daySleep[1]] || 0) : 0;
            sleepData.push((sleepVal / GOALS.sleep) * 100);
        }

        const ctx = document.getElementById("activityChart").getContext("2d");
        if (window.activityChartInstance) window.activityChartInstance.destroy();
        
        const allDatasets = [
            { label: 'Água (%)', data: waterData, borderColor: '#88c0d0', backgroundColor: 'rgba(136, 192, 208, 0.2)', fill: true, tension: 0.3 },
            { label: 'Comida (%)', data: foodData, borderColor: '#a3be8c', backgroundColor: 'rgba(163, 190, 140, 0.2)', fill: true, tension: 0.3 },
            { label: 'Humor (%)', data: moodData, borderColor: '#b48ead', backgroundColor: 'rgba(180, 142, 142, 0.2)', fill: true, tension: 0.3 },
            { label: 'Sono (%)', data: sleepData, borderColor: '#81a1c1', backgroundColor: 'rgba(129, 161, 193, 0.2)', fill: true, tension: 0.3 }
        ];

        const filteredDatasets = allDatasets.filter(ds => {
            const label = ds.label.toLowerCase();
            if (label.includes('água')) return activeMetrics.includes('water');
            if (label.includes('comida')) return activeMetrics.includes('food');
            if (label.includes('humor')) return activeMetrics.includes('mood');
            if (label.includes('sono')) return activeMetrics.includes('sleep');
            return false;
        });

        window.activityChartInstance = new Chart(ctx, {
            type: 'line',
            data: {
                labels: labels,
                datasets: filteredDatasets
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: { 
                        beginAtZero: true, 
                        title: { display: true, text: 'Progresso (%)', color: '#d8dee9' },
                        grid: { color: 'rgba(255,255,255,0.1)' }, 
                        ticks: { color: '#d8dee9', callback: value => value + '%' } 
                    },
                    x: { grid: { display: false }, ticks: { color: '#d8dee9' } }
                },
                plugins: { legend: { labels: { color: '#d8dee9' } } }
            }
        });
    } catch (e) { 
        debugLog(`Error rendering chart: ${e}`);
        console.error(e);
    }
}


async function initSensus() {
    debugLog("Init Sensus started");
    setupWaterTooltip();
    try {
        // Solicitar permissão de notificação do navegador
        if ("Notification" in window) {
            if (Notification.permission !== "granted" && Notification.permission !== "denied") {
                await Notification.requestPermission();
                debugLog(`Notification permission requested: ${Notification.permission}`);
            }
        }

        debugLog("Loading user data...");
        await loadData();
        debugLog("Data loaded. Setting up reminders...");
        setupReminders();
        
        // Sincronização automática ao abrir
        debugLog("Auto-syncing calendar on startup...");
        await syncCalendar();

        // TESTE DE NOTIFICAÇÃO: Dispara assim que o app abre para testar permissões do Windows
        sendNotification("Sensus Iniciou! 🌙", "Se você está vendo isso, as notificações do Windows estão funcionando perfeitamente.");
    } catch (e) {
        debugLog(`Erro na inicialização: ${e}`);
    }
    
    // Verificação de virada de dia e update de UI a cada minuto
    setInterval(() => {
        updateUI();
        checkDayChange();
    }, 60000);
    
    debugLog("Sensus fully initialized");
}

// Função para verificar se o dia mudou e sincronizar calendário
let lastCheckedDate = new Date().toDateString();
async function checkDayChange() {
    const today = new Date().toDateString();
    if (today !== lastCheckedDate) {
        debugLog("Day change detected! Auto-syncing calendar...");
        await syncCalendar();
        lastCheckedDate = today;
    }
}

if (document.readyState === "complete" || document.readyState === "interactive") {
    initSensus();
} else {
    window.addEventListener('DOMContentLoaded', initSensus);
}
