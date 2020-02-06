// Include libraries
#include <SPI.h>
#include <nRF24L01.h>
#include <RF24.h>
#include <printf.h>

#define CE 7
#define CSN 8

const uint8_t TX_ADDRESS[] = {0x22,0x22,0x22,0x22,0x22};
const uint8_t RX_ADDRESS[] = {0x11,0x11,0x11,0x11,0x11};

RF24 radio(CE, CSN);

void setup() {
  Serial.begin(9600, SERIAL_8N1);
  printf_begin();

  SPI.begin();

  radio.begin();

  radio.setDataRate(RF24_250KBPS);       // Send at 250 kilobits per second
  radio.setChannel(100);                 // Set the channel to 100, range is 0-125
  radio.setPALevel(RF24_PA_MIN);         // Set the power level to max (0 dB)
  radio.setCRCLength(RF24_CRC_16);       // 16 bit Cyclic Redundancy Check
  radio.enableDynamicPayloads();
  radio.setAutoAck(false);               // Disable auto acknowledgements

  // radio.openWritingPipe(TX_ADDRESS);  // Transmit address
  radio.openReadingPipe(0, RX_ADDRESS);  // Receiver address
  
  radio.startListening();                // Set module as receiver

  // radio.printDetails();                  // Debug configuration
  radio.flush_rx();
}

uint8_t prev = 0;

// Convert bytes to Big Endian.
byte *pack_long_be(byte pack[4], long value) {
  pack[0] = (value >> 24) & 0xFF;
  pack[1] = (value >> 16) & 0xFF;
  pack[2] = (value >> 8) & 0xFF;
  pack[3] = value & 0xFF;
  
  return pack;
}

void loop() {
  if (radio.available()) {

    byte rx_message_data[radio.getDynamicPayloadSize()]; 
    radio.read(&rx_message_data, radio.getDynamicPayloadSize());

    for (int i = 0; i < sizeof(rx_message_data); i++) { 
      Serial.write(rx_message_data[i]);
    } 

    unsigned long packet_length = sizeof(rx_message_data);

    byte rx_message_len[4];
    pack_long_be(rx_message_len, 16);

    byte packet[sizeof(rx_message_len) + sizeof(rx_message_data)];

    memcpy(packet, rx_message_len, sizeof(rx_message_len));
    memcpy(packet + sizeof(rx_message_len), rx_message_data, sizeof(rx_message_data));

    Serial.flush();
  }
}
