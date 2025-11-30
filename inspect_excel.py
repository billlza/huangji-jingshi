import pandas as pd
import json
import os

file_path = '/Users/bill/Desktop/hjjs/皇极经世推步全.2025.20250411212141796.xlsx'

def inspect_excel(path):
    if not os.path.exists(path):
        print(f"File not found: {path}")
        return

    try:
        xl = pd.ExcelFile(path)
        print(f"Sheet names: {xl.sheet_names}")
        
        for sheet in xl.sheet_names:
            print(f"\n--- Inspecting Sheet: {sheet} ---")
            df = pd.read_excel(path, sheet_name=sheet, nrows=5)
            print("Columns:", df.columns.tolist())
            print("First 5 rows:")
            print(df.head())
            
    except Exception as e:
        print(f"Error reading excel: {e}")

if __name__ == "__main__":
    inspect_excel(file_path)
