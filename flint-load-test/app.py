from fastapi import FastAPI
import time

app = FastAPI()

@app.post("/insert_bulk")
def insert_bulk():
    time.sleep(0.5)  # Simulate processing
    return {"status": "Bulk insert completed"}

@app.post("/insert_single")
def insert_single():
    for _ in range(5):
        time.sleep(0.2)
    return {"status": "Single insert completed"}
