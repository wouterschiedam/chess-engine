# import chess
#
# def is_player_in_check(fen: str) -> bool:
#     # Create a board from the FEN string
#     board = chess.Board(fen)
#     # Return if the current player is in check
#     return board.is_check()
#
# def read_fen_strings_from_file(file_path: str) -> list:
#     fens = []
#     with open(file_path, 'r') as file:
#         for line in file:
#             if line.startswith("fen:"):
#                 fen = line.split("fen:")[1].strip()
#                 fens.append(fen)
#     return fens
#
# # Path to the file containing the FEN strings
# file_path = 'debug.txt'
#
# # Read the FEN strings from the file
# fen_strings = read_fen_strings_from_file(file_path)
#
# # Check if the player is in check for each FEN string and print the results
# for fen in fen_strings:
#     check_status = is_player_in_check(fen)
#     print(f"FEN: {fen}")
#     print(f"Is player in check? {check_status}\n")

import chess
import chess.engine

# Initialize the chess board with a specific starting position (FEN)
start_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"  # Standard starting position
# Or use another FEN string for a different position
# start_fen = "rnbqkb1r/pppppppp/5n2/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1"  # Example position
board = chess.Board(start_fen)

# Path to your Stockfish executable
stockfish_path = "/opt/homebrew/bin/stockfish"  # Change this to the correct path

# Initialize the Stockfish engine
with chess.engine.SimpleEngine.popen_uci(stockfish_path) as engine:
    # Dictionary to hold the perft results
    perft_results = {}

    def perft(board, depth):
        if depth == 0:
            return 1
        nodes = 0
        for move in board.legal_moves:
            board.push(move)
            nodes += perft(board, depth - 1)
            board.pop()
        return nodes

    # Generate moves and perform perft for depth 3
    for move in board.legal_moves:
        board.push(move)
        perft_results[move.uci()] = perft(board, 2)  # Depth 2 from here means depth 3 overall
        board.pop()

    # Print the perft results for each move at depth 3
    for move, count in perft_results.items():
        print(f"Move: {move}, Count: {count}")

# Engine quit is handled by the context manager
