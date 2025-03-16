locust -f locustfile.py --host http://127.0.0.1:8000 --headless --users 100 --spawn-rate 10 --run-time 20s --csv=locust_results
