import subprocess
import time
import sys
import tempfile
import os
import jenkins
import requests
from requests.auth import HTTPBasicAuth


JENKINS_URL = "http://localhost:8080"
JENKINS_USER = "admin"       
JENKINS_API_TOKEN = "f630cd07f7514838af3e61f771403785"
JOB_NAME = "arc_job"
CUSTOM_IMAGE = "jenkins-dotnet:latest"

DOCKERFILE_CONTENT = r"""
FROM jenkins/jenkins:lts
USER root
RUN apt-get update && apt-get install -y wget apt-transport-https \
    && wget https://packages.microsoft.com/config/ubuntu/20.04/packages-microsoft-prod.deb \
    && dpkg -i packages-microsoft-prod.deb \
    && apt-get update && apt-get install -y dotnet-sdk-6.0 \
    && ln -s /usr/share/dotnet/dotnet /usr/bin/dotnet \
    && rm packages-microsoft-prod.deb
USER jenkins
"""

def image_exists(image_name):
    try:
        subprocess.run(["docker", "image", "inspect", image_name], check=True)
        return True
    except:
        return False  # If it fails, assume it doesn't exist

def build_custom_image():
    print("Building Docker image... this might take a while...")
    tmpdir = tempfile.mkdtemp()
    dockerfile_path = os.path.join(tmpdir, "Dockerfile")
    with open(dockerfile_path, "w") as f:
        f.write(DOCKERFILE_CONTENT)
    
    cmd = ["docker", "build", "-t", CUSTOM_IMAGE, tmpdir]
    subprocess.run(cmd)
    
def check_jenkins_container():
    try:
        output = subprocess.check_output(["docker", "ps", "-a", "--filter", "name=jenkins", "--format", "{{.Names}}"])
        return "jenkins" in output.decode()
    except:
        return False

def start_jenkins_container():
    print("Starting Jenkins container...")
    subprocess.run([
        "docker", "run", "-d", "--name", "jenkins",
        "-p", "8080:8080", "-p", "50000:50000", CUSTOM_IMAGE
    ])

def ensure_jenkins_running():
    if not image_exists(CUSTOM_IMAGE):
        build_custom_image()
    
    if check_jenkins_container():
        try:
            output = subprocess.check_output(["docker", "inspect", "-f", "{{.State.Running}}", "jenkins"])
            if output.decode().strip() != "true":
                print("Jenkins exists but is stopped. Starting...")
                subprocess.run(["docker", "start", "jenkins"])
        except:
            print("Something went wrong checking container status, ignoring.")
    else:
        start_jenkins_container()

def wait_for_jenkins():
    print("Waiting for Jenkins to be available...")
    while True:
        try:
            r = requests.get(JENKINS_URL, timeout=5, auth=HTTPBasicAuth(JENKINS_USER, JENKINS_API_TOKEN))
            if r.status_code in [200, 302, 403]:
                print("Jenkins is up!")
                break
        except:
            print("Still waiting...")
        time.sleep(5)

def create_or_update_job(server, repo_url):
    config_xml = f"""<?xml version='1.1' encoding='UTF-8'?>
<project>
  <description>Job for {repo_url}</description>
  <scm class="hudson.plugins.git.GitSCM">
    <userRemoteConfigs>
      <hudson.plugins.git.UserRemoteConfig>
        <url>{repo_url}</url>
      </hudson.plugins.git.UserRemoteConfig>
    </userRemoteConfigs>
    <branches>
      <hudson.plugins.git.BranchSpec>
        <name>*/master</name>
      </hudson.plugins.git.BranchSpec>
    </branches>
  </scm>
  <builders>
    <hudson.tasks.Shell>
      <command>dotnet test arc.sln</command>
    </hudson.tasks.Shell>
  </builders>
</project>
"""
    try:
        server.get_job_config(JOB_NAME)
        print("Job exists, updating it...")
        server.reconfig_job(JOB_NAME, config_xml)
    except:
        print("Creating new job...")
        server.create_job(JOB_NAME, config_xml)

def wait_for_build(server, build_number):
    print("Waiting for build to finish...")
    while True:
        try:
            build_info = server.get_build_info(JOB_NAME, build_number)
            if not build_info["building"]:
                break
        except:
            pass
        time.sleep(5)
    print("Build done!")

def main():
    repo_url = "https://github.com/arya2004/arc"
    
    ensure_jenkins_running()
    wait_for_jenkins()

    try:
        server = jenkins.Jenkins(JENKINS_URL, username=JENKINS_USER, password=JENKINS_API_TOKEN)
        print("Connected to Jenkins as", server.get_whoami()["fullName"])
    except:
        print("Couldn't connect to Jenkins, quitting.")
        sys.exit(1)

    create_or_update_job(server, repo_url)

    print("Starting build...")
    server.build_job(JOB_NAME)
    time.sleep(10)

    job_info = server.get_job_info(JOB_NAME)
    if "lastBuild" in job_info and job_info["lastBuild"]:
        build_number = job_info["lastBuild"]["number"]
        wait_for_build(server, build_number)

        log_output = server.get_build_console_output(JOB_NAME, build_number)
        with open("output.txt", "w") as f:
            f.write(log_output)
        print("Log saved in output.txt")
    else:
        print("No build info found, weird.")

if __name__ == "__main__":
    main()
