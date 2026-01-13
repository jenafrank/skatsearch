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
    // We use last_trick_cards string as ID since it changes every trick
    if (state.last_trick_cards && state.last_trick_cards !== lastTrickId) {
        lastTrickId = state.last_trick_cards;

        // Render the completed trick on the table
        renderTable(state.last_trick_cards);

        // Wait for user to see it (1s)
        await new Promise(r => setTimeout(r, 1000));

        // Clear trick (animation would go here)
        document.getElementById('trick-cards').innerHTML = '';

        await new Promise(r => setTimeout(r, 200));

        // Update Points Badges
        updatePoints(state);
    }

    // Refresh state
    state = game.get_state_json();
    const currentPlayer = state.current_player; // "D", "L", "R"

    if (currentPlayer !== "D" && !state.game_over) {
        document.getElementById('game-status').textContent = `Status: ${currentPlayer === 'L' ? 'Left' : 'Right'} AI thinking...`;

        // Delay before AI moves (Increased to 800ms)
        await new Promise(r => setTimeout(r, 800));

        const moved = game.make_ai_move();
        if (moved) {
            updateUI();
            // Loop again quickly to check for next move or trick completion
            setTimeout(gameLoop, 50);
        } else {
            // AI failed to move? Should not happen in God Mode unless game over
            console.log("AI refused to move?");
        }
    } else if (currentPlayer === "D") {
        document.getElementById('game-status').textContent = "Your Turn (Declarer)";
    }
}

function playCard(cardStr) {
    if (!game) return;

    const state = game.get_state_json();
    if (state.current_player !== "D") {
        console.log("Not your turn");
        return;
    }

    const res = game.play_card_str(cardStr);
    if (res) {
        updateUI();
        gameLoop();
    } else {
        console.log("Invalid move");
    }
}

function undoMove() {
    if (game) {
        game.undo();
        // If undoing into a previous trick state, reset ID so we don't think it's new?
        // Or just nullify it so if we replay it, it shows again?
        lastTrickId = null;
        updateUI();
        gameLoop();
    }
}

function showHint() {
    if (game) {
        const hint = game.get_hint_json();
        const el = document.getElementById('analysis');
        // Prepend hint or append?
        // Let's create a hint element if not exists or update.
        console.log("Hint:", hint);
        alert(`Bester Zug: ${hint.best_card} (Wert: ${hint.value})`);
    }
}

function updateUI() {
    if (!game) return;
    const state = game.get_state_json();

    renderHand(state.my_cards);
    renderOpponents(state);

    // If a trick is currently being animated (lastTrickId matched state), logic handles table.
    // If normal play, render state.trick_cards.
    // However, if we just finished a trick, state.trick_cards is empty.
    // We rely on gameLoop to render the last_trick_cards.
    // So if state.trick_cards is NOT empty, render it.
    if (state.trick_cards && state.trick_cards.length > 0) {
        renderTable(state.trick_cards);
    } else {
        // If empty, do NOT clear immediately if we are waiting for animation.
        // But updateUI is called after playCard.
        // If playCard finished trick, trick_cards is empty.
        // gameLoop will see last_trick_cards and render it.
        // So we can safely clear here?
        // If we clear here, the table flashes empty before gameLoop renders last trick.
        // Better: Don't clear if last_trick_cards is set and we haven't animated it yet?
        // Simplify: Just clear. The flash is 50ms.
        // Or check:
        // if (!state.trick_cards && state.last_trick_cards && state.last_trick_cards !== lastTrickId) { ... }
        if (!state.trick_cards) {
            // Don't clear if loop is about to handle it?
            // Safe to clear if we accept flash.
            document.getElementById('trick-cards').innerHTML = '';
        }
    }

    renderInfo(state);
    updatePoints(state);
}

function renderOpponents(state) {
    // Left
    const leftContainer = document.getElementById('hand-left');
    // Right
    const rightContainer = document.getElementById('hand-right');

    if (!leftContainer || !rightContainer) return; // safety

    if (cheatMode) {
        // Render open cards
        renderOpponentHand(leftContainer, state.left_cards, 'left');
        renderOpponentHand(rightContainer, state.right_cards, 'right');
    } else {
        // Render Back if not present
        // Check if back exists
        renderBack(leftContainer, 'left-points');
        renderBack(rightContainer, 'right-points');
    }
}

function renderBack(container, badgeId) {
    // If we are switching from cheat mode, we need to restore the back div
    if (!container.querySelector('.hand-back')) {
        // Note: Resetting point value visibility might be tricky. 
        // We ensure updatePoints is called frequently.
        container.innerHTML = `
            <div class="hand-back">
                <div class="points-badge" id="${badgeId}" style="display:none">0</div>
            </div>
        `;
    }
    // If it exists, ensure badge exists (might have been overwritten if we did something else)
}

function renderOpponentHand(container, cardsStr, side) {
    // We replace the container content with cards.
    // Add "opponent" class context or side?
    container.innerHTML = '';

    if (!cardsStr) return; // Should be empty string if empty
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

    // We need to find left/right points badges, which might be recreated by renderBack
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

function renderTable(cardsStr) {
    const tableDiv = document.getElementById('trick-cards');
    tableDiv.innerHTML = '';

    if (!cardsStr) return;
    const cleanStr = cardsStr.replace(/\[|\]/g, '').trim();
    if (cleanStr.length === 0) return;

    const cardList = cleanStr.split(/\s+/);

    cardList.forEach(cStr => {
        const el = createCardElement(cStr);
        tableDiv.appendChild(el);
    });
}

function createCardElement(shortStr) {
    const suit = shortStr[0];
    let rank = shortStr[1];
    if (rank === 'T') rank = '10';

    const div = document.createElement('div');
    div.className = `card suit-${suit}`;

    const suitChar = SUIT_CHARS[suit] || '?';

    div.innerHTML = `
        <div class="card-corner"><span>${rank}</span><span>${suitChar}</span></div>
        <div class="card-center">${suitChar}</div>
        <div class="card-corner" style="transform: rotate(180deg)"><span>${rank}</span><span>${suitChar}</span></div>
    `;
    return div;
}

run();
