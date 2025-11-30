import pandas as pd

file_path = '/Users/bill/Desktop/hjjs/皇极经世推步全.2025.20250411212141796.xlsx'

def deep_inspect():
    # Read without header
    df = pd.read_excel(file_path, header=None)
    
    print("Shape:", df.shape)
    print("\n--- Rows 0-9 ---")
    print(df.iloc[0:10].to_string())
    
    print("\n--- Rows 40-60 (Where 'Skipping' happened) ---")
    print(df.iloc[40:60].to_string())

    # Try to find where "公元年" or actual years appear
    print("\n--- Search for numeric years ---")
    # Check column 8 (index 8, which was '年' in previous attempt)
    col8 = df.iloc[:, 8]
    print("Column 8 sample:")
    print(col8.head(20).to_string())
    
    # Find first row in col 8 that is a number > 1000 or < -1000
    for idx, val in col8.items():
        try:
            v = float(val)
            if abs(v) > 1000:
                print(f"Found potential year at row {idx}: {v}")
                # Print surrounding row
                print(df.iloc[idx].to_string())
                break
        except:
            pass

if __name__ == "__main__":
    deep_inspect()
