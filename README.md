# 🌙 Sensus

Sensus is a gamified life-tracking application designed to help users maintain healthy habits and track their personal growth through RPG-like mechanics.

## ✨ Features

- **Comprehensive Stat Tracking**: Monitor your growth in HP (Health), STM (Stamina), INT (Intelligence), and SPR (Spirit).
- **Gamified Habit Modules**:
    - 💧 **Hydration**: Log water intake with real-time daily totals.
    - 🍲 **Nutrition**: Track daily meals and nutritional habits.
    - 🌈 **Mood**: Log your emotional state to balance your Spirit.
    - 🧼 **Hygiene**: Quick actions for daily care (Shower, Teeth, Skin Care) with XP rewards.
- **Advanced Quest System**: Organize your life with a hierarchical quest and subtask system, featuring a smooth drag-and-drop interface for custom priority.
- **Wellness Tracking**: 
    - 😴 **Sleep**: Log sleep quality to receive status bonuses (HP/STM/SPR) at the start of your day.
    - 💊 **Medications**: A daily checklist for meds with XP incentives and automatic daily resets.
- **Consistency & Analytics**:
    - **Daily Streaks**: Visual "Fire" streak to motivate daily consistency.
    - **Normalized Charts**: Interactive 7-day and 30-day progress charts using a percentage-based scale for balanced visualization of different metrics.
- **Smart Lifecycle**: Automatic daily reset of energy (HP/STM) and medication checks, synchronized with the user's local timezone.

## 🛠️ Tech Stack

- **Frontend**: JavaScript, HTML5, CSS3 (Glassmorphism Design / Nord Palette)
- **Backend**: Rust (via Tauri)
- **Database**: SQLite (Local Persistence)
- **Charts**: Chart.js

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v24+)

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/sensus.git
   cd sensus
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the app in development mode:
   ```bash
   npm run tauri dev
   ```
4. To build the final executable:
   ```bash
   npm run tauri build
   ```

## 🎨 Design
Sensus uses the **Nord color palette** combined with a **Glassmorphism** aesthetic for a modern, clean, and calming user experience, ensuring that habit tracking feels like a rewarding game rather than a chore.
