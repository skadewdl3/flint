import pandas as pd
import matplotlib.pyplot as plt
import os


available_files = os.listdir() 


stats_history_file = "locust_results_stats_history.csv"
stats_file = "locust_results_stats.csv"
failures_file = "locust_results_failures.csv"
exceptions_file = "locust_results_exceptions.csv"


def load_csv_if_exists(file_name):
    if file_name in available_files:
        try:
            return pd.read_csv(file_name)
        except Exception as e:
            print(f"Error loading {file_name}: {e}")
            return None
    else:
        print(f"File not found: {file_name}")
        return None


df_history = load_csv_if_exists(stats_history_file)
if df_history is not None:
    try:
        if "Timestamp" in df_history.columns:
            df_history["Timestamp"] = pd.to_datetime(df_history["Timestamp"], unit="s")
            
           
            percentiles = ["50%", "90%", "99%"]
            for p in percentiles:
                if p not in df_history.columns:
                    print(f"Warning: Missing column '{p}' in {stats_history_file}")
                    percentiles.remove(p) 
            
            plt.figure(figsize=(10, 5))
            for p in percentiles:
                plt.plot(df_history["Timestamp"], df_history[p], label=f"{p} Response Time (ms)")
            
            plt.xlabel("Time")
            plt.ylabel("Response Time (ms)")
            plt.title("Locust Stress Test: Response Time vs Time")
            plt.legend()
            plt.grid()
            plt.savefig("locust_response_time_vs_time.png")
            plt.show()
        else:
            print(f"Warning: 'Timestamp' column missing in {stats_history_file}")
    except Exception as e:
        print(f"Error processing {stats_history_file}: {e}")

df_stats = load_csv_if_exists(stats_file)
if df_stats is not None:
    try:
        required_columns = ["Name", "Request Count", "Failure Count"]
        if all(col in df_stats.columns for col in required_columns):
            plt.figure(figsize=(10, 5))
            plt.bar(df_stats["Name"], df_stats["Request Count"], label="Request Count")
            plt.bar(df_stats["Name"], df_stats["Failure Count"], label="Failure Count", alpha=0.7, color="red")
            plt.xlabel("API Endpoint")
            plt.ylabel("Count")
            plt.title("Locust Test: Request Count vs. Failures per Endpoint")
            plt.legend()
            plt.xticks(rotation=45, ha="right")
            plt.savefig("locust_requests_vs_failures.png")
            plt.show()
        else:
            print(f"Warning: Missing required columns in {stats_file}")
    except Exception as e:
        print(f"Error processing {stats_file}: {e}")


df_failures = load_csv_if_exists(failures_file)
if df_failures is not None:
    try:
        if "Name" in df_failures.columns and "Occurrences" in df_failures.columns:
            plt.figure(figsize=(10, 5))
            plt.bar(df_failures["Name"], df_failures["Occurrences"], color="red", alpha=0.7)
            plt.xlabel("API Endpoint")
            plt.ylabel("Failure Occurrences")
            plt.title("Locust Test: Failure Occurrences per Endpoint")
            plt.xticks(rotation=45, ha="right")
            plt.savefig("locust_failures.png")
            plt.show()
        else:
            print(f"Warning: Missing 'Name' or 'Occurrences' column in {failures_file}")
    except Exception as e:
        print(f"Error processing {failures_file}: {e}")

df_exceptions = load_csv_if_exists(exceptions_file)
if df_exceptions is not None:
    try:
        if "Message" in df_exceptions.columns and "Count" in df_exceptions.columns:
            plt.figure(figsize=(10, 5))
            plt.bar(df_exceptions["Message"], df_exceptions["Count"], color="purple", alpha=0.7)
            plt.xlabel("Exception Message")
            plt.ylabel("Count")
            plt.title("Locust Test: Exception Occurrences")
            plt.xticks(rotation=45, ha="right")
            plt.savefig("locust_exceptions.png")
            plt.show()
        else:
            print(f"Warning: Missing 'Message' or 'Count' column in {exceptions_file}")
    except Exception as e:
        print(f"Error processing {exceptions_file}: {e}")


print("done")
