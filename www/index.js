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
    // Initial status
    document.getElementById('game-status').textContent = "Waiting for game...";
    bindEvents();
    startNewGame();
}

// Game Selection State
let selectedSkatCards = new Set();
let selectionPhase = false;

function bindEvents() {
    document.getElementById('btn-new-game').onclick = startNewGame;
    document.getElementById('btn-hint').onclick = showHint;
    document.getElementById('btn-undo').onclick = undoMove;
    document.getElementById('btn-cheat').onclick = toggleCheatMode;

    // Selection Modal events
    document.getElementById('btn-calc-best').onclick = calculateBestGame;
    document.getElementById('btn-start-game').onclick = finishSelection;

    // Radio Button Listeners for Dynamic Sorting
    const radios = document.getElementsByName('game-type');
    console.log("Found radios:", radios.length);
    radios.forEach(r => {
        r.addEventListener('change', (e) => {
            console.log("Radio changed:", e.target.value);
            // Visual Debug
            const statusbar = document.getElementById('game-status');
            if (statusbar) statusbar.textContent = "Switching to " + e.target.value + "...";

            if (game && e.target.checked) {
                game.update_game_type(e.target.value);
                console.log("Updated game type in WASM");
                renderSelectionUI();
                if (statusbar) statusbar.textContent = "Mode: " + e.target.value;
            }
        });
    });
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
    selectedSkatCards.clear();
    selectionPhase = true;

    // Create new game (initialized in selection phase)
    game = SkatGame.new_random();

    document.getElementById('left-points').style.display = 'none';
    document.getElementById('my-tricks-pile').style.display = 'none';
    document.getElementById('my-points').textContent = '0';
    document.getElementById('left-points').textContent = '0';
    document.getElementById('game-over-overlay').style.display = 'none';

    // Show Selection Modal
    const modal = document.getElementById('game-selection-modal');
    modal.style.display = 'flex';
    document.getElementById('best-game-results').style.display = 'none';
    document.getElementById('best-game-table').querySelector('tbody').innerHTML = '';

    renderSelectionUI();
}

function renderSelectionUI() {
    if (!game) return;
    const state = game.get_state_json();
    console.log("WASM State Debug Info:", state.debug_info);
    console.log("WASM My Cards:", state.my_cards);
    const handContainer = document.getElementById('selection-hand-container');
    const skatContainer = document.getElementById('selection-skat-container');

    handContainer.innerHTML = '';
    skatContainer.innerHTML = '';

    if (!state.my_cards) {
        handContainer.innerHTML = '<div style="color:red">Error: No cards returned from WASM</div>';
        return;
    }

    const cleanStr = state.my_cards.replace(/\[|\]/g, '').trim();
    if (!cleanStr) return;

    // Use Set to track all cards to ensure we don't duplicate or lose them
    // Original cards from WASM (all 12)
    const allCards = cleanStr.split(/\s+/);

    // Check if we need to initialize selectedSkatCards (if first load)
    // Actually selectedSkatCards is global state managed by us

    // Iterate all cards. If in skat set, render in Skat. Else in Hand.

    // Sort cards slightly? 
    // They come from WASM sorted usually. Skat selection preserves them in Set.
    // We should keep hand order stable if possible.

    allCards.forEach(cStr => {
        if (selectedSkatCards.has(cStr)) {
            // Render in Skat
            const el = createCardElement(cStr);
            el.onclick = () => toggleSkatSelection(cStr); // Click to remove
            skatContainer.appendChild(el);
        } else {
            // Render in Hand
            const el = createCardElement(cStr);
            el.onclick = () => toggleSkatSelection(cStr); // Click to add
            handContainer.appendChild(el);
        }
    });

    if (skatContainer.children.length === 0) {
        skatContainer.innerHTML = '<span style="color:#aaa; font-style:italic;">Empty Skat</span>';
    }

    updateSelectionControls();
}

function toggleSkatSelection(cardStr) {
    if (selectedSkatCards.has(cardStr)) {
        selectedSkatCards.delete(cardStr);
    } else {
        if (selectedSkatCards.size >= 2) {
            // Optional: Shake animation or alert?
            return;
        }
        selectedSkatCards.add(cardStr);
    }
    renderSelectionUI();
}

function updateSelectionControls() {
    const arr = Array.from(selectedSkatCards);
    // Skat display text removed from UI, so we only update button
    const btnStart = document.getElementById('btn-start-game');
    if (btnStart) {
        if (arr.length === 2) {
            btnStart.disabled = false;
            btnStart.textContent = "Start Game";
        } else {
            btnStart.disabled = true;
            btnStart.textContent = `Select ${2 - arr.length} more`;
        }
    }
}

