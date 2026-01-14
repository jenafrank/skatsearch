import init, { SkatGame, init_panic_hook } from './pkg/skat_aug23.js';

let game = null;
let lastTrickId = null;
let lastTrickSize = 0;
let cheatMode = false;
let analysisTimeout = null; // Debounce for analysis

async function run() {
    console.log("Initializing WASM...");
    await init();
    init_panic_hook();
    console.log("WASM Initialized.");
    document.getElementById('game-status').textContent = "Game: Clubs (Declarer)";
    bindEvents();
    startNewGame();
}

function bindEvents() {
    document.getElementById('btn-new-game').onclick = startNewGame;
    document.getElementById('btn-hint').onclick = showHint;
    document.getElementById('btn-undo').onclick = undoMove;
    document.getElementById('btn-cheat').onclick = toggleCheatMode;
}

function toggleCheatMode() {
    cheatMode = !cheatMode;
    const btn = document.getElementById('btn-cheat');
    btn.textContent = cheatMode ? "Karten verbergen" : "Karten zeigen";
    updateUI();
}

function startNewGame() {
    lastTrickId = null;
    lastTrickSize = 0;

    // Create new game
    game = SkatGame.new_random();

    // Trigger async calculation of Max Points (if available)
    setTimeout(() => {
        if (game && game.calculate_max_points) {
            console.log("Calculating Max Points Async...");
            try {
                game.calculate_max_points(); // updates internal state
                updateUI(); // Reflect new max points
            } catch (e) {
                console.error("Max points calc failed", e);
            }
        }
    }, 500);

    document.getElementById('left-points').style.display = 'none';
    document.getElementById('my-tricks-pile').style.display = 'none';
    document.getElementById('my-points').textContent = '0';
    document.getElementById('left-points').textContent = '0';
    document.getElementById('game-over-overlay').style.display = 'none';

    updateUI();
    gameLoop();
}

async function gameLoop() {
    if (!game) return;

    let state = game.get_state_json();

    // Detect Completed Trick to Await Animation FIRST
    if (state.last_trick_cards && state.last_trick_cards !== lastTrickId) {
        lastTrickId = state.last_trick_cards;

        // Render the completed trick using ordered plays
        renderTable(state.last_trick_plays);

        // Wait for user to see the trick
        await new Promise(r => setTimeout(r, 1200));

        // Animate collection to winner
        await animateTrickCollection(state.last_trick_winner);

        // Clear table
        document.getElementById('trick-cards').innerHTML = '';
        lastTrickSize = 0;

        await new Promise(r => setTimeout(r, 100));

        updatePoints(state);
        triggerAnalysis(); // Update history logic
    }

    // Re-fetch state
    state = game.get_state_json();

    if (state.game_over) {
        showGameOver(state);
        document.getElementById('game-status').textContent = `Game Over! Winner: ${state.winner}`;
        return;
    }

    const currentPlayer = state.current_player;

    if (currentPlayer !== "D" && !state.game_over) {
        document.getElementById('game-status').textContent = `Status: ${currentPlayer === 'L' ? 'Left' : 'Right'} AI thinking...`;

        await new Promise(r => setTimeout(r, 1000));

        const moved = game.make_ai_move();
        if (moved) {
            updateUI();
            setTimeout(gameLoop, 50);
        } else {
            console.log("AI refused to move?");
        }
    } else if (currentPlayer === "D") {
        document.getElementById('game-status').textContent = "Your Turn (Declarer)";
        // Ensure analysis is up to date if we just arrived here via trick clear
        triggerAnalysis();
    }
}

async function triggerAnalysis() {
    if (!game || !game.calculate_analysis) return;

    if (analysisTimeout) clearTimeout(analysisTimeout);

    analysisTimeout = setTimeout(() => {
        try {
            // This updates internal state
            game.calculate_analysis();

            // Re-render info panel directly
            const state = game.get_state_json();
            renderInfo(state);
        } catch (e) {
            console.error("Analysis failed", e);
        }
    }, 100);
}

