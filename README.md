# 🐉 Calabozos y Dragones Online! ⚔️ / Dungeons & Dragons Real-Time Crawler!

A high-performance, real-time multiplayer web dungeon crawler inspired by the mechanics of classic snakes and ladders, reimagined with an epic **Calabozos y Dragones** (Dungeons & Dragons / Fantasy Dungeon Crawler) identity. Built with a robust **Rust** backend (Axum, Tokio, WebSockets) and a modern, responsive, bilingual vanilla JS client with zero external audio assets. Features interactive 3D d6 dice, animated step-by-step hero token hops, runic ladders, majestic horned dragons, battle cries & spell shouts, and instant Dragon Slayer shareable victory certificates.

---

## 🇺🇸 English Guide

### 🌟 Key Features

1. **High-Fantasy Dungeon Board (100 & 50 Tiles)**:
   - **Dragon's Spire (Torre del Dragón - 100 Tiles)**: 10×10 boustrophedon dungeon grid leading to the dragon's hoard at Tile 100.
   - **Crypt Express (Cripta Express - 50 Tiles)**: Fast-paced 5×10 dungeon for quick 3–5 minute expeditions.
   - **8 Runic & Iron Ladders**: 4➔14, 9➔31, 20➔38, 28➔84, 40➔59, 51➔67, 63➔81, 71➔91 forged with ancient rungs and ascending brass fanfare arpeggios.
   - **8 Fearsome Dragons**: 17➔7, 54➔34, 62➔19, 64➔60, 87➔24, 93➔73, 95➔75, 99➔78 rendered as majestic dragons with horns, dorsal spine crests, flame breath, and tail spades.

2. **Real-Time Multiplayer (Rust + Axum WebSockets)**:
   - Private dungeon rooms with fantasy 6-character room codes (e.g. `DRAG12`, `CALA34`, `MAGO88`, `RUNA55`).
   - One-click shareable invite links (`?room=CODE`) with automatic session resume on reconnect.
   - Automatic party leader migration if the host disconnects.
   - Up to 10 simultaneous adventurers with D&D hero class avatars (`🧙‍♂️` Wizard, `⚔️` Warrior, `🏹` Ranger, `🧝` Rogue, `🛡️` Paladin, `🧌` Barbarian, `🐉` Dragonborn, `🧝‍♀️` Sorceress, `🧛` Necromancer, `🪕` Bard) and hero colors.
   - **🤖 Party Companion Bots**: Play solo or assemble an adventuring party with up to 9 automated bots (`🤖 Sir Galahad`, `🤖 Gandalf`, `🤖 Legolas`, `🤖 Conan`, `🤖 Ignis Dragón`, etc.) featuring ~1s turn cadence and spontaneous battle cries.

3. **✨ Casillas Mágicas (Magic & Mystery Tiles)**:
   - Optional host toggle in lobby enabling enchanted dungeon tiles:
     - 🧪 **Poción / Potion**: +3 tiles forward haste boost.
     - 📜 **Pergamino / Scroll**: +5 tiles arcane teleport sprint.
     - 📦 **Cofre / Chest**: Free bonus roll from the treasure hoard!
     - 🕸️ **Telaraña / Web Trap**: Ensnared in spiderwebs (skips 1 turn).

4. **🏆 Hall of Champions, Guild Honors & 📜 Dragon Slayer Certificate**:
   - 🥇 🥈 🥉 Celebratory end-game podium showcasing 1st, 2nd, and 3rd place adventurers with radiant gold aura.
   - 4 honorary Guild badges calculated from dungeon telemetry:
     - 🧗 *Conquistador de Cimas / Peak Conqueror*: Most runic ladders ascended.
     - 🐉 *Domador de Dragones / Dragon Tamer*: Most dragon strikes endured.
     - 🎲 *Mano del Destino / Hand of Destiny*: Fewest rolls to victory.
     - ⚡ *Héroe Veloz / Swift Hero*: Match winner.
   - **📜 Dragon Slayer Certificate of Valor**: Generates an 800×1000 obsidian canvas certificate with gilded borders, hero avatar, stats, and Guild honors, ready for 1-click Web Share and PNG download.

5. **⚔️ Battle Cries, Spells & Live Shouts**:
   - Sleek horizontal touch scroll and drag-to-scroll battle shout bar.
   - **Automatic Real-Time Bilingual Translation**:
     - `Beware the Dragon! 🐉` <-> `¡Cuidado con el Dragón! 🐉`
     - `For glory and honor! ⚔️` <-> `¡Por el honor y la gloria! ⚔️`
     - `Saving throw! 🛡️` <-> `¡Tirada de salvación! 🛡️`
     - `Critical hit! 💥` <-> `¡Golpe crítico! 💥`
     - `Critical fail! 💀` <-> `¡Fallo crítico! 💀`
     - `Roll for initiative! 🎲` <-> `¡Lanza iniciativa! 🎲`
     - `Drinking a potion! 🧪` <-> `¡Bebiendo poción! 🧪`
   - Floating speech bubbles popping over the board with hero class avatar and name.
   - Procedural Web Audio sounds with browser voice locution.