async function calculateBestGame() {
    if (!game) return;
    const btn = document.getElementById('btn-calc-best');
    const originalText = btn.textContent;
    btn.textContent = "Calculating...";
    btn.disabled = true;

    // Yield to UI
    await new Promise(r => setTimeout(r, 10));

    try {
        const results = game.calculate_best_game_perfect_info();
        renderBestGameResults(results);
    } catch (e) {
        console.error("Best Game Calc Failed", e);
        alert("Calculation Failed");
    } finally {
        btn.textContent = originalText;
        btn.disabled = false;
    }
}

function renderBestGameResults(results) {
    const tableBody = document.getElementById('best-game-table').querySelector('tbody');
    tableBody.innerHTML = '';
    document.getElementById('best-game-results').style.display = 'block';

    if (!results || results.length === 0) {
        tableBody.innerHTML = '<tr><td colspan="5">No results found</td></tr>';
        return;
    }

    results.forEach(res => {
        const row = document.createElement('tr');
        const isWin = res.win_rate > 0.5;
        const winClass = isWin ? 'win-yes' : 'win-no';
        const winText = isWin ? 'WIN' : 'LOSS';

        let skatStr = "";
        if (res.skat && res.skat.length > 0) {
            // Clean skat strings too just in case
            skatStr = res.skat.map(s => s.replace(/\[|\]/g, '')).join(", ");
        }

        // Action Button
        const applyBtn = document.createElement('button');
        applyBtn.textContent = "Select";
        applyBtn.style.padding = "4px 10px";
        applyBtn.style.fontSize = "12px";
        applyBtn.style.cursor = "pointer";
        applyBtn.onclick = () => applyBestGameSelection(res);

        row.innerHTML = `
            <td style="font-weight:bold;">${res.game}</td>
            <td class="${winClass}" style="font-weight:bold;">${winText}</td>
            <td>${res.value}</td>
            <td style="font-family:monospace;">${skatStr}</td>
        `;
        const tdAction = document.createElement('td');
        tdAction.appendChild(applyBtn);
        row.appendChild(tdAction);

        tableBody.appendChild(row);
    });
}

function applyBestGameSelection(res) {
    // 1. Set Skat Cards (Update first so render logic sees them)
    selectedSkatCards.clear();
    if (res.skat) {
        // Skat strings might need trimming
        res.skat.forEach(c => selectedSkatCards.add(c.replace(/\[|\]/g, '').trim()));
    }

    // 2. Set Game Type (Radio) and Trigger Event
    const g = res.game; // "Null", "Grand", "Clubs", "Spades", ...

    const radios = document.getElementsByName('game-type');
    for (const r of radios) {
        if (r.value === g) {
            r.checked = true;
            // Dispatch event to ensure WASM updates and UI resorts
            r.dispatchEvent(new Event('change'));
            break;
        }
    }
}

function finishSelection() {
    if (selectedSkatCards.size !== 2) return;

    // Get Radio Value
    let gameType = "Suit"; // Default
    const radios = document.getElementsByName('game-type');
    for (const r of radios) {
        if (r.checked) {
            gameType = r.value;
            break;
        }
    }

    const skatArr = Array.from(selectedSkatCards);
    const skatStr = skatArr.join(" ");

    const success = game.finalize_game_selection(gameType, skatStr);
    if (success) {
        document.getElementById('game-selection-modal').style.display = 'none';
        selectionPhase = false;

        // Setup UI for playing
        gameLoop();

        // Trigger generic analysis
        setTimeout(() => triggerAnalysis(), 500);
    } else {
        alert("Failed to start game. Check selection.");
    }
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
        // Status updated in updateUI mostly, but here for turn notification
        // We use the debug info from state if available
        // document.getElementById('game-status').textContent = "Your Turn"; 
    }
}

// ... existing triggerAnalysis ...

// Update UI:
// We need to inject the status update in updateUI to ensure it persists


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

    // Status Bar Update
    let statusText = state.debug_info || "Game Running";
    if (state.game_over) {
        statusText += ` | Game Over (${state.winner})`;
    } else {
        const turn = state.current_player === "D" ? "Your Turn" : (state.current_player === "L" ? "Left Thinking" : "Right Thinking");
        statusText += ` | ${turn}`;
    }
    document.getElementById('game-status').textContent = statusText;

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
