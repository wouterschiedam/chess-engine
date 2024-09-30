import pandas as pd

# Load the data from CSV (assuming your data is in a file named 'chess_puzzles.csv')
df = pd.read_csv('/home/wouter/Downloads/lichess_puzzle_transformed.csv')

# Sort the dataframe by the 'Rating' column in descending order
df_sorted = df.sort_values(by='Rating', ascending=True)

# Save the sorted data to a new CSV file
df_sorted.to_csv('/home/wouter/Downloads/sorted_puzzles.csv', index=False)

# Print the first few rows of the sorted data
print(df_sorted.head())