async function animateTrickCollection(winner) {
    const trickCards = document.querySelectorAll('#trick-cards .trick-card-item');
    if (trickCards.length === 0) return;

    let targetEl = null;

    // Winner string is "Declarer" or "Left" or "Right"
    const w = winner ? winner[0] : "";

    if (w === "D") {
        targetEl = document.getElementById('my-points');
        if (!targetEl || targetEl.offsetParent === null) targetEl = document.getElementById('player-info');
    } else {
        targetEl = document.getElementById('left-points');
        if (!targetEl || targetEl.offsetParent === null) targetEl = document.getElementById('opponent-left');
    }

    if (!targetEl) return;
    const targetRect = targetEl.getBoundingClientRect();

    const animations = [];
    trickCards.forEach(el => {
        const rect = el.getBoundingClientRect();
        const deltaX = targetRect.left - rect.left;
        const deltaY = targetRect.top - rect.top;

        el.style.transition = 'transform 0.6s ease-in, opacity 0.6s ease-in';
        el.style.transform = `translate(${deltaX}px, ${deltaY}px) scale(0.2)`;
        el.style.opacity = '0';
        el.style.zIndex = 100;

        animations.push(new Promise(r => setTimeout(r, 600)));
    });

    await Promise.all(animations);
}

function showGameOver(state) {
    const overlay = document.getElementById('game-over-overlay');
    const msg = document.getElementById('game-result-message');
    const score = document.getElementById('game-result-score');

    const isWin = state.winner === "Declarer";
    const text = isWin ? "Gewonnen!" : "Verloren!";

    msg.textContent = text;
    msg.style.color = isWin ? "#2ecc71" : "#e74c3c";

    score.innerHTML = `
        <div class="score-line">
            <span class="score-own">${state.declarer_points}</span> : 
            <span class="score-opp">${state.team_points}</span>
        </div>
        <div class="score-max">(${state.max_possible_points} max)</div>
    `;

    overlay.style.display = 'flex';
    document.getElementById('trick-cards').innerHTML = '';
    lastTrickSize = 0;
}

function playCard(cardStr) {
    if (!game) return;

    let state = game.get_state_json();
    if (state.current_player !== "D") {
        console.log("Not your turn");
        return;
    }

    const res = game.play_card_str(cardStr);
    if (res) {
        const newState = game.get_state_json();
        // If trick just completed (empty plays now), let gameLoop handle animation
        if (newState.trick_plays.length === 0 && newState.last_trick_cards) {
            renderHand(newState.my_cards, newState.legal_moves); // Update hand immediately (remove played card)
            gameLoop();
        } else {
            updateUI();
            gameLoop();
        }
    } else {
        console.log("Invalid move");
    }
}

function undoMove() {
    if (game) {
        // 1. Undo at least one step (the user's last action decision point is in the past)
        game.undo();

        // 2. Continue undoing until it is the User's turn (Declarer)
        // or we reached the start of the game.
        let state = game.get_state_json();
        let attempts = 0;

        // We look for 'D' (Declarer) as current player.
        // Also check history length to avoid infinite loop at start.
        while (state.current_player !== "D" && state.move_history.length > 0 && attempts < 20) {
            game.undo();
            state = game.get_state_json();
            attempts++;
        }

        lastTrickId = null;
        lastTrickSize = 0;
        document.getElementById('game-over-overlay').style.display = 'none';
        document.getElementById('trick-cards').innerHTML = '';


        // Reset analysis
        document.querySelectorAll('.hint-overlay').forEach(e => e.remove());
        document.querySelectorAll('.card-optimal').forEach(e => e.classList.remove('card-optimal'));

        updateUI();
        gameLoop();
    }
}

