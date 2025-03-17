locust -f locustfile.py --host http://127.0.0.1:8000 --headless --users 100 --spawn-rate 10 --run-time 20s --csv=locust_results
# remove --headless if want to run on web GIU

# use below vanilla cmd to run web ui without config. increase the --run-time ideally to 1 minute atleast
#locust -f locustfile.py --host http://127.0.0.1:8000