import re

with open('points_playout_log.txt', encoding='utf-8') as f:
    content = f.read()

games = re.findall(
    r'GAME\s+(\d+)/100.*?(WIN|LOSS).*?Perfect:\s*(\d+)\s*pts.*?PIMC Loss:\s*(\d+)\s*pts\s*\(D:(\d+)\s*O:(\d+)\)',
    content
)

print(f'Total game headers parsed: {len(games)}\n')

print('=== LOSS games where PIMC might have outscored perfect play ===')
print('  (est. PIMC score = perfect + opponent_loss - declarer_loss)\n')
interesting = []
for g in games:
    num    = int(g[0])
    tag    = g[1]
    perfect = int(g[2])
    d_loss  = int(g[4])
    o_loss  = int(g[5])
    pimc_est = perfect + o_loss - d_loss
    if tag == 'LOSS' and pimc_est >= 61:
        interesting.append((num, perfect, pimc_est, d_loss, o_loss))

if interesting:
    print(f"  {'Game':>6}  {'Perfect':>8}  {'PIMC est':>10}  {'D-Loss':>7}  {'O-Loss':>7}")
    print('  ' + '-'*50)
    for num, perfect, pimc_est, d_loss, o_loss in interesting:
        print(f'  Game {num:3d}:  perfect={perfect:3d}  PIMC_est={pimc_est:3d}  D-loss={d_loss:3d}  O-loss={o_loss:3d}  *** POTENTIALLY WON ***')
else:
    print('  Keine gefunden.')

print()
print('=== Alle LOSS games (sortiert nach perfect result) ===')
print(f"  {'Game':>6}  {'Perfect':>8}  {'PIMC est':>10}  {'D-Loss':>7}  {'O-Loss':>7}")
print('  ' + '-'*55)
loss_list = []
for g in games:
    num     = int(g[0])
    tag     = g[1]
    perfect = int(g[2])
    d_loss  = int(g[4])
    o_loss  = int(g[5])
    if tag == 'LOSS':
        pimc_est = perfect + o_loss - d_loss
        loss_list.append((num, perfect, pimc_est, d_loss, o_loss))

for num, perfect, pimc_est, d_loss, o_loss in sorted(loss_list, key=lambda x: -x[1]):
    flag = '  <-- close!' if pimc_est >= 50 else ''
    print(f'  Game {num:3d}:  perfect={perfect:3d}  PIMC_est={pimc_est:3d}  D-loss={d_loss:3d}  O-loss={o_loss:3d}{flag}')

print(f'\n  Total LOSS games: {len(loss_list)}/{len(games)}')
