# 🐍 Serpientes y Escaleras Multijugador Online! 🪜

Un juego web multijugador en tiempo real del clásico tradicional **Serpientes y Escaleras** (conocido en inglés como *Snakes and Ladders* / *Chutes and Ladders*), construido con un backend de alto rendimiento en **Rust** (Axum, Tokio, WebSockets) y un cliente moderno, responsivo y bilingüe.

---

## 🌟 Características Principales / Key Features

1. **Tablero Tradicional Completo (100 Casillas)**:
   - Tablero 10×10 numerado de la casilla 1 a la 100 con recorrido en zigzag (estilo bustrófedon tradicional).
   - **Escaleras (Ladders)** con arpegios de triunfo ascendentes (ej. 4➔14, 9➔31, 20➔38, 28➔84, 40➔59, 51➔67, 63➔81, 71➔91).
   - **Serpientes (Snakes)** con silbido descendente y cascabel (ej. 17➔7, 54➔34, 62➔19, 64➔60, 87➔24, 93➔73, 95➔75, 99➔78).

2. **Multijugador en Tiempo Real (Rust + Axum WebSockets)**:
   - Creación de salas privadas con códigos rápidos (ej. `SERP72`, `ESCA15`).
   - Enlace directo compartible (`?room=CODIGO`).
   - Selector de hasta 8 jugadores con fichas y colores personalizados (🤠, 💀, 🪅, 🌵, 🤼, 🎸, 🌶️, 🐆, ☀️, 🦅).

3. **Optimizado para Celulares y Pantallas Pequeñas (Mobile First)**:
   - Tablero escalado dinámicamente con `dvh` (`min(96vw, calc(100dvh - 215px), 480px)`) para encajar perfectamente en la pantalla sin scroll vertical.
   - Panel de control táctil con tiro de dado al tocar el dado o el botón.
   - Crónica de actividad deslizable mediante drawer (`📜`) en celular.

4. **Animación Paso a Paso & Física de Fichas**:
   - Cada tiro de dado anima a la ficha saltando casilla por casilla con sonido táctil ("clack").
   - Animación de deslizamiento por la serpiente o ascenso por la escalera.
   - Si varios jugadores caen en la misma casilla, las fichas se acomodan radialmente para que todos los avatares sean visibles.

5. **Dado 3D Interactivo**:
   - Dado 3D con caras reales y animación física de giro.
   - Soporte para **1 Dado (Modo Clásico)** o **2 Dados (Modo Rápido)**.

6. **Reglas Configurables por el Anfitrión**:
   - 🎯 **Tiro Exacto a la Meta**: Si te pasas del 100, rebotas hacia atrás (configurable: Sí / No).
   - ✨ **Tiro Extra al Sacar 6**: Saca un 6 y obtén otro tiro consecutivo.
   - ⚠️ **Penalización por 3 Seises Seguidos**: Regresa a la casilla 1 si la suerte es demasiado extrema.
   - ⏱️ **Temporizador de Turno**: Opciones de 15s, 25s, 40s o ilimitado con auto-tiro para jugadores AFK.

7. **📱 Código QR para Dispositivos Móviles**:
   - Botón **📱 QR** en la sala para que amigos en una reunión o fiesta apunten su cámara y entren al instante sin teclear códigos.

8. **Sintetizador de Audio Web Audio API (Sin archivos externos)**:
   - 100% autónomo: sonido de dado rodando, saltos de ficha, ascenso de escalera, silbido de serpiente, fanfarria de mariachi y efectos de reacción.
   - Locutor de voz integrado (`SpeechSynthesis`) en español e inglés.

9. **Seguridad y Producción**:
   - Sanitización estricta de nombres, avatares y códigos hex en backend y frontend contra inyecciones XSS.
   - Headers HTTP de seguridad (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`).
   - Tarea en segundo plano en Tokio para recolección de salas abandonadas (evita fugas de memoria).
   - Limitador de tasa contra spam de reacciones.

10. **🎨 Temas Visuales Mexicanos & Bilingüe**:
    - 🇲🇽 **Fiesta Clásica**, 💀 **Día de los Muertos** y 🌿 **Selva Maya**.
    - Alternancia instantánea con un clic (`🌐 Español` / `🌐 English`).

---

## 🚀 Despliegue en Render.com (Production)

El repositorio incluye un `Dockerfile` multi-etapa y un `render.yaml` listos para Render:

1. Sube este proyecto a tu repositorio de GitHub o GitLab.
2. En Render, crea un nuevo **Web Service** seleccionando tu repositorio (o usando Blueprint con `render.yaml`).
3. Render detectará automáticamente el `Dockerfile`, compilará el binario Rust en release y desplegará la aplicación en segundos.

---

## 💻 Ejecución Local

```powershell
# Ejecutar directamente el binario compilado:
.\target\release\shoots_ladders.exe

# O con Cargo:
cargo run --release
```

- **Local**: [http://localhost:3000](http://localhost:3000)
- **Red Local (Celular)**: `http://TU_IP_LOCAL:3000`
