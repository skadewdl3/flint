from locust import HttpUser, task, between

class APITestUser(HttpUser):
    wait_time = between(1, 2)  # Simulate user wait time 

    @task(1)
    def insert_bulk(self):
        self.client.post("/insert_bulk")

    @task(1)
    def insert_single(self):
        self.client.post("/insert_single")
