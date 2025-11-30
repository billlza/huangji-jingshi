import pandas as pd
import json
import numpy as np
import os

file_path = '/Users/bill/Desktop/hjjs/皇极经世推步全.2025.20250411212141796.xlsx'
output_path = '/Users/bill/Desktop/hjjs/huangji-jingshi-web/huangji_core/data/year_mapping.json'

def parse_excel():
    # Read without header to access by index
    df = pd.read_excel(file_path, header=None)
    
    # Columns based on deep_inspect:
    # 1: 元
    # 2: 会
    # 5: 运 (Mixed with name)
    # 6: 世 (Hexagram?)
    # 7: 旬 (Hexagram?)
    # 8: 年卦 (Hexagram)
    # 9: 干支
    # 10: 公元年
    # 11: Dynasty/Event
    # 12: Person
    
    # We need to fill down the hierarchy columns
    # Note: col 5 sometimes contains "11运", sometimes "雷风恒（1384-1743）". 
    # We might need to handle this, but for now simple ffill is a good start.
    
    hierarchy_cols = [1, 2, 5, 6, 7]
    df[hierarchy_cols] = df[hierarchy_cols].ffill()
    
    data = []
    
    for i, row in df.iterrows():
        try:
            # Check if col 10 is a valid year
            year_val = row[10]
            if pd.isna(year_val):
                continue
                
            try:
                year = int(year_val)
            except ValueError:
                continue
                
            # Valid record
            record = {
                "gregorian_year": year,
                "ganzhi": str(row[9]) if pd.notna(row[9]) else "",
                "nian_hexagram": str(row[8]) if pd.notna(row[8]) else "",
                "dynasty": str(row[11]) if pd.notna(row[11]) else "",
                "person": str(row[12]) if pd.notna(row[12]) else "",
                # Hierarchy info (raw for now)
                "yuan_raw": str(row[1]) if pd.notna(row[1]) else "",
                "hui_raw": str(row[2]) if pd.notna(row[2]) else "",
                "yun_raw": str(row[5]) if pd.notna(row[5]) else "",
                "shi_raw": str(row[6]) if pd.notna(row[6]) else "",
                "xun_raw": str(row[7]) if pd.notna(row[7]) else "",
            }
            
            # Basic cleaning for hierarchy
            # Extract "12" from "12运" if possible, or just keep as string
            
            data.append(record)
            
        except Exception as e:
            # print(f"Error at row {i}: {e}")
            continue
            
    print(f"Extracted {len(data)} records.")
    
    if len(data) > 0:
        os.makedirs(os.path.dirname(output_path), exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
        print(f"Saved to {output_path}")

if __name__ == "__main__":
    parse_excel()
