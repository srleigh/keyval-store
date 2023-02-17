import os
import signal
import subprocess
import urllib.request
import time
import threading

URL = "http://grugmq.com"
#URL = "http://0.0.0.0:8080"

def git_pull():
    print("Running git pull...")
    result = subprocess.run(["git","pull"], capture_output=True, cwd="./grugmq")
    print(result.stdout)

def git_head():
    print("Running git pull...")
    result = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, cwd="./grugmq")
    return result.stdout.decode('utf-8')

def cargo_build():
    print("Running cargo build...")
    result = subprocess.run(["cargo","build","--release"], capture_output=True, cwd="./grugmq")
    print(result.stdout)

def set(msg):
    try:
        urllib.request.urlopen(URL + "/v1/deploy/write/" + msg)
    except:
        pass

def get():
    try:
        return urllib.request.urlopen(URL + "/v1/deploy/read").read().decode('utf-8')
    except:
        return ""

def printer(out, dummy):
    while True:
        line = out.readline()
        if line != b'':
            print(line)
        time.sleep(0.1)

class KillableProcess:
    ''' This class does 2 things:
          - kills a process if sigterm/sigint is sent to the script
          - prints output of process in non-blocking way '''

    kill_now = False
    p = None
    t = None

    def __init__(self):
        signal.signal(signal.SIGINT, self.exit_gracefully)
        signal.signal(signal.SIGTERM, self.exit_gracefully)

    def start(self):
        self.p = subprocess.Popen('./target/release/grugmq 80', cwd="./grugmq", shell=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, preexec_fn=os.setsid)
        os.set_blocking(self.p.stdout.fileno(), False)
        t = threading.Thread(target=printer, args=(self.p.stdout, 1))
        t.daemon = True
        t.start()

    def kill(self):
        os.killpg(os.getpgid(self.p.pid), signal.SIGTERM)

    def exit_gracefully(self, *args):
        self.kill()
        print("exited gracefully")
        self.kill_now = True


if __name__ == '__main__':
    head = git_head()
    print("git head: " + head)
    killable = KillableProcess()
    killable.start()
    while not killable.kill_now:
        if (get() == "redeploy"):
            print("Redeploying...")
            set("redeploy_in_progress")
            git_pull()
            new_head = git_head()
            if (new_head == head):
                print("No update from git.  Not re-deploying")
                set("redeploy_skipped")
                continue
            cargo_build()
            killable.kill()
            killable.start()
            head = git_head()
            print("Redeploy done.  New build: " + head)
            time.sleep(5) # Give time for server to restart
            set("redeploy_done_head_" + head)
        time.sleep(1)