function showHint() {
    if (game) {
        const btn = document.getElementById('btn-hint');
        const originalText = btn.textContent;
        btn.textContent = "Berechne...";
        btn.disabled = true;

        setTimeout(() => {
            try {
                const analysis = game.get_move_analysis_json();
                if (!analysis) {
                    alert("Kein Tipp verfügbar.");
                    return;
                }

                // Clear previous hints
                document.querySelectorAll('.hint-overlay').forEach(e => e.remove());
                document.querySelectorAll('.card-optimal').forEach(e => e.classList.remove('card-optimal'));

                // Apply new hints
                analysis.forEach(entry => {
                    const cardEl = document.querySelector(`.card[data-card="${entry.card}"]`);
                    if (cardEl) {
                        const delta = entry.delta;
                        const isBest = entry.is_best;

                        const overlay = document.createElement('div');
                        overlay.className = 'hint-overlay';
                        if (isBest) {
                            overlay.classList.add('hint-best');
                            overlay.textContent = "0"; // or "Opt"
                            cardEl.classList.add('card-optimal');
                        } else {
                            overlay.classList.add('hint-bad');
                            overlay.textContent = delta; // e.g. -5
                        }
                        cardEl.appendChild(overlay);
                    }
                });
            } catch (e) {
                console.error("Hint failed", e);
                alert("Fehler bei der Berechnung.");
            } finally {
                btn.textContent = originalText;
                btn.disabled = false;
            }
        }, 50);
    }
}

function updateUI() {
    if (!game) return;
    const state = game.get_state_json();

    renderHand(state.my_cards, state.legal_moves);
    renderOpponents(state);
    renderSkat(state.skat_cards);

    if (state.trick_plays) {
        renderTable(state.trick_plays);
    }

    renderInfo(state);
    updatePoints(state);

    // Trigger Async Analysis
    triggerAnalysis();
}

function renderTable(trickPlays) {
    const tableDiv = document.getElementById('trick-cards');
    if (!trickPlays) trickPlays = [];

    // Reset if cleared externally
    if (trickPlays.length === 0 && lastTrickSize > 0) {
        if (tableDiv.innerHTML === '') {
            lastTrickSize = 0;
        }
    }

    if (trickPlays.length === 0) {
        if (lastTrickSize === 0) tableDiv.innerHTML = '';
        return;
    }

    if (trickPlays.length < lastTrickSize) {
        tableDiv.innerHTML = '';
        lastTrickSize = 0;
    }

    for (let i = lastTrickSize; i < trickPlays.length; i++) {
        const play = trickPlays[i];
        const el = createCardElement(play.card, play.player);
        el.classList.add('trick-card-item');
        el.style.zIndex = i;
        if (i > 0) {
            el.style.marginLeft = "-40px";
            el.style.marginBottom = `${i * 5}px`;
        }
        tableDiv.appendChild(el);
        animateEntrance(el, play);
    }
    lastTrickSize = trickPlays.length;
}

function animateEntrance(el, playInfo) {
    let sourceRect = null;
    const p = playInfo.player;
    if (p === "D") {
        const handDiv = document.getElementById('my-hand');
        if (handDiv) {
            const rect = handDiv.getBoundingClientRect();
            sourceRect = { left: rect.left + rect.width / 2 - 40, top: rect.top };
        }
    } else if (p === "L") {
        const handDiv = document.getElementById('hand-left');
        if (handDiv) {
            const rect = handDiv.getBoundingClientRect();
            sourceRect = { left: rect.left, top: rect.top };
        }
    } else if (p === "R") {
        const handDiv = document.getElementById('hand-right');
        if (handDiv) {
            const rect = handDiv.getBoundingClientRect();
            sourceRect = { left: rect.left, top: rect.top };
        }
    }

    if (sourceRect) {
        const targetRect = el.getBoundingClientRect();
        const deltaX = sourceRect.left - targetRect.left;
        const deltaY = sourceRect.top - targetRect.top;
        el.style.transition = 'none';
        el.style.transform = `translate(${deltaX}px, ${deltaY}px)`;
        el.offsetHeight;
        el.style.transition = 'transform 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275)';
        el.style.transform = 'translate(0, 0)';
    }
}

function renderOpponents(state) {
    const leftContainer = document.getElementById('hand-left');
    const rightContainer = document.getElementById('hand-right');
    if (!leftContainer || !rightContainer) return;

    if (cheatMode) {
        renderOpponentHand(leftContainer, state.left_cards, 'left');
        renderOpponentHand(rightContainer, state.right_cards, 'right');
    } else {
        const leftCount = getCardCount(state.left_cards);
        const rightCount = getCardCount(state.right_cards);
        renderHandBacks(leftContainer, leftCount, 'left-points');
        renderHandBacks(rightContainer, rightCount, null);
    }
}

