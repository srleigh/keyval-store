import urllib.request as req
data_in = "123"
my_channel = "http://keyval.store/v1/my_channel/"  # api version 1 and channel 'my_channel'
req.urlopen(my_channel + "set/" + data_in)  # set
data_out = req.urlopen(my_channel + "get").read().decode('utf-8')  # get
assert(data_in == data_out)
