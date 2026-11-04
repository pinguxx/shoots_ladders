# 🐍 Serpientes y Escaleras Online! 🪜 / Online Multiplayer Snakes and Ladders!

A high-performance, real-time multiplayer web game of the classic **Serpientes y Escaleras** (*Snakes and Ladders* / *Chutes and Ladders*), built with a **Rust** backend (Axum, Tokio, WebSockets) and a modern, responsive, bilingual vanilla JS client. Features interactive 3D dice rolling, animated step-by-step token hops, radial token clustering, fairground banter speech bubbles, mobile-first single-screen layout, and zero external audio asset dependencies.

---

## 🇺🇸 English Guide

### 🌟 Key Features

1. **Complete Traditional 100-Tile Board**:
   - 10×10 zigzag boustrophedon path numbered 1 to 100.
   - **8 Wooden Ladders**: 4➔14, 9➔31, 20➔38, 28➔84, 40➔59, 51➔67, 63➔81, 71➔91 with ascending brass fanfare arpeggios.
   - **8 Cartoon Snakes**: 17➔7, 54➔34, 62➔19, 64➔60, 87➔24, 93➔73, 95➔75, 99➔78 with descending slide whistle and rattle effects.
   - Smooth curved SVG snake bodies with animated patterns and shaded wooden rungs.

2. **Real-Time Multiplayer (Rust + Axum WebSockets)**:
   - Private game rooms with quick 6-character room codes (e.g. `SERP72`, `ESCA15`).
   - One-click shareable invite links (`?room=CODE`) with automatic session resume on reconnect.
   - Automatic host migration if the original host disconnects.
   - Up to 8 simultaneous players with custom avatars (🤠, 💀, 🪅, 🌵, 🤼, 🎸, 🌶️, 🐆, ☀️, 🦅) and player colors.

3. **Mobile-First Responsive Design (Single-Screen Experience)**:
   - Dynamic `dvh` scaling (`min(96vw, calc(100dvh - 215px), 480px)`) ensuring the header, board, dice roller, and banter buttons fit on any screen without vertical scrolling.
   - Responsive two-line mobile header (`SERPIENTES Y` / `ESCALERAS` & `SNAKES &` / `LADDERS`) preventing overlap with action buttons on small screens.
   - Slide-up bottom drawer for the Activity Chronicle Feed (`📜`) on mobile phones.

4. **Interactive 3D Dice & Tile Animation**:
   - Real 3D CSS cube with perspective depth and backface-visibility protection across all 6 faces.
   - Support for **1 Die (Classic Mode)** or **2 Dice (Fast Pace)**.
   - Step-by-step tile hopping with tactile "clack" audio feedback and radial displacement so multiple tokens sharing a tile remain visible.

5. **Fairground Banter ("El Cotorreo") & Live Shouts**:
   - Sleek horizontal touch scroll and drag-to-scroll banter bar at the bottom.
   - **Automatic Real-Time Cross-Language Translation**:
     - `Watch out for snakes! 🐍` <-> `¡Aguas con la víbora! 🐍`
     - `Climb that ladder! 🪜` <-> `¡Sube, compadre! 🪜`
     - `Almost at 100! 🏁` <-> `¡Ya merito llego! 🏁`
     - `Roll with faith! 🎲` <-> `¡Tírale con fe! 🎲`
     - `Down I go! 🧂` <-> `¡Salado me quedé! 🧂`
     - `Hurry up, your turn! ⏰` <-> `¡Ándale, te toca! ⏰`
     - `Pure luck! 🍀` <-> `¡Pura suerte! 🍀`
   - Comic-style floating speech bubbles popping over the board with the player's avatar and name.
   - Web Audio pop sounds with optional voice locution via browser SpeechSynthesis.

6. **Customizable Host Rules & In-Game Timer Cycling**:
   - 🎯 **Exact 100 Win**: Bounce back if you overshoot tile 100 (Yes / No).
   - ✨ **Extra Roll on 6**: Roll a 6 and get an extra consecutive turn.
   - ⚠️ **Three Consecutive 6s Penalty**: Sent back to tile 1 if luck is pushed too far.
   - ⏱️ **Turn Timer**: 15s, 25s, 40s, or Unlimited with auto-roll for AFK players.
   - **Host Live In-Game Cycling**: The host can cycle timer limits mid-game directly from the board header.

7. **📱 Party QR Code**:
   - In-lobby **📱 QR Code** modal so friends and family can point their smartphone camera and join directly without typing room codes.

8. **Web Audio Synthesizer (Zero External Audio Assets)**:
   - 100% synthesized procedural sound effects using Web Audio API: dice roll rumble, tile hops, ladder arpeggios, snake rattles, mariachi fanfare, and party reactions.
   - Built-in text-to-speech announcer for game events.

