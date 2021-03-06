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
  radio.setAutoAck(false);               // Disable Auto Acknowledgement

  // radio.openWritingPipe(TX_ADDRESS);  // Transmit address
  radio.openReadingPipe(0, RX_ADDRESS);  // Receiver address
  
  radio.startListening();                // Set module as receiver

  // radio.printDetails();                  // Debug configuration
  radio.flush_rx();
}

void loop() {
  if (radio.available()) {

    byte rx_message_data[radio.getDynamicPayloadSize()]; 
    radio.read(&rx_message_data, radio.getDynamicPayloadSize());

    Serial.write(rx_message_data, sizeof(rx_message_data));
  }
}
