// Web Audio API Sound Synthesizer & Speech Locution for Serpientes y Escaleras
// 100% self-contained — no external mp3/wav files required!

class SerpientesAudio {
  constructor() {
    this.ctx = null;
    this.soundEnabled = true;
    this.speechEnabled = true;
    this.currentLang = 'es';
    this.spanishVoice = null;
    this.englishVoice = null;

    if ('speechSynthesis' in window) {
      const loadVoices = () => {
        const voices = window.speechSynthesis.getVoices();
        this.spanishVoice = voices.find(v => v.lang.startsWith('es') || v.lang.includes('ES') || v.lang.includes('MX')) || null;
        this.englishVoice = voices.find(v => v.lang.startsWith('en') || v.lang.includes('US') || v.lang.includes('GB')) || null;
      };
      loadVoices();
      window.speechSynthesis.onvoiceschanged = loadVoices;
    }
  }

  init() {
    if (!this.ctx) {
      const AudioContext = window.AudioContext || window.webkitAudioContext;
      this.ctx = new AudioContext();
    }
    if (this.ctx && this.ctx.state === 'suspended') {
      this.ctx.resume();
    }
  }

  // Realistic dice rattle / rolling clatter
  playDiceRoll() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    // Simulate 4-5 quick dice bounces/tumbles
    const clicks = [0, 0.04, 0.09, 0.15, 0.22, 0.30];
    clicks.forEach((delay, i) => {
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();
      const filter = this.ctx.createBiquadFilter();

      filter.type = 'bandpass';
      filter.frequency.setValueAtTime(800 + (Math.random() * 400), t + delay);
      filter.Q.setValueAtTime(3.0, t + delay);

      osc.type = 'triangle';
      const freq = 180 + Math.random() * 120;
      osc.frequency.setValueAtTime(freq, t + delay);
      osc.frequency.exponentialRampToValueAtTime(60, t + delay + 0.04);

      const vol = 0.35 * (1 - (i / clicks.length) * 0.4);
      gain.gain.setValueAtTime(vol, t + delay);
      gain.gain.exponentialRampToValueAtTime(0.001, t + delay + 0.04);

      osc.connect(filter);
      filter.connect(gain);
      gain.connect(this.ctx.destination);

      osc.start(t + delay);
      osc.stop(t + delay + 0.04);
    });
  }

  // Token hopping on a tile
  playHop() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(380, t);
    osc.frequency.exponentialRampToValueAtTime(140, t + 0.06);

    gain.gain.setValueAtTime(0.28, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.06);

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start(t);
    osc.stop(t + 0.06);
  }

  // Triumphant ascending arpeggio for climbing a ladder (Escalera)
  playLadder() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    // Bright ascending notes: C5, E5, G5, C6, E6
    const notes = [523.25, 659.25, 783.99, 1046.50, 1318.51];
    notes.forEach((freq, idx) => {
      const startT = t + idx * 0.09;
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();

      osc.type = 'triangle';
      osc.frequency.setValueAtTime(freq, startT);

      gain.gain.setValueAtTime(0.3, startT);
      gain.gain.exponentialRampToValueAtTime(0.001, startT + 0.35);

      osc.connect(gain);
      gain.connect(this.ctx.destination);

      osc.start(startT);
      osc.stop(startT + 0.35);
    });
  }

  // Descending slide whistle & playful hiss for snake bite (Serpiente)
  playSnake() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    // 1. Descending slide tone
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(650, t);
    osc.frequency.exponentialRampToValueAtTime(110, t + 0.55);

    const filter = this.ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(1200, t);
    filter.frequency.exponentialRampToValueAtTime(300, t + 0.55);

    gain.gain.setValueAtTime(0.25, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.55);

    osc.connect(filter);
    filter.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start(t);
    osc.stop(t + 0.55);

    // 2. Snake hiss noise burst at the end
    try {
      const bufferSize = this.ctx.sampleRate * 0.25;
      const noiseBuffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
      const output = noiseBuffer.getChannelData(0);
      for (let i = 0; i < bufferSize; i++) {
        output[i] = Math.random() * 2 - 1;
      }

      const whiteNoise = this.ctx.createBufferSource();
      whiteNoise.buffer = noiseBuffer;

      const noiseFilter = this.ctx.createBiquadFilter();
      noiseFilter.type = 'bandpass';
      noiseFilter.frequency.setValueAtTime(3200, t + 0.3);
      noiseFilter.Q.setValueAtTime(2.0, t + 0.3);

      const noiseGain = this.ctx.createGain();
      noiseGain.gain.setValueAtTime(0.001, t + 0.3);
      noiseGain.gain.linearRampToValueAtTime(0.18, t + 0.38);
      noiseGain.gain.exponentialRampToValueAtTime(0.001, t + 0.55);

      whiteNoise.connect(noiseFilter);
      noiseFilter.connect(noiseGain);
      noiseGain.connect(this.ctx.destination);

      whiteNoise.start(t + 0.3);
      whiteNoise.stop(t + 0.55);
    } catch (_) {}
  }

  // Triumphant victory fanfare (Ascending celebration chords)
  playVictory() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    const chords = [
      { t: 0.00, notes: [523.25, 659.25], dur: 0.14 }, // C5, E5
      { t: 0.16, notes: [523.25, 659.25], dur: 0.14 },
      { t: 0.32, notes: [523.25, 659.25], dur: 0.14 },
      { t: 0.50, notes: [659.25, 783.99], dur: 0.35 }, // E5, G5
      { t: 0.88, notes: [587.33, 698.46], dur: 0.18 }, // D5, F5
      { t: 1.10, notes: [783.99, 1046.50], dur: 0.80 }, // G5, C6 (grand finish)
    ];

    chords.forEach(c => {
      c.notes.forEach(freq => {
        const osc = this.ctx.createOscillator();
        const gain = this.ctx.createGain();

        osc.type = 'triangle';
        osc.frequency.setValueAtTime(freq, t + c.t);

        gain.gain.setValueAtTime(0.3, t + c.t);
        gain.gain.exponentialRampToValueAtTime(0.001, t + c.t + c.dur);

        osc.connect(gain);
        gain.connect(this.ctx.destination);

        osc.start(t + c.t);
        osc.stop(t + c.t + c.dur);
      });
    });
  }

  // Playful pop for emoji reactions
  playReactionPop() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(600, t);
    osc.frequency.exponentialRampToValueAtTime(1200, t + 0.08);

    gain.gain.setValueAtTime(0.2, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.08);

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start(t);
    osc.stop(t + 0.08);
  }

  // Button click
  playClick() {
    if (!this.soundEnabled) return;
    this.init();
    const t = this.ctx.currentTime;

    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();

    osc.type = 'sine';
    osc.frequency.setValueAtTime(400, t);
    osc.frequency.exponentialRampToValueAtTime(200, t + 0.03);

    gain.gain.setValueAtTime(0.12, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.03);

    osc.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start(t);
    osc.stop(t + 0.03);
  }

  // Voice Caller / Locución
  speak(textEs, textEn) {
    if (!this.speechEnabled || !('speechSynthesis' in window)) return;
    window.speechSynthesis.cancel(); // Stop any pending utterances

    const text = (this.currentLang === 'es') ? textEs : textEn;
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.rate = 1.05;
    utterance.pitch = 1.05;

    if (this.currentLang === 'es') {
      utterance.lang = 'es-MX';
      if (this.spanishVoice) utterance.voice = this.spanishVoice;
    } else {
      utterance.lang = 'en-US';
      if (this.englishVoice) utterance.voice = this.englishVoice;
    }

    window.speechSynthesis.speak(utterance);
  }
}

window.serpientesAudio = new SerpientesAudio();