6. **📲 Progressive Web App (PWA) 1-Click Install**:
   - Native header installation button (`#btn-install-pwa`) hooking into `beforeinstallprompt` for smooth 1-tap installation on Android, iOS, Windows, and macOS.

7. **Mobile-First Responsive Design (Single-Screen Experience)**:
   - Dynamic `dvh` scaling (`min(94vw, calc(100dvh - 235px), 500px)`) ensuring header, dungeon board, dice roller, and battle shout bar fit on any smartphone without vertical scrolling.
   - Slide-up bottom drawer for the Dungeon Chronicle Feed (`📜`) on mobile phones.

8. **Customizable Host Rules**:
   - 🎯 **Exact 100 Win**: Bounce back if overshooting the Dragon Hoard (Yes / No).
   - ✨ **Extra Roll on 6**: Consecutive turn when rolling a maximum roll.
   - ⚠️ **Three Consecutive 6s Penalty**: Wild magic surge sends hero back to tile 1.
   - ⏱️ **Turn Timer**: 15s, 25s, 40s, or Unlimited with auto-roll for AFK adventurers (adjustable live mid-game).

9. **🎨 High-Fantasy Themes & Bilingual Toggle**:
   - 🏰 **Calabozo de Obsidiana** (Default deep slate/granite dungeon).
   - 🐉 **Guarida del Dragón Rojo** (Volcanic fiery magma).
   - 🔮 **Sanctum Arcano del Mago** (Arcane starlight mana & violet sapphire).
   - One-click instant language switcher (`🌐 Español` / `🌐 English`).

