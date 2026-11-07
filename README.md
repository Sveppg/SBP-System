# SBP-System
Sound Based Prediction System for Maintenance

### Sensors & Actors:
- SEN-VIB01
- SPH9645 I2S Mic
- VMA438
- ESP32-S3 1-N16R8 Dev Kit

I might change them later a bit, depending on how good the prototype will perform

### Dependencies:
- esp hal ver.: 1.2.2 
- bootloader esp idf ver.: 0.6.0
- embedded graphics ver.: 0.8.1
- ssd1306 ver.: 0.10.0

The general idea is to proof that mechanical Valve clearances or transmition noises can be recorded and used to check wether the clearence is in order or not. The combination of a microphone with RMS and a vibration sensor, even simple ones like the SEN-VIB01, should be enough to build a prediction system to make maintanance easier for mechanics and engineers.

The microphone is a mono type, so we use per select the left canal to use up to 32kHz, reason for that would be the Nyquist stability criteria, with that we can record up to 480 S/s at 8000rpm, where each full rotation of the engine would last about 15ms. Based on that it will be possible to record frequences up to 16kHz.
