import urllib.request
import threading
import time

#URL = "http://0.0.0.0:8080"
#URL = "http://grugmq.com"
URL = "http://144.24.175.153"

OneKB = "12345678" * 128
urllib.request.urlopen(URL + "/v1/benchmark/write/" + OneKB)

NUM_THREADS=500
READS_PER_THREAD=100

def threadedReader(threadNum):
    count = 0
    for i in range(0, READS_PER_THREAD):
        data = urllib.request.urlopen(URL + "/v1/benchmark/read", timeout=10.0).read()
        count+=1
        if count%10 == 0:
            print("threadNum: " + str(threadNum) + " count: " + str(count) + " data: " + str(len(data)))


threads = []
for i in range(0,NUM_THREADS):
    t = threading.Thread(target=threadedReader, args=(i,))
    threads.append(t)

t0 = time.time()
for t in threads:
    t.start()
for t in threads:
    t.join()
t1 = time.time()

print("Total time: " + str(t1-t0))
readsPerSec = (NUM_THREADS * READS_PER_THREAD) / (t1-t0)
print("Reads per sec: " + str(readsPerSec))