10. **Security & Production Readiness (82%+ Test Coverage)**:
    - 30 automated Rust unit and integration tests passing in <0.05s.
    - **82.1% code line coverage** verified via `cargo llvm-cov`.
    - `cargo clippy`: Clean with zero warnings (`-D warnings`).
    - Strict input sanitization for adventurer names and avatars against XSS.
    - Security headers (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`).
    - Periodic Tokio background cleanup of inactive/abandoned rooms.
    - Rate-limiting protection on WebSocket reactions.

---

### 🚀 How to Play

#### 1. Start the Server
```powershell
# Run with Cargo in Release mode:
cargo run --release

# Or launch the compiled release binary directly:
.\target\release\shoots_ladders.exe
```

Access the game:
- **On your PC**: [http://localhost:3000](http://localhost:3000)
- **On mobile devices on the same Wi-Fi**: `http://YOUR_LOCAL_IP:3000` (printed in the terminal at startup)

#### 2. Create a Dungeon Room
1. Open [http://localhost:3000](http://localhost:3000).
2. Enter your adventurer name, hero class avatar, and color.
3. Configure your desired dungeon rules (dice count, exact win bounce, turn timer, magic tiles, etc.).
4. Click **🎲 Enter the Dungeon**.

#### 3. Summon Allies
1. Share the 6-character room code, click **📋 Copy Link**, or show the **📱 QR Code** for allies to scan with their camera.
2. Players join from their mobile or desktop browser via the **Join Dungeon** tab.
3. Once the party is assembled, the leader clicks **⚔️ Enter the Dungeon!**.

---

## 🇲🇽 Guía en Español

### 🌟 Características Principales

1. **Tablero de Mazmorra Fantástica (100 y 50 Casillas)**:
   - **Torre del Dragón (100 Casillas)**: Cuadrícula de mazmorra 10×10 que culmina en la guarida y tesoro del dragón en la casilla 100.
   - **Cripta Express (50 Casillas)**: Mazmorra de 5×10 para expediciones rápidas de 3 a 5 minutos.
   - **8 Escaleras Rúnicas**: Portales de ascenso de hierro y roble rúnico (4➔14, 9➔31, 20➔38, 28➔84, 40➔59, 51➔67, 63➔81, 71➔91).
   - **8 Dragones Majestuosos**: Dragones alados con cuernos, crestas dorsales y aliento de fuego (17➔7, 54➔34, 62➔19, 64➔60, 87➔24, 93➔73, 95➔75, 99➔78).

2. **Multijugador en Tiempo Real (Rust + Axum WebSockets)**:
   - Creación de salas privadas con códigos de mazmorra (ej. `DRAG12`, `CALA34`, `MAGO88`, `RUNA55`).
   - Enlace directo compartible (`?room=CODIGO`) con reconexión automática de sesión.
   - Traspaso automático de líder de grupo si el creador abandona la expedición.
   - Hasta 10 aventureros simultáneos con clases de D&D (`🧙‍♂️` Mago, `⚔️` Guerrero, `🏹` Arquero, `🧝` Pícaro, `🛡️` Paladín, `🧌` Bárbaro, `🐉` Dracónido, `🧝‍♀️` Hechicera, `🧛` Nigromante, `🪕` Bardo) y colores de héroe.
   - **🤖 Compañeros de Grupo Bots**: Juega en solitario o recluta hasta 9 bots (`🤖 Sir Galahad`, `🤖 Gandalf`, `🤖 Legolas`, `🤖 Conan`, `🤖 Ignis Dragón`, etc.) con turnos ágiles de ~1s y gritos de batalla espontáneos.

3. **✨ Casillas Mágicas (Pociones, Pergaminos, Cofres y Trampas)**:
   - Casillas especiales en el tablero:
     - 🧪 **Poción**: Impulso rápido de +3 casillas.
     - 📜 **Pergamino**: Teletransportación arcana de +5 casillas.
     - 📦 **Cofre del Tesoro**: ¡Tiro extra consecutivo de recompensa!
     - 🕸️ **Telaraña Trampa**: Quedas atrapado en la red (pierdes 1 turno).

4. **🏆 Salón de Campeones y 📜 Certificado de Matadragones**:
   - 🥇 🥈 🥉 Podio de honor para los aventureros con destellos dorados.
   - 4 condecoraciones de la Orden de Aventureros:
     - 🧗 *Conquistador de Cimas*: El que subió más escaleras rúnicas.
     - 🐉 *Domador de Dragones*: El que enfrentó más dragones y resistió.
     - 🎲 *Mano del Destino*: El más certero con los dados.
     - ⚡ *Héroe Veloz*: El primer aventurero en coronarse Dragon Slayer.
   - **📜 Certificado de Matadragones**: Diploma en canvas (800×1000) con filigrana dorada, avatar de clase, sellos y estadísticas, listo para compartir o descargar en PNG.

5. **⚔️ Gritos de Batalla y Conjuros**:
   - Barra inferior de combate táctil y deslizante:
     - `¡Cuidado con el Dragón! 🐉` <-> `Beware the Dragon! 🐉`
     - `¡Por el honor y la gloria! ⚔️` <-> `For glory and honor! ⚔️`
     - `¡Tirada de salvación! 🛡️` <-> `Saving throw! 🛡️`
     - `¡Golpe crítico! 💥` <-> `Critical hit! 💥`
     - `¡Fallo crítico! 💀` <-> `Critical fail! 💀`
     - `¡Lanza iniciativa! 🎲` <-> `Roll for initiative! 🎲`
     - `¡Bebiendo poción! 🧪` <-> `Drinking a potion! 🧪`
   - Bocadillos flotantes estilo cómic sobre el tablero con avatar y nombre.

6. **🎨 Temas de Alta Fantasía & Bilingüe**:
   - 🏰 **Calabozo de Obsidiana** (Predeterminado - losas de granito y piedra antigua).
   - 🐉 **Guarida del Dragón Rojo** (Volcánico y magma ardiente).
   - 🔮 **Sanctum Arcano del Mago** (Zafiro, violeta y maná místico).
   - Alternancia instantánea con un clic (`🌐 Español` / `🌐 English`).

---

## ☁️ Free Cloud Deployment to Render.com / Despliegue en Render.com

This repository includes a multi-stage [`Dockerfile`](Dockerfile) and a [`render.yaml`](render.yaml) blueprint ready for 100% free hosting:

### English:
1. Push your repository to GitHub: `git push origin main`.
2. Go to [dashboard.render.com](https://dashboard.render.com) and click **New +** → **Blueprint** (or **Web Service**).
3. Connect your repository (`shoots_ladders`).
4. Render automatically detects the `Dockerfile` and `render.yaml`.
5. Select the **Free** instance tier and click **Apply** / **Deploy**.
6. Within 2–3 minutes, you will receive your free public URL (e.g. `https://shoots-ladders.onrender.com`) accessible from any phone or computer worldwide!

---

## 🛠️ Developer Commands & REST API / Comandos y API para Desarrolladores

### Rust CLI Commands:
```powershell
# Run the local development server:
cargo run

# Build optimized production release binary:
cargo build --release

# Run all 30 automated unit and integration tests:
cargo test

# Check code test coverage (requires cargo-llvm-cov):
cargo llvm-cov --summary-only

# Run strict clippy linter:
cargo clippy --all-targets -- -D warnings
```

### REST API Endpoints:
- `GET /api/health`:
  - Returns service health status, active dungeon count, and active WebSocket connections.
  ```json
  {
    "status": "ok",
    "service": "calabozos-y-dragones-rust",
    "active_rooms": 1,
    "active_connections": 2
  }
  ```
- `GET /api/board?size=50&mystery=true`:
  - Returns board metadata: `total_tiles` (50 or 100), `ladders`, `snakes`, and `mystery_tiles` with their respective tile positions and powerup types (`potion`, `scroll`, `chest`, `trap`).
  ```json
  {
    "total_tiles": 50,
    "ladders": [{"from": 3, "to": 12}, {"from": 14, "to": 28}],
    "snakes": [{"from": 18, "to": 6}, {"from": 32, "to": 16}],
    "mystery_tiles": [{"tile": 15, "kind": "potion"}, {"tile": 21, "kind": "trap"}]
  }
  ```
