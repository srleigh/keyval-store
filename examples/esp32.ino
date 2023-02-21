#include <WiFi.h>
#include <HTTPClient.h>

void setup() {
  Serial.begin(9600);

  // Connect wifi
  const char* ssid = "FRITZ!Box 6591 Cable FK";
  const char* password = "CorrectHorseBatteryStaple";
  WiFi.begin(ssid, password);
  Serial.println("\nConnecting");
  while (WiFi.status() != WL_CONNECTED) {
    Serial.print(".");
    delay(100);
  }
  Serial.println("\nConnected to the WiFi network");

  const String url = "http://keyval.store/v1/arduinoexample/";

  // Set val
  HTTPClient http;
  String data_in = "123abc";
  http.begin(url + "set/" + data_in);
  http.GET();

  // Get val
  http.begin(url + "get");
  http.GET();
  String data_out = http.getString();
  Serial.println(data_in + ":" + data_out);
}

void loop() {}