function getCardCount(cardStr) {
    if (!cardStr) return 0;
    const clean = cardStr.replace(/\[|\]/g, '').trim();
    if (!clean || clean.length === 0) return 0;
    return clean.split(/\s+/).length;
}

function renderHandBacks(container, count, badgeId) {
    container.innerHTML = '';
    for (let i = 0; i < count; i++) {
        const div = document.createElement('div');
        div.className = 'hand-back opponent-card-back';
        if (i > 0) div.style.marginLeft = "-40px";
        container.appendChild(div);
    }
    if (badgeId) {
        const badgeDiv = document.createElement('div');
        badgeDiv.className = 'opponent-points-pile';
        badgeDiv.innerHTML = `<div class="points-badge" id="${badgeId}" style="display:none">0</div>`;
        container.appendChild(badgeDiv);
    }
}

function renderOpponentHand(container, cardsStr, side) {
    container.innerHTML = '';
    if (!cardsStr) return;
    const cleanStr = cardsStr.replace(/\[|\]/g, '').trim();
    if (cleanStr.length === 0) return;
    const cardList = cleanStr.split(/\s+/);
    cardList.forEach(cStr => {
        const el = createCardElement(cStr, null, true); // Use simplified mode
        container.appendChild(el);
    });
}

function renderSkat(skatStr) {
    let skatDiv = document.getElementById('skat-display');
    if (!skatDiv) return;
    if (cheatMode && skatStr) {
        skatDiv.style.display = 'flex';
        skatDiv.innerHTML = '';
        const cleanStr = skatStr.replace(/\[|\]/g, '').trim();
        const cards = cleanStr.split(/\s+/);
        cards.forEach(c => {
            const el = createCardElement(c);
            el.style.transform = "scale(0.8)";
            el.style.margin = "0 5px";
            skatDiv.appendChild(el);
        });
        const label = document.createElement('div');
        label.textContent = "Skat";
        label.style.position = "absolute";
        label.style.top = "-20px";
        label.style.color = "#fff";
        label.style.fontSize = "12px";
        skatDiv.appendChild(label);
    } else {
        skatDiv.style.display = 'none';
        skatDiv.innerHTML = '';
    }
}

function renderHand(cards, legalMoves) {
    const handDiv = document.getElementById('my-hand');
    handDiv.innerHTML = '';
    if (!cards) return;
    const cleanStr = cards.replace(/\[|\]/g, '').trim();
    if (cleanStr.length === 0) return;
    const cardList = cleanStr.split(/\s+/);

    // Normalize legal moves for comparison (trim strings)
    const legalSet = new Set(legalMoves ? legalMoves.map(m => m.trim()) : []);
    // If legalMoves is empty/null (e.g. game over or not turn), maybe disable all? 
    // Or if turn is not ours?
    // Let's rely on passed legalMoves. If empty, everything disabled?
    // If legalMoves is null (old state?), allow all.
    // If legalMoves is [], means no moves possible (wait).

    cardList.forEach(cStr => {
        const el = createCardElement(cStr);
        if (legalMoves && !legalSet.has(cStr)) {
            el.classList.add('card-disabled');
            // Remove click handler or make it no-op?
            // Better to keep onclick and check, or just visual.
            // visual is enough, click handler validates in Rust anyway or we can guard.
        } else {
            el.onclick = () => playCard(cStr);
        }
        handDiv.appendChild(el);
    });
}

function updatePoints(state) {
    const myPile = document.getElementById('my-tricks-pile');
    const myBadge = document.getElementById('my-points');
    const leftBadge = document.getElementById('left-points');
    if (state.declarer_points + state.team_points > 0) {
        if (myPile) {
            myPile.style.display = 'flex';
            if (myBadge) myBadge.textContent = state.declarer_points;
        }
        if (state.team_points >= 0) {
            if (leftBadge) {
                leftBadge.style.display = 'block';
                leftBadge.textContent = state.team_points;
            }
        }
    }
}


