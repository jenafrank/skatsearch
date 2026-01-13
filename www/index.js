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
    btn.textContent = cheatMode ? "Hide Cards" : "Cheat (Show Cards)";
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
        game.undo();
        lastTrickId = null;
        lastTrickSize = 0;
        document.getElementById('game-over-overlay').style.display = 'none';
        updateUI();
        gameLoop();
    }
}

function showHint() {
    if (game) {
        const hint = game.get_hint_json();
        alert(`Bester Zug: ${hint.best_card} (Wert: ${hint.value})`);
    }
}

function updateUI() {
    if (!game) return;
    const state = game.get_state_json();

    renderHand(state.my_cards);
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
        const el = createCardElement(cStr);
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

function renderHand(cards) {
    const handDiv = document.getElementById('my-hand');
    handDiv.innerHTML = '';
    if (!cards) return;
    const cleanStr = cards.replace(/\[|\]/g, '').trim();
    if (cleanStr.length === 0) return;
    const cardList = cleanStr.split(/\s+/);
    cardList.forEach(cStr => {
        const el = createCardElement(cStr);
        el.onclick = () => playCard(cStr);
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
    analysisDiv.innerHTML = `
        <h3>Analysis</h3>
        <div>Max: ${state.max_possible_points}</div>
        <div>Value: ${state.current_value}</div>
        <div>Loss: ${state.last_loss}</div>
        <hr>
    `;
}

const SUIT_CHARS = { 'C': '♣', 'S': '♠', 'H': '♥', 'D': '♦' };

function createCardElement(shortStr, owner) {
    const suit = shortStr[0];
    let rank = shortStr[1];
    if (rank === 'T') rank = '10';
    const div = document.createElement('div');
    div.className = `card suit-${suit}`;
    div.dataset.card = shortStr;
    const suitChar = SUIT_CHARS[suit] || '?';
    let tokenHtml = '';
    if (owner) {
        tokenHtml = `<div class="owner-token">${owner}</div>`;
    }
    div.innerHTML = `
        <div class="card-corner"><span>${rank}</span><span>${suitChar}</span></div>
        <div class="card-center">${suitChar}</div>
        <div class="card-corner" style="transform: rotate(180deg)"><span>${rank}</span><span>${suitChar}</span></div>
        ${tokenHtml}
    `;
    return div;
}

run();
