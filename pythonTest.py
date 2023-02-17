import urllib.request
import unittest

URL = "http://0.0.0.0:8080"
#URL = "http://keyval.store
#URL = "http://144.24.175.153"

class IntegrationTests(unittest.TestCase):

    def test_post(self):
        dataIn = "123456789".encode('utf-8')
        print(dataIn)
        req = urllib.request.Request(URL + "/v1/PostTest/set", data=dataIn)
        resp = urllib.request.urlopen(req)
        print(resp.read())

        dataOut = urllib.request.urlopen(URL + "/v1/PostTest/get").read()
        print("data: " + str(dataOut))
        self.assertEqual(dataIn, dataOut)

    def test_set_get_url(self):
        dataIn = "123456789"
        print("dataIn: " + dataIn)
        req = urllib.request.urlopen(URL + "/v1/GetTest/set/" + dataIn)

        dataOut = urllib.request.urlopen(URL + "/v1/GetTest/get").read().decode('utf-8')
        print("dataOut: " + dataOut)
        self.assertEqual(dataIn, dataOut)


if __name__ == '__main__':
    unittest.main()

