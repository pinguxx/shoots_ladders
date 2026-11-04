// Serpientes y Escaleras Online - Real-Time Multiplayer Client
// Built with Axum + Tokio Rust WebSockets & Web Audio API

(function () {
  'use strict';

  // -------------------------------------------------------------
  // 1. Translations & State
  // -------------------------------------------------------------
  const i18n = {
    es: {
      appTitle: "SERPIENTES & ESCALERAS",
      appSubtitle: "Multijugador en Vivo 🇲🇽",
      tabCreate: "Crear Sala",
      tabJoin: "Unirse a Sala",
      labelName: "Tu Nombre o Apodo",
      placeholderName: "Ej. Pancho Villa",
      labelAvatar: "Elige tu Ficha / Avatar",
      labelColor: "Elige tu Color",
      labelDiceCount: "Número de Dados",
      dice1: "1 Dado (Clásico)",
      dice2: "2 Dados (Rápido)",
      labelExactWin: "¿Rebotar si pasa de 100?",
      exactWinYes: "Sí (Tiro exacto para ganar)",
      exactWinNo: "No (100 o más gana)",
      labelBonusSix: "Tiro extra al sacar 6",
      labelThreeSixes: "Penalización por tres 6 seguidos (regresa al 1)",
      labelTimer: "Tiempo por turno",
      timerNone: "Sin límite",
      timer15: "15 segundos",
      timer25: "25 segundos (Recomendado)",
      timer40: "40 segundos",
      btnCreate: "🎲 Crear Sala & Jugar",
      btnJoin: "🚀 Entrar a la Sala",
      labelRoomCode: "Código de la Sala",
      placeholderRoomCode: "Ej. SERP12",
      lobbyTitle: "Sala de Espera",
      waitingPlayers: "Esperando jugadores...",
      btnStartGame: "🏁 ¡Iniciar Partida!",
      btnCopyLink: "📋 Copiar Enlace",
      btnQr: "📱 Código QR",
      btnRollDice: "🎲 ¡TIRAR DADO!",
      waitingRoll: "Esperando a",
      turnTitle: "Turno actual",
      feedTitle: "Crónica de la Partida",
      qrTitle: "Invita a tus Amigos",
      qrSub: "Escanea con la cámara de tu celular para entrar directamente a la partida:",
      modalClose: "Cerrar",
      winnerTitle: "¡VICTORIA!",
      winnerSub: "¡Ha llegado a la casilla 100 y se corona Campeón!",
      btnPlayAgain: "🔄 Jugar Otra Vez",
      statRolls: "Tiros",
      statLadders: "Escaleras",
      statSnakes: "Serpientes",
      toastCopied: "¡Enlace copiado al portapapeles!",
      errorFillName: "Por favor escribe tu nombre.",
      errorFillCode: "Por favor ingresa un código de sala válido.",
      leaveRoom: "Salir de la Sala",
      confirmLeave: "¿Seguro que deseas salir de la sala?",
      resumeTitle: "Partida en Curso",
      btnResume: "▶️ Reanudar",
      reconnectingText: "Reconectando a la partida...",
      toastReconnected: "¡Reconectado a la partida!",
      toastGameEnded: "La partida anterior ha finalizado."
    },
    en: {
      appTitle: "SNAKES & LADDERS",
      appSubtitle: "Real-Time Multiplayer 🇲🇽",
      tabCreate: "Create Room",
      tabJoin: "Join Room",
      labelName: "Your Name or Nickname",
      placeholderName: "e.g. Lucky Charlie",
      labelAvatar: "Choose Your Token / Avatar",
      labelColor: "Choose Your Color",
      labelDiceCount: "Number of Dice",
      dice1: "1 Die (Classic)",
      dice2: "2 Dice (Fast Pace)",
      labelExactWin: "Bounce back if overshoot 100?",
      exactWinYes: "Yes (Exact roll to win)",
      exactWinNo: "No (100 or over wins)",
      labelBonusSix: "Extra roll on rolling 6",
      labelThreeSixes: "Penalty on 3 consecutive 6s (back to 1)",
      labelTimer: "Turn timer",
      timerNone: "No limit",
      timer15: "15 seconds",
      timer25: "25 seconds (Recommended)",
      timer40: "40 seconds",
      btnCreate: "🎲 Create Room & Play",
      btnJoin: "🚀 Join Room",
      labelRoomCode: "Room Code",
      placeholderRoomCode: "e.g. SERP12",
      lobbyTitle: "Waiting Lobby",
      waitingPlayers: "Waiting for players...",
      btnStartGame: "🏁 Start Game!",
      btnCopyLink: "📋 Copy Link",
      btnQr: "📱 QR Code",
      btnRollDice: "🎲 ROLL DICE!",
      waitingRoll: "Waiting for",
      turnTitle: "Current Turn",
      feedTitle: "Live Activity Log",
      qrTitle: "Invite Your Friends",
      qrSub: "Scan with your phone camera to jump straight into this match:",
      modalClose: "Close",
      winnerTitle: "VICTORY!",
      winnerSub: "Has reached tile 100 and is the Grand Champion!",
      btnPlayAgain: "🔄 Play Again",
      statRolls: "Rolls",
      statLadders: "Ladders",
      statSnakes: "Snakes",
      toastCopied: "Invite link copied to clipboard!",
      errorFillName: "Please enter your name.",
      errorFillCode: "Please enter a valid room code.",
      leaveRoom: "Leave Room",
      confirmLeave: "Are you sure you want to leave the room?",
      resumeTitle: "Game in Progress",
      btnResume: "▶️ Resume",
      reconnectingText: "Reconnecting to game...",
      toastReconnected: "Reconnected to game!",
      toastGameEnded: "The previous game has ended."
    }
  };

  const AVATARS = ["🤠", "💀", "🪅", "🌵", "🤼", "🎸", "🌶️", "🐆", "☀️", "🦅"];
  const COLORS = [
    "#ef4444", "#f59e0b", "#10b981", "#3b82f6",
    "#8b5cf6", "#ec4899", "#06b6d4", "#f97316"
  ];
  const THEMES = ["fiesta", "muertos", "selva"];

  let currentLang = localStorage.getItem("serpientes_lang") || "es";
  let currentThemeIdx = 0;
  let selectedAvatar = AVATARS[0];
  let selectedColor = COLORS[0];

  let ws = null;
  let roomCode = null;
  let playerId = localStorage.getItem("serpientes_player_id") || null;
  let isHost = false;
  let currentRoom = null;
  let boardMeta = null;
  let isAnimatingMove = false;
  let moveQueue = [];

  // -------------------------------------------------------------
  // 2. DOM Elements
  // -------------------------------------------------------------
  const el = {
    btnToggleLang: document.getElementById("btn-toggle-lang"),
    btnToggleTheme: document.getElementById("btn-toggle-theme"),
    btnToggleSound: document.getElementById("btn-toggle-sound"),
    btnToggleSpeech: document.getElementById("btn-toggle-speech"),
    btnHeaderLeave: document.getElementById("btn-header-leave"),

    screenWelcome: document.getElementById("screen-welcome"),
    screenLobby: document.getElementById("screen-lobby"),
    screenGame: document.getElementById("screen-game"),

    resumeBanner: document.getElementById("resume-banner"),
    resumeRoomCode: document.getElementById("resume-room-code"),
    resumePlayerName: document.getElementById("resume-player-name"),
    btnResumeGame: document.getElementById("btn-resume-game"),
    btnDismissResume: document.getElementById("btn-dismiss-resume"),
    reconnectingOverlay: document.getElementById("reconnecting-overlay"),

    tabCreate: document.getElementById("tab-create"),
    tabJoin: document.getElementById("tab-join"),
    formCreate: document.getElementById("form-create"),
    formJoin: document.getElementById("form-join"),

    createName: document.getElementById("create-name"),
    joinName: document.getElementById("join-name"),
    joinCode: document.getElementById("join-code"),

    avatarPicker: document.getElementById("avatar-picker"),
    colorPicker: document.getElementById("color-picker"),

    settingDiceCount: document.getElementById("setting-dice-count"),
    settingExactWin: document.getElementById("setting-exact-win"),
    settingBonusSix: document.getElementById("setting-bonus-six"),
    settingThreeSixes: document.getElementById("setting-three-sixes"),
    settingTurnTimer: document.getElementById("setting-turn-timer"),

    lobbyRoomCode: document.getElementById("lobby-room-code"),
    lobbyPlayersList: document.getElementById("lobby-players-list"),
    btnCopyLink: document.getElementById("btn-copy-link"),
    btnShowQr: document.getElementById("btn-show-qr"),
    btnStartGame: document.getElementById("btn-start-game"),

    boardGrid: document.getElementById("board-grid"),
    boardSvgOverlay: document.getElementById("board-svg-overlay"),
    boardTokensLayer: document.getElementById("board-tokens-layer"),

    turnAvatar: document.getElementById("turn-avatar"),
    turnPlayerName: document.getElementById("turn-player-name"),
    turnTimer: document.getElementById("turn-timer"),
    btnToggleFeedMobile: document.getElementById("btn-toggle-feed-mobile"),

    diceContainerBox: document.getElementById("dice-container-box"),
    cubeDie1: document.getElementById("cube-die-1"),
    cubeDie2: document.getElementById("cube-die-2"),
    btnRollDice: document.getElementById("btn-roll-dice"),
    activityFeed: document.getElementById("activity-feed"),
    drawerActivityLog: document.getElementById("drawer-activity-log"),
    btnCloseFeedDrawer: document.getElementById("btn-close-feed-drawer"),
    reactionsBar: document.getElementById("reactions-bar"),
    floatingEmojisLayer: document.getElementById("floating-emojis-layer"),

    modalQr: document.getElementById("modal-qr"),
    qrContainer: document.getElementById("qr-container"),
    qrShareLink: document.getElementById("qr-share-link"),
    btnCopyModalLink: document.getElementById("btn-copy-modal-link"),
    modalWinner: document.getElementById("modal-winner"),
    winnerAvatar: document.getElementById("winner-avatar"),
    winnerName: document.getElementById("winner-name"),
    winnerStatRolls: document.getElementById("winner-stat-rolls"),
    winnerStatLadders: document.getElementById("winner-stat-ladders"),
    winnerStatSnakes: document.getElementById("winner-stat-snakes"),
    btnPlayAgain: document.getElementById("btn-play-again"),
    confettiCanvas: document.getElementById("confetti-canvas")
  };

  // -------------------------------------------------------------
  // 3. UI Helpers & Localization
  // -------------------------------------------------------------
  function updateLanguage() {
    const texts = i18n[currentLang];
    document.querySelectorAll("[data-i18n]").forEach(node => {
      const key = node.getAttribute("data-i18n");
      if (texts[key]) node.textContent = texts[key];
    });
    document.querySelectorAll("[data-i18n-ph]").forEach(node => {
      const key = node.getAttribute("data-i18n-ph");
      if (texts[key]) node.setAttribute("placeholder", texts[key]);
    });

    el.btnToggleLang.innerHTML = currentLang === "es"
      ? '<span class="lang-icon">🌐</span> <span class="lang-text-long">Español</span><span class="lang-text-short">ES</span>'
      : '<span class="lang-icon">🌐</span> <span class="lang-text-long">English</span><span class="lang-text-short">EN</span>';
    if (window.serpientesAudio) {
      window.serpientesAudio.currentLang = currentLang;
    }
  }

  function toggleLanguage() {
    currentLang = currentLang === "es" ? "en" : "es";
    localStorage.setItem("serpientes_lang", currentLang);
    updateLanguage();
    if (currentRoom) renderActivityFeed(currentRoom.logs);
  }

  function updateAmbientDecorations(themeName) {
    const layer = document.getElementById("ambient-theme-layer");
    if (!layer) return;
    layer.innerHTML = "";

    let icons = ["🎉", "✨", "🪅", "🎊", "🇲🇽", "🌮", "🌶️"];
    let count = 12;

    if (themeName === "muertos") {
      icons = ["💀", "🌼", "🏵️", "🕯️", "💀", "🌼", "🏵️"];
      count = 14;
    } else if (themeName === "selva") {
      icons = ["🌿", "🦜", "🍃", "🐆", "☀️", "🌺"];
      count = 12;
    }

    for (let i = 0; i < count; i++) {
      const p = document.createElement("div");
      p.className = "ambient-particle";
      const icon = icons[i % icons.length];
      p.textContent = icon;

      const left = (i * (94 / count) + Math.random() * (90 / count)).toFixed(1);
      const duration = (10 + Math.random() * 8).toFixed(1);
      const delay = (Math.random() * 7).toFixed(1);
      const size = (0.95 + Math.random() * 0.75).toFixed(2);
      const driftX = (Math.random() * 90 - 45).toFixed(0) + "px";
      const rot = (Math.random() * 140 - 70).toFixed(0) + "deg";

      p.style.left = `${left}%`;
      p.style.fontSize = `${size}rem`;
      p.style.animationDuration = `${duration}s`;
      p.style.animationDelay = `${delay}s`;
      p.style.setProperty("--drift-x", driftX);
      p.style.setProperty("--rot", rot);

      layer.appendChild(p);
    }
  }

  function toggleTheme() {
    currentThemeIdx = (currentThemeIdx + 1) % THEMES.length;
    const theme = THEMES[currentThemeIdx];
    document.documentElement.setAttribute("data-theme", theme);
    updateAmbientDecorations(theme);
  }

  function showScreen(screenId) {
    document.querySelectorAll(".screen").forEach(s => s.classList.remove("active"));
    const target = document.getElementById(screenId);
    if (target) target.classList.add("active");

    if (screenId === "screen-welcome") {
      el.btnHeaderLeave.style.display = "none";
    } else {
      el.btnHeaderLeave.style.display = "flex";
    }
  }

  function showToast(msg) {
    const toast = document.createElement("div");
    toast.style.cssText = `
      position: fixed;
      bottom: 24px;
      left: 50%;
      transform: translateX(-50%);
      background: rgba(28, 18, 14, 0.95);
      border: 1.5px solid #f59e0b;
      color: #fff;
      padding: 9px 18px;
      border-radius: 20px;
      font-size: 0.88rem;
      font-weight: 800;
      z-index: 999;
      box-shadow: 0 4px 15px rgba(0,0,0,0.5);
      animation: fadeIn 0.2s ease;
      pointer-events: none;
    `;
    toast.textContent = msg;
    document.body.appendChild(toast);
    setTimeout(() => {
      toast.style.opacity = "0";
      toast.style.transition = "opacity 0.3s";
      setTimeout(() => toast.remove(), 300);
    }, 2200);
  }

  // Session Persistence & Reconnection Management
  const SESSION_KEY = "serpientes_session";

  function saveSession(rCode, pId, pName, avatar, color, hostFlag) {
    const session = {
      roomCode: rCode,
      playerId: pId,
      playerName: pName,
      avatar: avatar,
      color: color,
      isHost: hostFlag
    };
    try {
      localStorage.setItem(SESSION_KEY, JSON.stringify(session));
      localStorage.setItem("serpientes_player_id", pId);
    } catch (e) {
      console.warn("Could not save session to localStorage", e);
    }
    const url = new URL(window.location.href);
    url.searchParams.set("room", rCode);
    window.history.replaceState({}, "", url.toString());
  }

  function clearSession() {
    try {
      localStorage.removeItem(SESSION_KEY);
    } catch (e) {}
    const url = new URL(window.location.href);
    url.searchParams.delete("room");
    window.history.replaceState({}, "", url.toString());
    if (el.resumeBanner) el.resumeBanner.style.display = "none";
    if (el.reconnectingOverlay) el.reconnectingOverlay.style.display = "none";
  }

  function getStoredSession() {
    try {
      const raw = localStorage.getItem(SESSION_KEY);
      if (!raw) return null;
      return JSON.parse(raw);
    } catch (e) {
      return null;
    }
  }

  function checkResumeBanner() {
    const session = getStoredSession();
    if (session && session.roomCode && session.playerName) {
      if (el.resumeBanner) {
        el.resumeRoomCode.textContent = session.roomCode;
        el.resumePlayerName.textContent = session.playerName;
        el.resumeBanner.style.display = "flex";
      }
    } else {
      if (el.resumeBanner) el.resumeBanner.style.display = "none";
    }
  }

  // -------------------------------------------------------------
  // 4. Board Geometry & SVG Renderer (High-Aesthetic Mexican Style)
  // -------------------------------------------------------------
  function getTileCoords(tile) {
    if (tile <= 0) {
      return { x: 50, y: 1040 };
    }
    const clamped = Math.min(100, Math.max(1, tile));
    const row = Math.floor((clamped - 1) / 10);
    const col = (row % 2 === 0) ? ((clamped - 1) % 10) : (9 - ((clamped - 1) % 10));

    // In a 1000x1000 SVG viewBox
    const x = (col + 0.5) * 100;
    const y = (9 - row + 0.5) * 100;
    return { x, y, row, col };
  }

  function buildBoardGrid(board) {
    el.boardGrid.innerHTML = "";

    // 10x10 grid: row 9 (top) down to row 0 (bottom)
    for (let row = 9; row >= 0; row--) {
      for (let col = 0; col < 10; col++) {
        const tileNum = (row % 2 === 0) ? (row * 10 + col + 1) : (row * 10 + (9 - col) + 1);

        const tileDiv = document.createElement("div");
        tileDiv.className = "tile";
        tileDiv.id = `tile-${tileNum}`;

        // True checkerboard pattern:
        if ((row + col) % 2 === 0) {
          tileDiv.classList.add("tile-alt");
        }

        if (tileNum === 100) tileDiv.classList.add("tile-goal");
        if (tileNum === 1) tileDiv.classList.add("tile-start");

        const numSpan = document.createElement("span");
        numSpan.className = "tile-number";
        if (tileNum === 100) {
          numSpan.textContent = "🏆 100";
        } else if (tileNum === 1) {
          numSpan.textContent = "🏁 1";
        } else {
          numSpan.textContent = String(tileNum);
        }
        tileDiv.appendChild(numSpan);

        // Subtle indicator if tile is a ladder start or snake mouth
        if (board && board.ladders) {
          const lad = board.ladders.find(l => l.from === tileNum);
          if (lad) {
            const hint = document.createElement("span");
            hint.className = "tile-badge-hint tile-badge-ladder";
            hint.textContent = `▲${lad.to}`;
            hint.title = `Escalera a casilla ${lad.to}`;
            tileDiv.appendChild(hint);
          }
        }
        if (board && board.snakes) {
          const snk = board.snakes.find(s => s.from === tileNum);
          if (snk) {
            const hint = document.createElement("span");
            hint.className = "tile-badge-hint tile-badge-snake";
            hint.textContent = `▼${snk.to}`;
            hint.title = `Serpiente a casilla ${snk.to}`;
            tileDiv.appendChild(hint);
          }
        }

        el.boardGrid.appendChild(tileDiv);
      }
    }
  }

  function drawBoardSvg(board) {
    el.boardSvgOverlay.innerHTML = "";
    const svgNS = "http://www.w3.org/2000/svg";

    const defs = document.createElementNS(svgNS, "defs");

    // Ladder Wood 3D Gradient
    const ladderGrad = document.createElementNS(svgNS, "linearGradient");
    ladderGrad.id = "ladderWood3D";
    ladderGrad.setAttribute("x1", "0%");
    ladderGrad.setAttribute("y1", "0%");
    ladderGrad.setAttribute("x2", "100%");
    ladderGrad.setAttribute("y2", "0%");
    ladderGrad.innerHTML = `
      <stop offset="0%" stop-color="#78350f" />
      <stop offset="25%" stop-color="#d97706" />
      <stop offset="50%" stop-color="#fef3c7" />
      <stop offset="80%" stop-color="#b45309" />
      <stop offset="100%" stop-color="#451a03" />
    `;
    defs.appendChild(ladderGrad);

    // Realistic Drop Shadow Filter
    const filter = document.createElementNS(svgNS, "filter");
    filter.id = "ladderDropShadow";
    filter.setAttribute("x", "-20%");
    filter.setAttribute("y", "-20%");
    filter.setAttribute("width", "150%");
    filter.setAttribute("height", "150%");
    filter.innerHTML = `<feDropShadow dx="3" dy="6" stdDeviation="4.5" flood-color="#000" flood-opacity="0.45"/>`;
    defs.appendChild(filter);

    el.boardSvgOverlay.appendChild(defs);

    // Helper to draw a rich 3D wooden cylinder (works at any angle without SVG bounding-box collapse)
    function createWoodSegment(x1, y1, x2, y2, outerWidth, midWidth, highlightWidth) {
      const segGroup = document.createElementNS(svgNS, "g");

      // 1. Dark Bark / Hardwood Outermost Layer
      const outer = document.createElementNS(svgNS, "line");
      outer.setAttribute("x1", String(x1));
      outer.setAttribute("y1", String(y1));
      outer.setAttribute("x2", String(x2));
      outer.setAttribute("y2", String(y2));
      outer.setAttribute("stroke", "#451a03");
      outer.setAttribute("stroke-width", String(outerWidth));
      outer.setAttribute("stroke-linecap", "round");
      segGroup.appendChild(outer);

      // 2. Warm Amber Cedar Core
      const mid = document.createElementNS(svgNS, "line");
      mid.setAttribute("x1", String(x1));
      mid.setAttribute("y1", String(y1));
      mid.setAttribute("x2", String(x2));
      mid.setAttribute("y2", String(y2));
      mid.setAttribute("stroke", "#b45309");
      mid.setAttribute("stroke-width", String(midWidth));
      mid.setAttribute("stroke-linecap", "round");
      segGroup.appendChild(mid);

      // 3. Specular Golden Grain Ridge
      if (highlightWidth > 0) {
        const hi = document.createElementNS(svgNS, "line");
        hi.setAttribute("x1", String(x1 - 1));
        hi.setAttribute("y1", String(y1 - 1));
        hi.setAttribute("x2", String(x2 - 1));
        hi.setAttribute("y2", String(y2 - 1));
        hi.setAttribute("stroke", "#fef3c7");
        hi.setAttribute("stroke-width", String(highlightWidth));
        hi.setAttribute("stroke-linecap", "round");
        hi.setAttribute("opacity", "0.75");
        segGroup.appendChild(hi);
      }

      return segGroup;
    }

    // 1. Draw 3D Authentic Wooden Ladders
    board.ladders.forEach(l => {
      let p1 = getTileCoords(l.from); // Base
      let p2 = getTileCoords(l.to);   // Top

      // Add a natural, subtle physical lean for vertical ladders (e.g. 71 -> 91 along the board border)
      // This prevents visual clipping against tile numbers or border frames
      if (Math.abs(p2.x - p1.x) < 2) {
        p1 = { ...p1, x: p1.x - 7, y: p1.y + 6 };
        p2 = { ...p2, x: p2.x + 7, y: p2.y - 6 };
      }

      const dx = p2.x - p1.x;
      const dy = p2.y - p1.y;
      const len = Math.sqrt(dx * dx + dy * dy);
      const ux = dx / len;
      const uy = dy / len;
      // Perpendicular rail offset (rail spacing: 26px total)
      const nx = -uy * 13;
      const ny = ux * 13;

      const group = document.createElementNS(svgNS, "g");
      group.setAttribute("filter", "url(#ladderDropShadow)");

      // Left & Right Wooden Rails (3D Layered Cylinders)
      const rail1 = createWoodSegment(p1.x + nx, p1.y + ny, p2.x + nx, p2.y + ny, 8.5, 5.5, 1.8);
      group.appendChild(rail1);

      const rail2 = createWoodSegment(p1.x - nx, p1.y - ny, p2.x - nx, p2.y - ny, 8.5, 5.5, 1.8);
      group.appendChild(rail2);

      // Wooden Rungs with realistic joints and drop-shadows
      const rungsCount = Math.floor(len / 32);
      for (let i = 1; i <= rungsCount; i++) {
        const frac = i / (rungsCount + 1);
        const rx = p1.x + dx * frac;
        const ry = p1.y + dy * frac;

        // Shadow under rung
        const rungShadow = document.createElementNS(svgNS, "line");
        rungShadow.setAttribute("x1", String(rx + nx));
        rungShadow.setAttribute("y1", String(ry + ny + 2.5));
        rungShadow.setAttribute("x2", String(rx - nx));
        rungShadow.setAttribute("y2", String(ry - ny + 2.5));
        rungShadow.setAttribute("stroke", "rgba(0,0,0,0.4)");
        rungShadow.setAttribute("stroke-width", "5.5");
        rungShadow.setAttribute("stroke-linecap", "round");
        group.appendChild(rungShadow);

        // 3D Wooden Rung
        const rung = createWoodSegment(rx + nx, ry + ny, rx - nx, ry - ny, 6, 4, 1.2);
        group.appendChild(rung);

        // Wooden Pegs / Dowel Pins at rail-rung intersections
        const peg1 = document.createElementNS(svgNS, "circle");
        peg1.setAttribute("cx", String(rx + nx));
        peg1.setAttribute("cy", String(ry + ny));
        peg1.setAttribute("r", "2.6");
        peg1.setAttribute("fill", "#290e02");
        peg1.setAttribute("stroke", "#d97706");
        peg1.setAttribute("stroke-width", "0.7");
        group.appendChild(peg1);

        const peg2 = document.createElementNS(svgNS, "circle");
        peg2.setAttribute("cx", String(rx - nx));
        peg2.setAttribute("cy", String(ry - ny));
        peg2.setAttribute("r", "2.6");
        peg2.setAttribute("fill", "#290e02");
        peg2.setAttribute("stroke", "#d97706");
        peg2.setAttribute("stroke-width", "0.7");
        group.appendChild(peg2);
      }

      el.boardSvgOverlay.appendChild(group);
    });

    // 2. Draw Authentic Mexican Serpientes (Organic, Shaded, Expressive)
    board.snakes.forEach((s, idx) => {
      const head = getTileCoords(s.from);
      const tail = getTileCoords(s.to);

      const dx = tail.x - head.x;
      const dy = tail.y - head.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const nx = -dy / dist;
      const ny = dx / dist;

      // S-curve wave displacement
      const sign = (idx % 2 === 0) ? 1 : -1;
      const wave = Math.min(80, dist * 0.28) * sign;

      const c1x = head.x + dx * 0.32 + nx * wave;
      const c1y = head.y + dy * 0.32 + ny * wave;
      const c2x = head.x + dx * 0.68 - nx * wave;
      const c2y = head.y + dy * 0.68 - ny * wave;

      const pathData = `M ${head.x} ${head.y} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${tail.x} ${tail.y}`;

      const group = document.createElementNS(svgNS, "g");
      group.setAttribute("filter", "url(#ladderDropShadow)");

      // Color Palettes for Serpents
      const snakePalettes = [
        { main: "#047857", belly: "#34d399", accent: "#fef08a", dark: "#064e3b" }, // Emerald Quetzalcoatl
        { main: "#be123c", belly: "#fb7185", accent: "#fef08a", dark: "#881337" }, // Coral Viper
        { main: "#6d28d9", belly: "#c084fc", accent: "#fde047", dark: "#4c1d95" }, // Royal Aztec
        { main: "#b45309", belly: "#fde047", accent: "#fffbeb", dark: "#78350f" }  // Golden Cascabel
      ];
      const p = snakePalettes[idx % snakePalettes.length];

      // Snake Linear Gradient
      const gradId = `snakeGradient-${idx}`;
      const grad = document.createElementNS(svgNS, "linearGradient");
      grad.id = gradId;
      grad.innerHTML = `
        <stop offset="0%" stop-color="${p.main}" />
        <stop offset="60%" stop-color="${p.belly}" />
        <stop offset="100%" stop-color="${p.dark}" />
      `;
      defs.appendChild(grad);

      // Main Muscular Body
      const body = document.createElementNS(svgNS, "path");
      body.setAttribute("d", pathData);
      body.setAttribute("fill", "none");
      body.setAttribute("stroke", `url(#${gradId})`);
      body.setAttribute("stroke-width", "26");
      body.setAttribute("stroke-linecap", "round");
      body.setAttribute("stroke-linejoin", "round");
      group.appendChild(body);

      // Snake Belly Shading / Diamond pattern
      const belly = document.createElementNS(svgNS, "path");
      belly.setAttribute("d", pathData);
      belly.setAttribute("fill", "none");
      belly.setAttribute("stroke", p.accent);
      belly.setAttribute("stroke-width", "5");
      belly.setAttribute("stroke-linecap", "round");
      belly.setAttribute("opacity", "0.8");
      group.appendChild(belly);

      // Snake Head (Almond / Teardrop viper head facing outward from body)
      const headGroup = document.createElementNS(svgNS, "g");
      const angle = Math.atan2(c1y - head.y, c1x - head.x) * (180 / Math.PI);
      headGroup.setAttribute("transform", `translate(${head.x}, ${head.y}) rotate(${angle + 90})`);

      // Red Forked Tongue
      const tongue = document.createElementNS(svgNS, "path");
      tongue.setAttribute("d", "M 0 16 L 0 32 L -5 38 M 0 32 L 5 38");
      tongue.setAttribute("fill", "none");
      tongue.setAttribute("stroke", "#dc2626");
      tongue.setAttribute("stroke-width", "3");
      tongue.setAttribute("stroke-linecap", "round");
      headGroup.appendChild(tongue);

      // White Sharp Fangs
      const fangL = document.createElementNS(svgNS, "polygon");
      fangL.setAttribute("points", "-4,16 -2,22 -6,16");
      fangL.setAttribute("fill", "#ffffff");
      headGroup.appendChild(fangL);

      const fangR = document.createElementNS(svgNS, "polygon");
      fangR.setAttribute("points", "4,16 6,16 2,22");
      fangR.setAttribute("fill", "#ffffff");
      headGroup.appendChild(fangR);

      // Viper Head Base
      const headShape = document.createElementNS(svgNS, "path");
      headShape.setAttribute("d", "M 0 18 C 14 18, 18 -6, 0 -16 C -18 -6, -14 18, 0 18 Z");
      headShape.setAttribute("fill", p.main);
      headShape.setAttribute("stroke", p.dark);
      headShape.setAttribute("stroke-width", "2");
      headGroup.appendChild(headShape);

      // Golden Viper Eyes with slit pupils
      const eyeL = document.createElementNS(svgNS, "circle");
      eyeL.setAttribute("cx", "-8");
      eyeL.setAttribute("cy", "4");
      eyeL.setAttribute("r", "4.5");
      eyeL.setAttribute("fill", "#fef08a");
      headGroup.appendChild(eyeL);

      const pupilL = document.createElementNS(svgNS, "ellipse");
      pupilL.setAttribute("cx", "-8");
      pupilL.setAttribute("cy", "4");
      pupilL.setAttribute("rx", "1.5");
      pupilL.setAttribute("ry", "3.5");
      pupilL.setAttribute("fill", "#111");
      headGroup.appendChild(pupilL);

      const eyeR = document.createElementNS(svgNS, "circle");
      eyeR.setAttribute("cx", "8");
      eyeR.setAttribute("cy", "4");
      eyeR.setAttribute("r", "4.5");
      eyeR.setAttribute("fill", "#fef08a");
      headGroup.appendChild(eyeR);

      const pupilR = document.createElementNS(svgNS, "ellipse");
      pupilR.setAttribute("cx", "8");
      pupilR.setAttribute("cy", "4");
      pupilR.setAttribute("rx", "1.5");
      pupilR.setAttribute("ry", "3.5");
      pupilR.setAttribute("fill", "#111");
      headGroup.appendChild(pupilR);

      group.appendChild(headGroup);

      // Tail Rattle Rings at destination
      const tailGroup = document.createElementNS(svgNS, "g");
      tailGroup.setAttribute("transform", `translate(${tail.x}, ${tail.y})`);
      for (let r = 0; r < 3; r++) {
        const ring = document.createElementNS(svgNS, "circle");
        ring.setAttribute("cx", "0");
        ring.setAttribute("cy", String(r * 5));
        ring.setAttribute("r", String(6.5 - r * 1.5));
        ring.setAttribute("fill", "#f59e0b");
        ring.setAttribute("stroke", "#78350f");
        ring.setAttribute("stroke-width", "1.5");
        tailGroup.appendChild(ring);
      }
      group.appendChild(tailGroup);

      el.boardSvgOverlay.appendChild(group);
    });
  }

  // -------------------------------------------------------------
  // 5. Token Management & Movement Animation
  // -------------------------------------------------------------
  function renderTokens(players) {
    el.boardTokensLayer.innerHTML = "";

    const tileGroups = {};
    players.forEach(p => {
      const pos = p.position;
      if (!tileGroups[pos]) tileGroups[pos] = [];
      tileGroups[pos].push(p);
    });

    Object.entries(tileGroups).forEach(([posStr, group]) => {
      const pos = parseInt(posStr, 10);
      const coords = getTileCoords(pos);

      group.forEach((p, idx) => {
        const token = document.createElement("div");
        token.className = "player-token";
        token.id = `token-${p.id}`;
        token.textContent = p.avatar;
        token.style.borderColor = p.color;

        let offsetX = 0;
        let offsetY = 0;
        if (group.length > 1) {
          const angle = (idx / group.length) * (2 * Math.PI);
          offsetX = Math.cos(angle) * 1.8;
          offsetY = Math.sin(angle) * 1.8;
        }

        const pctX = (coords.x / 10) + offsetX;
        const pctY = (coords.y / 10) + offsetY;

        token.style.left = `${pctX}%`;
        token.style.top = `${pctY}%`;

        el.boardTokensLayer.appendChild(token);
      });
    });
  }

  function setDiceRotation(cubeEl, value) {
    const rotations = {
      1: "rotateX(0deg) rotateY(0deg)",
      2: "rotateX(0deg) rotateY(180deg)",
      3: "rotateX(0deg) rotateY(90deg)",
      4: "rotateX(0deg) rotateY(-90deg)",
      5: "rotateX(-90deg) rotateY(0deg)",
      6: "rotateX(90deg) rotateY(0deg)"
    };
    cubeEl.style.transform = rotations[value] || rotations[1];
  }

  async function animateMoveStep(step) {
    isAnimatingMove = true;

    // 1. Roll Dice Animation
    const dice = step.roll.dice;
    el.cubeDie1.classList.add("rolling");
    if (dice.length > 1) {
      el.cubeDie2.style.display = "block";
      el.cubeDie2.classList.add("rolling");
    } else {
      el.cubeDie2.style.display = "none";
    }

    if (window.serpientesAudio) window.serpientesAudio.playDiceRoll();

    await new Promise(r => setTimeout(r, 450));
    el.cubeDie1.classList.remove("rolling");
    el.cubeDie2.classList.remove("rolling");
    setDiceRotation(el.cubeDie1, dice[0]);
    if (dice.length > 1) {
      setDiceRotation(el.cubeDie2, dice[1]);
    }

    // 2. Step-by-step token hopping
    const token = document.getElementById(`token-${step.player_id}`);
    if (token && step.path && step.path.length > 0) {
      for (const t of step.path) {
        const coords = getTileCoords(t);
        token.style.left = `${coords.x / 10}%`;
        token.style.top = `${coords.y / 10}%`;
        token.classList.add("hopping");

        if (window.serpientesAudio) window.serpientesAudio.playHop();

        await new Promise(r => setTimeout(r, 160));
        token.classList.remove("hopping");
      }
    }

    // 3. Handle Portal Climb / Slide
    if (step.portal_applied && token) {
      await new Promise(r => setTimeout(r, 220));

      const finalCoords = getTileCoords(step.final_tile);
      token.style.transition = "top 0.65s cubic-bezier(0.25, 1, 0.5, 1), left 0.65s cubic-bezier(0.25, 1, 0.5, 1)";
      token.style.left = `${finalCoords.x / 10}%`;
      token.style.top = `${finalCoords.y / 10}%`;

      if (step.portal_applied.is_ladder) {
        if (window.serpientesAudio) {
          window.serpientesAudio.playLadder();
          window.serpientesAudio.speak(
            `¡${step.player_name} subió por la escalera!`,
            `${step.player_name} climbed a ladder!`
          );
        }
      } else {
        if (window.serpientesAudio) {
          window.serpientesAudio.playSnake();
          window.serpientesAudio.speak(
            `¡${step.player_name} resbaló por la serpiente!`,
            `${step.player_name} slid down a snake!`
          );
        }
      }

      await new Promise(r => setTimeout(r, 700));
      token.style.transition = "";
    }

    // 4. Handle Win Celebration
    if (step.won) {
      if (window.serpientesAudio) {
        window.serpientesAudio.playVictory();
        window.serpientesAudio.speak(
          `¡Victoria! ¡${step.player_name} ha ganado la partida!`,
          `Victory! ${step.player_name} won the game!`
        );
      }
      launchConfetti();
    }

    isAnimatingMove = false;

    if (moveQueue.length > 0) {
      const next = moveQueue.shift();
      await animateMoveStep(next);
    }
  }

  // -------------------------------------------------------------
  // 6. Confetti Cannon
  // -------------------------------------------------------------
  function launchConfetti() {
    const canvas = el.confettiCanvas;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    const colors = ["#ef4444", "#f59e0b", "#10b981", "#3b82f6", "#ec4899", "#8b5cf6", "#fef08a"];
    const particles = [];
    for (let i = 0; i < 140; i++) {
      particles.push({
        x: canvas.width / 2,
        y: canvas.height * 0.45,
        vx: (Math.random() - 0.5) * 16,
        vy: (Math.random() - 0.9) * 18,
        size: Math.random() * 8 + 5,
        color: colors[Math.floor(Math.random() * colors.length)],
        rotation: Math.random() * 360,
        vr: (Math.random() - 0.5) * 12,
        gravity: 0.38,
        alpha: 1
      });
    }

    let animId;
    function render() {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      let alive = false;
      particles.forEach(p => {
        p.vy += p.gravity;
        p.x += p.vx;
        p.y += p.vy;
        p.rotation += p.vr;
        p.alpha -= 0.008;

        if (p.alpha > 0) {
          alive = true;
          ctx.save();
          ctx.globalAlpha = Math.max(0, p.alpha);
          ctx.translate(p.x, p.y);
          ctx.rotate((p.rotation * Math.PI) / 180);
          ctx.fillStyle = p.color;
          ctx.fillRect(-p.size / 2, -p.size / 2, p.size, p.size * 0.6);
          ctx.restore();
        }
      });

      if (alive) {
        animId = requestAnimationFrame(render);
      } else {
        cancelAnimationFrame(animId);
        ctx.clearRect(0, 0, canvas.width, canvas.height);
      }
    }
    render();
  }

  // -------------------------------------------------------------
  // 7. Activity Feed & Reactions (Safe DOM methods)
  // -------------------------------------------------------------
  function renderActivityFeed(logs) {
    if (!logs) return;
    el.activityFeed.innerHTML = "";
    logs.slice().reverse().forEach(log => {
      const div = document.createElement("div");
      div.className = `log-item log-${log.kind}`;
      div.textContent = currentLang === "es" ? log.message_es : log.message_en;
      el.activityFeed.appendChild(div);
    });
  }

  function spawnFloatingEmoji(emoji) {
    const span = document.createElement("span");
    span.className = "floating-emoji";
    span.textContent = emoji;

    const leftPct = 15 + Math.random() * 70;
    span.style.left = `${leftPct}%`;
    span.style.bottom = "80px";

    el.floatingEmojisLayer.appendChild(span);
    setTimeout(() => span.remove(), 2800);
  }

  // -------------------------------------------------------------
  // 8. Room State Synchronization (Safe DOM construction)
  // -------------------------------------------------------------
  function syncRoomState(room) {
    currentRoom = room;

    if (!boardMeta) {
      boardMeta = room.board;
      buildBoardGrid(room.board);
      drawBoardSvg(room.board);
    }

    if (playerId) {
      const me = room.players.find(p => p.id === playerId);
      if (me) isHost = me.is_host;
    }

    if (room.state === "lobby") {
      showScreen("screen-lobby");
      el.lobbyRoomCode.textContent = room.code;

      el.lobbyPlayersList.innerHTML = "";
      room.players.forEach(p => {
        const card = document.createElement("div");
        card.className = `lobby-player-card ${p.is_host ? "is-host" : ""}`;

        const avatarDiv = document.createElement("div");
        avatarDiv.className = "lobby-player-avatar";
        avatarDiv.textContent = p.avatar;

        if (p.is_host) {
          const crown = document.createElement("span");
          crown.className = "lobby-host-crown";
          crown.textContent = "👑";
          avatarDiv.appendChild(crown);
        }
        card.appendChild(avatarDiv);

        const nameDiv = document.createElement("div");
        nameDiv.className = "lobby-player-name";
        nameDiv.textContent = p.name;
        card.appendChild(nameDiv);

        const dot = document.createElement("div");
        dot.className = "lobby-player-color-dot";
        dot.style.backgroundColor = p.color;
        card.appendChild(dot);

        el.lobbyPlayersList.appendChild(card);
      });

      if (isHost) {
        el.btnStartGame.style.display = "flex";
        el.btnStartGame.disabled = room.players.length < 1;
      } else {
        el.btnStartGame.style.display = "none";
      }
    } else if (room.state === "playing") {
      showScreen("screen-game");

      if (!isAnimatingMove) {
        renderTokens(room.players);
      }

      // Update Turn Banner
      const curPlayer = room.players[room.turn_index % room.players.length];
      if (curPlayer) {
        el.turnAvatar.textContent = curPlayer.avatar;
        el.turnAvatar.style.borderColor = curPlayer.color;
        el.turnPlayerName.textContent = curPlayer.name;

        const isMyTurn = (curPlayer.id === playerId);
        el.btnRollDice.disabled = !isMyTurn;

        const turnBanner = document.querySelector(".turn-banner");
        const turnTitle = document.querySelector(".turn-title");
        if (isMyTurn) {
          if (turnBanner) turnBanner.classList.add("my-turn");
          if (turnTitle) turnTitle.textContent = currentLang === "es" ? "⭐ ¡ES TU TURNO! ⭐" : "⭐ YOUR TURN! ⭐";
          el.btnRollDice.classList.add("my-turn-pulse");
          el.btnRollDice.textContent = i18n[currentLang].btnRollDice;
        } else {
          if (turnBanner) turnBanner.classList.remove("my-turn");
          if (turnTitle) turnTitle.textContent = i18n[currentLang].turnTitle;
          el.btnRollDice.classList.remove("my-turn-pulse");
          el.btnRollDice.textContent = `⏳ ${i18n[currentLang].waitingRoll} ${curPlayer.name}...`;
        }
      }

      renderActivityFeed(room.logs);

      if (room.settings.dice_count > 1) {
        el.cubeDie2.style.display = "block";
      } else {
        el.cubeDie2.style.display = "none";
      }
    } else if (room.state === "finished") {
      showScreen("screen-game");
      renderTokens(room.players);
      renderActivityFeed(room.logs);

      if (room.winner) {
        el.winnerAvatar.textContent = room.winner.avatar;
        el.winnerName.textContent = room.winner.player_name;
        el.winnerStatRolls.textContent = room.winner.total_rolls;
        el.winnerStatLadders.textContent = room.winner.ladders_climbed;
        el.winnerStatSnakes.textContent = room.winner.snakes_hit;

        el.btnPlayAgain.style.display = isHost ? "flex" : "none";
        el.modalWinner.classList.add("active");

        // Grand fiesta victory confetti barrage
        if (typeof window.confetti === "function") {
          window.confetti({ particleCount: 110, spread: 80, origin: { y: 0.6 } });
          setTimeout(() => window.confetti({ particleCount: 70, angle: 60, spread: 55, origin: { x: 0 } }), 350);
          setTimeout(() => window.confetti({ particleCount: 70, angle: 120, spread: 55, origin: { x: 1 } }), 700);
        }
      }
    }
  }

  // -------------------------------------------------------------
  // 9. WebSocket Connection
  // -------------------------------------------------------------
  function connectWebSocket() {
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    const url = `${proto}//${location.host}/ws`;

    ws = new WebSocket(url);

    ws.onopen = () => {
      console.log("WebSocket connected.");
      const session = getStoredSession();
      const params = new URLSearchParams(window.location.search);
      const urlRoom = params.get("room");

      // Auto-reconnect if session exists
      if (session && session.roomCode && session.playerId) {
        if (el.reconnectingOverlay) el.reconnectingOverlay.style.display = "block";
        if (el.resumeBanner) {
          el.resumeRoomCode.textContent = session.roomCode;
          el.resumePlayerName.textContent = session.playerName;
          el.resumeBanner.style.display = "flex";
        }
        ws.send(JSON.stringify({
          type: "join_room",
          roomCode: session.roomCode,
          playerName: session.playerName,
          avatar: session.avatar,
          color: session.color,
          playerId: session.playerId
        }));
      } else if (urlRoom && !currentRoom) {
        el.tabJoin.click();
        el.joinCode.value = urlRoom.toUpperCase();
      }
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);

        switch (msg.type) {
          case "room_joined":
            roomCode = msg.roomCode;
            playerId = msg.playerId;
            isHost = msg.isHost;

            if (el.reconnectingOverlay) el.reconnectingOverlay.style.display = "none";
            if (el.resumeBanner) el.resumeBanner.style.display = "none";

            const myPlayer = msg.room.players.find(p => p.id === playerId);
            saveSession(
              roomCode,
              playerId,
              myPlayer ? myPlayer.name : el.joinName.value || el.createName.value || "Jugador",
              myPlayer ? myPlayer.avatar : selectedAvatar,
              myPlayer ? myPlayer.color : selectedColor,
              isHost
            );

            syncRoomState(msg.room);
            break;

          case "room_state":
            syncRoomState(msg.room);
            break;

          case "player_moved":
            if (isAnimatingMove) {
              moveQueue.push(msg.step);
            } else {
              animateMoveStep(msg.step);
            }
            break;

          case "reaction":
            spawnFloatingEmoji(msg.emoji);
            if (window.serpientesAudio) window.serpientesAudio.playReactionPop();
            break;

          case "error":
            if (el.reconnectingOverlay && el.reconnectingOverlay.style.display !== "none") {
              el.reconnectingOverlay.style.display = "none";
              clearSession();
              showScreen("screen-welcome");
              showToast(i18n[currentLang].toastGameEnded);
            } else {
              showToast(`⚠️ ${msg.message}`);
            }
            break;

          default:
            break;
        }
      } catch (err) {
        console.error("Failed to parse server message", err);
      }
    };

    ws.onclose = () => {
      console.warn("WebSocket closed. Reconnecting in 2s...");
      setTimeout(connectWebSocket, 2000);
    };

    setInterval(() => {
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: "ping" }));
      }
    }, 25000);
  }

  // -------------------------------------------------------------
  // 10. Event Listeners & Interaction
  // -------------------------------------------------------------
  function initEventListeners() {
    el.btnToggleLang.addEventListener("click", toggleLanguage);
    el.btnToggleTheme.addEventListener("click", toggleTheme);

    // Audio toggles
    el.btnToggleSound.addEventListener("click", () => {
      if (!window.serpientesAudio) return;
      window.serpientesAudio.soundEnabled = !window.serpientesAudio.soundEnabled;
      el.btnToggleSound.textContent = window.serpientesAudio.soundEnabled ? "🔊" : "🔇";
      showToast(window.serpientesAudio.soundEnabled ? "Sonido activado" : "Sonido desactivado");
    });

    el.btnToggleSpeech.addEventListener("click", () => {
      if (!window.serpientesAudio) return;
      window.serpientesAudio.speechEnabled = !window.serpientesAudio.speechEnabled;
      el.btnToggleSpeech.textContent = window.serpientesAudio.speechEnabled ? "🗣️" : "🤐";
      showToast(window.serpientesAudio.speechEnabled ? "Voz activada" : "Voz desactivada");
    });

    // Leave Room
    el.btnHeaderLeave.addEventListener("click", () => {
      if (confirm(i18n[currentLang].confirmLeave)) {
        if (ws && ws.readyState === WebSocket.OPEN && roomCode) {
          ws.send(JSON.stringify({
            type: "leave_room",
            roomCode
          }));
        }
        clearSession();
        location.href = "/";
      }
    });

    // Resume Game Banner Controls
    if (el.btnResumeGame) {
      el.btnResumeGame.addEventListener("click", () => {
        const session = getStoredSession();
        if (session && ws && ws.readyState === WebSocket.OPEN) {
          if (el.reconnectingOverlay) el.reconnectingOverlay.style.display = "block";
          ws.send(JSON.stringify({
            type: "join_room",
            roomCode: session.roomCode,
            playerName: session.playerName,
            avatar: session.avatar,
            color: session.color,
            playerId: session.playerId
          }));
        }
      });
    }

    if (el.btnDismissResume) {
      el.btnDismissResume.addEventListener("click", () => {
        clearSession();
      });
    }

    // Mobile Activity Feed Drawer Toggle
    el.btnToggleFeedMobile.addEventListener("click", () => {
      el.drawerActivityLog.classList.toggle("drawer-open");
    });
    el.btnCloseFeedDrawer.addEventListener("click", () => {
      el.drawerActivityLog.classList.remove("drawer-open");
    });

    // Welcome Tabs
    el.tabCreate.addEventListener("click", () => {
      el.tabCreate.classList.add("active");
      el.tabJoin.classList.remove("active");
      el.formCreate.style.display = "block";
      el.formJoin.style.display = "none";
    });

    el.tabJoin.addEventListener("click", () => {
      el.tabJoin.classList.add("active");
      el.tabCreate.classList.remove("active");
      el.formCreate.style.display = "none";
      el.formJoin.style.display = "block";
    });

    // Avatar Picker
    el.avatarPicker.innerHTML = "";
    AVATARS.forEach((av, idx) => {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = `avatar-chip ${idx === 0 ? "selected" : ""}`;
      chip.textContent = av;
      chip.addEventListener("click", () => {
        document.querySelectorAll(".avatar-chip").forEach(c => c.classList.remove("selected"));
        chip.classList.add("selected");
        selectedAvatar = av;
        if (window.serpientesAudio) window.serpientesAudio.playClick();
      });
      el.avatarPicker.appendChild(chip);
    });

    // Color Picker
    el.colorPicker.innerHTML = "";
    COLORS.forEach((col, idx) => {
      const swatch = document.createElement("button");
      swatch.type = "button";
      swatch.className = `color-swatch ${idx === 0 ? "selected" : ""}`;
      swatch.style.backgroundColor = col;
      swatch.addEventListener("click", () => {
        document.querySelectorAll(".color-swatch").forEach(s => s.classList.remove("selected"));
        swatch.classList.add("selected");
        selectedColor = col;
        if (window.serpientesAudio) window.serpientesAudio.playClick();
      });
      el.colorPicker.appendChild(swatch);
    });

    // Create Room Submit
    el.formCreate.addEventListener("submit", (e) => {
      e.preventDefault();
      const name = el.createName.value.trim();
      if (!name) {
        showToast(i18n[currentLang].errorFillName);
        return;
      }

      const settings = {
        dice_count: parseInt(el.settingDiceCount.value, 10) || 1,
        exact_win: el.settingExactWin.value === "true",
        bonus_on_six: el.settingBonusSix.checked,
        three_sixes_penalty: el.settingThreeSixes.checked,
        turn_timer_seconds: parseInt(el.settingTurnTimer.value, 10) || 25,
        board_theme: THEMES[currentThemeIdx]
      };

      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          type: "create_room",
          playerName: name,
          avatar: selectedAvatar,
          color: selectedColor,
          settings
        }));
      }
    });

    // Join Room Submit
    el.formJoin.addEventListener("submit", (e) => {
      e.preventDefault();
      const name = el.joinName.value.trim();
      const code = el.joinCode.value.trim().toUpperCase();

      if (!name) {
        showToast(i18n[currentLang].errorFillName);
        return;
      }
      if (!code) {
        showToast(i18n[currentLang].errorFillCode);
        return;
      }

      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
          type: "join_room",
          roomCode: code,
          playerName: name,
          avatar: selectedAvatar,
          color: selectedColor,
          playerId: playerId
        }));
      }
    });

    // Copy Link Buttons
    function copyInviteLink() {
      const inviteUrl = `${window.location.origin}/?room=${roomCode}`;
      navigator.clipboard.writeText(inviteUrl).then(() => {
        showToast(i18n[currentLang].toastCopied);
      });
    }
    el.btnCopyLink.addEventListener("click", copyInviteLink);
    el.btnCopyModalLink.addEventListener("click", copyInviteLink);

    // QR Code Modal
    el.btnShowQr.addEventListener("click", () => {
      const inviteUrl = `${window.location.origin}/?room=${roomCode}`;
      el.qrShareLink.value = inviteUrl;
      el.qrContainer.innerHTML = "";

      if (window.QRCode) {
        new window.QRCode(el.qrContainer, {
          text: inviteUrl,
          width: 190,
          height: 190,
          colorDark: "#1c1917",
          colorLight: "#ffffff"
        });
      }
      el.modalQr.classList.add("active");
    });

    // Close Modals
    document.querySelectorAll(".modal-close").forEach(btn => {
      btn.addEventListener("click", () => {
        document.querySelectorAll(".modal-overlay").forEach(m => m.classList.remove("active"));
      });
    });

    // Start Game (Host only)
    el.btnStartGame.addEventListener("click", () => {
      if (ws && ws.readyState === WebSocket.OPEN && roomCode) {
        ws.send(JSON.stringify({
          type: "start_game",
          roomCode
        }));
      }
    });

    // Roll Dice trigger helper
    function triggerRoll() {
      if (ws && ws.readyState === WebSocket.OPEN && roomCode && !isAnimatingMove && !el.btnRollDice.disabled) {
        ws.send(JSON.stringify({
          type: "roll_dice",
          roomCode
        }));
      }
    }

    // Roll on button click or tapping the 3D dice stage
    el.btnRollDice.addEventListener("click", triggerRoll);
    el.diceContainerBox.addEventListener("click", triggerRoll);

    // Reactions Bar
    el.reactionsBar.addEventListener("click", (e) => {
      const btn = e.target.closest(".btn-react");
      if (btn && ws && ws.readyState === WebSocket.OPEN && roomCode) {
        const emoji = btn.getAttribute("data-emoji");
        ws.send(JSON.stringify({
          type: "send_reaction",
          roomCode,
          emoji
        }));
      }
    });

    // Gritos y Frases de Feria (El Cotorreo)
    const cotorreoBar = document.getElementById("cotorreo-bar");
    if (cotorreoBar) {
      cotorreoBar.addEventListener("click", (e) => {
        const btn = e.target.closest(".btn-cotorreo");
        if (btn && ws && ws.readyState === WebSocket.OPEN && roomCode) {
          const shout = btn.getAttribute("data-shout");
          ws.send(JSON.stringify({
            type: "send_reaction",
            roomCode,
            emoji: shout
          }));
        }
      });
    }

    // Play Again
    el.btnPlayAgain.addEventListener("click", () => {
      el.modalWinner.classList.remove("active");
      if (ws && ws.readyState === WebSocket.OPEN && roomCode) {
        ws.send(JSON.stringify({
          type: "restart_game",
          roomCode
        }));
      }
    });
  }

  // -------------------------------------------------------------
  // Initialization
  // -------------------------------------------------------------
  document.addEventListener("DOMContentLoaded", () => {
    updateLanguage();
    checkResumeBanner();
    updateAmbientDecorations(THEMES[currentThemeIdx]);
    initEventListeners();
    connectWebSocket();
  });

})();
