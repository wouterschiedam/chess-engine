import chess
import chess.engine
from tqdm import tqdm

# Path to the Stockfish executable
STOCKFISH_PATH = "/opt/homebrew/Cellar/stockfish/16/bin/stockfish"

# Function to read games from output.txt
def read_games(file_path):
    games = []
    with open(file_path, 'r') as file:
        lines = file.readlines()
        for i in range(0, len(lines), 2):
            name = lines[i].strip()
            fen = lines[i+1].strip()
            games.append((name, fen))
    return games

# Function to evaluate the game using Stockfish
def evaluate_game(fen):
    with chess.engine.SimpleEngine.popen_uci(STOCKFISH_PATH) as engine:
        board = chess.Board(fen)
        try:
            result = engine.analyse(board, chess.engine.Limit(time=0.1))
            return result['score'].relative.score()
        except:
            return None

# Read games from the file
games = read_games('output.txt')

# Define a threshold for "approximately equal" evaluation
threshold = 100  # Example threshold in centipawns

# Filter games with evaluations within the threshold
filtered_games = []

# Add a progress bar
for name, fen in tqdm(games, desc="Evaluating games"):
    score = evaluate_game(fen)
    if score is not None and abs(score) <= threshold:
        filtered_games.append((name, fen, score))

# Write filtered games to a new file
with open('equal_games.txt', 'w') as file:
    for name, fen, score in filtered_games:
        file.write(f"{name}\n{fen}\nEvaluation: {score/100:.2f} centipawns\n\n")

print("Filtered games have been written to equal_games.txt")