function renderInfo(state) {
    const analysisDiv = document.getElementById('analysis');

    // Create structure if not present
    if (!document.getElementById('analysis-header')) {
        analysisDiv.innerHTML = `
            <h3 id="analysis-header" style="margin-bottom: 5px;">Analysis</h3>
            <div id="move-history"></div>
        `;
    }

    // Update History (and integrated stats)
    renderHistory(state.move_history, state);
}

const SUIT_SYMBOL = { 'C': '♣', 'S': '♠', 'H': '♥', 'D': '♦' };

function renderHistory(history, state) {
    const container = document.getElementById('move-history');
    if (!container || !history) return;

    container.innerHTML = '';

    // 0. Start / Max Value Header
    const startDiv = document.createElement('div');
    startDiv.className = 'history-start-row';
    startDiv.innerHTML = `
        <span>Start (Max)</span>
        <span class="history-val">${state.max_possible_points}</span>
    `;
    container.appendChild(startDiv);

    history.forEach((entry, idx) => {
        // Trick Header
        if (idx % 3 === 0) {
            const trickNum = Math.floor(idx / 3) + 1;
            const header = document.createElement('div');
            header.className = 'history-trick-header';
            header.textContent = `Trick ${trickNum}`;
            container.appendChild(header);
        }

        const div = document.createElement('div');
        div.className = 'history-item';

        // Format Card
        let cardStr = entry.card;
        if (cardStr.length === 2) {
            const suit = cardStr[0];
            const rank = cardStr[1] === 'T' ? '10' : cardStr[1];
            cardStr = (SUIT_SYMBOL[suit] || suit) + rank;
        }

        // Delta Logic
        let deltaStr = '';
        let deltaClass = 'delta-neutral';

        if (entry.delta !== undefined && entry.delta !== null) {
            const d = entry.delta;
            if (d > 0) {
                deltaStr = '+' + d;
                deltaClass = 'delta-pos';
            } else if (d < 0) {
                deltaStr = '' + d;
                deltaClass = 'delta-neg';
            } else {
                deltaStr = '0';
            }
        } else {
            // Pending
            deltaStr = '';
        }

        // Value Logic
        let valStr = (entry.value_after !== undefined && entry.value_after !== null) ? entry.value_after : '';

        div.innerHTML = `
            <span class="history-card ${'suit-' + entry.card[0]}">${cardStr}</span>
            <span class="history-player">${entry.player.substring(0, 3)}</span>
            <span class="history-delta ${deltaClass}">${deltaStr}</span>
            <span class="history-val">${valStr}</span>
        `;
        container.appendChild(div);
    });

    // Result Footer
    // Only show if we have points or game is over? Always show "Current"
    const resultDiv = document.createElement('div');
    resultDiv.className = 'history-result-row';
    // User points (Declarer)
    const points = state.declarer_points;
    resultDiv.innerHTML = `
        <span>Result</span>
        <span class="history-val">${points}</span>
    `;
    container.appendChild(resultDiv);

    // Auto-scroll
    container.scrollTop = container.scrollHeight;
}

function createCardElement(shortStr, owner, simplified = false) {
    const suit = shortStr[0];
    let rank = shortStr[1];
    if (rank === 'T') rank = '10';
    const div = document.createElement('div');
    div.className = `card suit-${suit} ${simplified ? 'simplified' : ''}`;
    div.dataset.card = shortStr;
    const suitChar = SUIT_SYMBOL[suit] || '?';
    let tokenHtml = '';
    if (owner) {
        tokenHtml = `<div class="owner-token">${owner}</div>`;
    }

    if (simplified) {
        div.innerHTML = `
            <div class="simplified-center">${suitChar}${rank}</div>
            ${tokenHtml}
        `;
    } else {
        div.innerHTML = `
            <div class="card-corner"><span>${rank}</span><span>${suitChar}</span></div>
            <div class="card-center">${suitChar}</div>
            <div class="card-corner" style="transform: rotate(180deg)"><span>${rank}</span><span>${suitChar}</span></div>
            ${tokenHtml}
        `;
    }
    return div;
}

run();
