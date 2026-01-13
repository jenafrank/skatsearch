import init, { SkatGame } from './pkg/skat_aug23.js';

let game = null;
let lastTrickId = null;
let cheatMode = false;

async function run() {
    console.log("Initializing WASM...");
    await init();
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
    game = SkatGame.new_random();

    // Hide points initially
    document.getElementById('left-points').style.display = 'none';
    document.getElementById('right-points').style.display = 'none';
    document.getElementById('my-tricks-pile').style.display = 'none';
    document.getElementById('my-points').textContent = '0';
    document.getElementById('left-points').textContent = '0';
    document.getElementById('right-points').textContent = '0';

    updateUI();
    gameLoop();
}

async function gameLoop() {
    if (!game) return;

    let state = game.get_state_json();

    if (state.game_over) {
        document.getElementById('game-status').textContent = `Game Over! Winner: ${state.winner}`;
        return;
    }

    // Detect Completed Trick to Await Animation
    if (state.last_trick_cards && state.last_trick_cards !== lastTrickId) {
        lastTrickId = state.last_trick_cards;

        // Render the completed trick using ordered plays
        renderTable(state.last_trick_plays);

        await new Promise(r => setTimeout(r, 1000));

        document.getElementById('trick-cards').innerHTML = '';

        await new Promise(r => setTimeout(r, 200));

        updatePoints(state);
    }

    state = game.get_state_json();
    const currentPlayer = state.current_player;

    if (currentPlayer !== "D" && !state.game_over) {
        document.getElementById('game-status').textContent = `Status: ${currentPlayer === 'L' ? 'Left' : 'Right'} AI thinking...`;

        await new Promise(r => setTimeout(r, 800));

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

    // Skat Rendering
    renderSkat(state.skat_cards);

    // Trick Rendering
    // If trick_plays has content, render it.
    // If empty, clear (unless waiting for animation, effectively handled by loop logic)
    if (state.trick_plays && state.trick_plays.length > 0) {
        renderTable(state.trick_plays);
    } else {
        if (!state.trick_plays || state.trick_plays.length === 0) {
            document.getElementById('trick-cards').innerHTML = '';
        }
    }

    renderInfo(state);
    updatePoints(state);
}

function renderSkat(skatStr) {
    // Container for Skat? 
    // We need to inject it into HTML if not present, or assume it exists.
    // I will add <div id="skat-display"> to HTML later.
    let skatDiv = document.getElementById('skat-display');
    if (!skatDiv) return;

    if (cheatMode && skatStr) {
        skatDiv.style.display = 'flex';
        skatDiv.innerHTML = '';
        const cleanStr = skatStr.replace(/\[|\]/g, '').trim();
        const cards = cleanStr.split(/\s+/);
        cards.forEach(c => {
            const el = createCardElement(c);
            // Make skat cards smaller?
            el.style.transform = "scale(0.8)";
            el.style.margin = "0 5px";
            skatDiv.appendChild(el);
        });
        // Add label?
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

function renderOpponents(state) {
    const leftContainer = document.getElementById('hand-left');
    const rightContainer = document.getElementById('hand-right');

    if (!leftContainer || !rightContainer) return;

    if (cheatMode) {
        renderOpponentHand(leftContainer, state.left_cards, 'left');
        renderOpponentHand(rightContainer, state.right_cards, 'right');
    } else {
        renderBack(leftContainer, 'left-points');
        renderBack(rightContainer, 'right-points');
    }
}

function renderBack(container, badgeId) {
    if (!container.querySelector('.hand-back')) {
        container.innerHTML = `
            <div class="hand-back">
                <div class="points-badge" id="${badgeId}" style="display:none">0</div>
            </div>
        `;
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

function updatePoints(state) {
    const myPile = document.getElementById('my-tricks-pile');
    const myBadge = document.getElementById('my-points');
    const leftBadge = document.getElementById('left-points');
    const rightBadge = document.getElementById('right-points');

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
            if (rightBadge) {
                rightBadge.style.display = 'block';
                rightBadge.textContent = state.team_points;
            }
        }
    }
}

function renderInfo(state) {
    const analysisDiv = document.getElementById('analysis');
    analysisDiv.innerHTML = `
        <h3>Analysis</h3>
        <div>Value: ${state.current_value}</div>
        <div>Loss: ${state.last_loss}</div>
        <hr>
    `;
}

// Helpers

const SUIT_CHARS = { 'C': '♣', 'S': '♠', 'H': '♥', 'D': '♦' };

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

function renderTable(trickPlays) {
    // trickPlays is Array<{ card: string, player: string }>
    const tableDiv = document.getElementById('trick-cards');
    tableDiv.innerHTML = '';

    if (!trickPlays || trickPlays.length === 0) return;

    trickPlays.forEach((play, index) => {
        const el = createCardElement(play.card, play.player);
        // Overlap logic:
        // Use index to offset.
        // If we use flex with negative margins? or absolute?
        // Let's use negative margins for overlap simplicity, assuming container handles it.
        // We'll class them 'trick-card-item' and styles handle visuals?
        el.classList.add('trick-card-item');
        el.style.zIndex = index; // Ensure correct stack order
        if (index > 0) {
            el.style.marginLeft = "-40px"; // Overlap
            el.style.marginBottom = `${index * 5}px`; // Slight vertical fan?
        }
        tableDiv.appendChild(el);
    });
}

function createCardElement(shortStr, owner) {
    const suit = shortStr[0];
    let rank = shortStr[1];
    if (rank === 'T') rank = '10';

    const div = document.createElement('div');
    div.className = `card suit-${suit}`;

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