9. **🎨 Mexican Festive Themes & Bilingual Toggle**:
   - **🇲🇽 Fiesta Clásica**, **💀 Día de los Muertos**, and **🌿 Selva Maya**.
   - One-click instant language switcher (`🌐 Español` / `🌐 English`).

10. **Security & Production Readiness**:
    - Strict input sanitization for player names and avatars.
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

#### 2. Create a Room
1. Open [http://localhost:3000](http://localhost:3000).
2. Enter your nickname and select your avatar and color.
3. Configure your desired rules (1 or 2 dice, exact win bounce, turn timer, etc.).
4. Click **🎲 Create Room & Play**.

#### 3. Invite Players
1. Share the 6-character room code, click **📋 Copy Link**, or show the **📱 QR Code** for friends to scan with their camera.
2. Players join from their mobile or desktop browser via the **Join Room** tab.
3. Once everyone is in, the host clicks **🏁 Start Game!**.

#### 4. Play the Round!
- Take turns rolling the dice when the turn banner highlights your name.
- Watch your token hop across tiles, ascend ladders, or slide down snakes!
- Send festive reactions and fairground banter shouts to cheer or tease your friends!
- First player to reach tile 100 wins the match with confetti and fanfare!

---

## 🇲🇽 Guía en Español

### 🌟 Características Principales

1. **Tablero Tradicional Completo (100 Casillas)**:
   - Tablero 10×10 numerado de la casilla 1 a la 100 con recorrido en zigzag (estilo bustrófedon tradicional).
   - **8 Escaleras (Ladders)** con arpegios ascendentes de triunfo (4➔14, 9➔31, 20➔38, 28➔84, 40➔59, 51➔67, 63➔81, 71➔91).
   - **8 Serpientes (Snakes)** con silbido descendente y cascabel (17➔7, 54➔34, 62➔19, 64➔60, 87➔24, 93➔73, 95➔75, 99➔78).
   - Cuerpos de víbora en SVG curvado con sombras y peldaños de madera texturizados.

2. **Multijugador en Tiempo Real (Rust + Axum WebSockets)**:
   - Creación de salas privadas con códigos rápidos (ej. `SERP72`, `ESCA15`).
   - Enlace directo compartible (`?room=CODIGO`) con reconexión automática de sesión.
   - Traspaso automático de anfitrión si el creador abandona la sala.
   - Hasta 8 jugadores simultáneos con fichas tradicionales (🤠, 💀, 🪅, 🌵, 🤼, 🎸, 🌶️, 🐆, ☀️, 🦅) y colores personalizados.

3. **Optimizado para Celulares y Pantallas Pequeñas (Mobile First)**:
   - Escalado dinámico con `dvh` (`min(96vw, calc(100dvh - 215px), 480px)`) para encajar perfectamente en la pantalla sin scroll vertical.
   - Título responsivo de dos líneas en móviles (`SERPIENTES Y` / `ESCALERAS`) para evitar colisiones con los botones de acción.
   - Panel de control táctil con tiro de dado al tocar el dado 3D o el botón.
   - Crónica de actividad deslizable mediante drawer (`📜`) en celular.

4. **Dado 3D Interactivo y Física de Fichas**:
   - Dado 3D cúbico con caras reales y animación física de giro con `backface-visibility` protegida.
   - Soporte para **1 Dado (Modo Clásico)** o **2 Dados (Modo Rápido)**.
   - Animación de salto casilla por casilla con sonido táctil ("clack") y distribución radial para casillas compartidas.

5. **Gritos de Feria ("El Cotorreo") y Reacciones en Vivo**:
   - Tira horizontal inferior con soporte de deslizamiento táctil y arrastre con mouse.
   - **Traducción Automática en Tiempo Real**:
     - `¡Aguas con la víbora! 🐍` <-> `Watch out for snakes! 🐍`
     - `¡Sube, compadre! 🪜` <-> `Climb that ladder! 🪜`
     - `¡Ya merito llego! 🏁` <-> `Almost at 100! 🏁`
     - `¡Tírale con fe! 🎲` <-> `Roll with faith! 🎲`
     - `¡Salado me quedé! 🧂` <-> `Down I go! 🧂`
     - `¡Ándale, te toca! ⏰` <-> `Hurry up, your turn! ⏰`
     - `¡Pura suerte! 🍀` <-> `Pure luck! 🍀`
   - Burbujas de cómic animadas sobre el tablero con avatar y nombre del jugador.
   - Efectos de sonido Web Audio y locución por voz del navegador.

6. **Reglas Configurables por el Anfitrión**:
   - 🎯 **Tiro Exacto a la Meta**: Si te pasas del 100, rebotas hacia atrás (Sí / No).
   - ✨ **Tiro Extra al Sacar 6**: Saca un 6 y obtén otro tiro consecutivo.
   - ⚠️ **Penalización por 3 Seises Seguidos**: Regresa a la casilla 1 si la suerte es demasiado extrema.
   - ⏱️ **Temporizador de Turno**: 15s, 25s, 40s o ilimitado con auto-tiro para jugadores ausentes (el anfitrión puede cambiar el tiempo en plena partida).

7. **📱 Código QR para Dispositivos Móviles**:
   - Botón **📱 QR** en la sala para que amigos en una reunión o fiesta apunten su cámara y entren al instante sin teclear códigos.

8. **Sintetizador de Audio Web Audio API (Sin archivos externos)**:
   - 100% autónomo: sonido de dado rodando, saltos de ficha, ascenso de escalera, silbido de serpiente, fanfarria de mariachi y efectos de reacción.
   - Locutor de voz integrado (`SpeechSynthesis`) en español e inglés.

9. **🎨 Temas Visuales Mexicanos & Bilingüe**:
   - 🇲🇽 **Fiesta Clásica**, 💀 **Día de los Muertos** y 🌿 **Selva Maya**.
   - Alternancia instantánea con un clic (`🌐 Español` / `🌐 English`).

10. **Seguridad y Producción**:
    - Sanitización estricta de nombres, avatares y códigos hex contra inyecciones XSS.
    - Headers HTTP de seguridad (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`).
    - Tarea en segundo plano en Tokio para recolección de salas abandonadas (evita fugas de memoria).
    - Limitador de tasa contra spam de reacciones.

---

### 🚀 Cómo Jugar

#### 1. Iniciar el Servidor
```powershell
# Ejecutar directamente con Cargo en modo Release:
cargo run --release

# O ejecutar el binario compilado:
.\target\release\shoots_ladders.exe
```

El juego estará disponible en:
- **En tu computadora**: [http://localhost:3000](http://localhost:3000)
- **En tu red local (móviles/tablets)**: `http://TU_IP_LOCAL:3000` (impreso en la consola al iniciar)

#### 2. Crear una Sala
1. Entra a [http://localhost:3000](http://localhost:3000).
2. Escribe tu nombre o apodo, y elige tu avatar y color.
3. Configura las reglas deseadas (1 o 2 dados, tiro exacto, temporizador, etc.).
4. Haz clic en **🎲 Crear Sala & Jugar**.

#### 3. Invitar Jugadores
1. Comparte el código de sala, haz clic en **📋 Copiar Enlace**, o abre el **📱 Código QR** para que tus amigos lo escaneen con la cámara de su celular.
2. Los demás entran desde sus teléfonos en la pestaña **Unirse a Sala**.
3. El anfitrión presiona **🏁 ¡Iniciar Partida!**.

#### 4. ¡A Jugar!
- Tira el dado en tu turno tocando el dado 3D o el botón de tirar.
- Observa cómo tu ficha salta por las casillas, sube por las escaleras o resbala por las serpientes.
- Manda reacciones y gritos de feria para cotorrear con tus amigos.
- ¡El primer jugador en llegar a la casilla 100 se corona Campeón con confeti y fanfarria!

---

## ☁️ Free Cloud Deployment to Render.com / Despliegue en Render.com

This repository includes a multi-stage [`Dockerfile`](file:///c:/dev/shoots_ladders/Dockerfile) and a [`render.yaml`](file:///c:/dev/shoots_ladders/render.yaml) blueprint ready for 100% free hosting:

### English:
1. Push your repository to GitHub: `git push origin main`.
2. Go to [dashboard.render.com](https://dashboard.render.com) and click **New +** → **Blueprint** (or **Web Service**).
3. Connect your repository (`shoots_ladders`).
4. Render automatically detects the `Dockerfile` and `render.yaml`.
5. Select the **Free** instance tier and click **Apply** / **Deploy**.
6. Within 2–3 minutes, you will receive your free public URL (e.g. `https://serpientes-y-escaleras.onrender.com`) accessible from any phone or computer worldwide!

### Español:
1. Sube tus cambios a GitHub: `git push origin main`.
2. Ve a [dashboard.render.com](https://dashboard.render.com) y haz clic en **New +** → **Blueprint** (o **Web Service**).
3. Conecta tu repositorio (`shoots_ladders`).
4. Render detectará automáticamente el `Dockerfile` y `render.yaml`.
5. Selecciona el plan **Free** y presiona **Apply** / **Deploy**.
6. ¡En 2 a 3 minutos tendrás tu enlace público gratuito (ej. `https://serpientes-y-escaleras.onrender.com`) listo para jugar con amigos y familia en cualquier lugar del mundo!
