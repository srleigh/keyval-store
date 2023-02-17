import urllib.request
import unittest

URL = "http://0.0.0.0:8080"
#URL = "http://grugmq.com"
#URL = "http://144.24.175.153"

class IntegrationTests(unittest.TestCase):

    def test_write_post(self):
        dataIn = "123456789".encode('utf-8')
        print(dataIn)
        req = urllib.request.Request(URL + "/v1/PostTest/write", data=dataIn)
        resp = urllib.request.urlopen(req)
        print(resp.read())

        dataOut = urllib.request.urlopen(URL + "/v1/PostTest/read").read()
        print("data: " + str(dataOut))
        self.assertEqual(dataIn, dataOut)

    def test_write_get(self):
        dataIn = "123456789"
        print("dataIn: " + dataIn)
        req = urllib.request.urlopen(URL + "/v1/GetTest/write/" + dataIn)

        dataOut = urllib.request.urlopen(URL + "/v1/GetTest/read").read().decode('utf-8')
        print("dataOut: " + dataOut)
        self.assertEqual(dataIn, dataOut)


if __name__ == '__main__':
    unittest.main()

