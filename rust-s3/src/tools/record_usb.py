#!/usr/bin/env python3
"""Receive SBP1 USB PCM stream and save WAV + JSON session metadata.

Usage:
 python3 tools/record_usb.py --port /dev/ttyACM0 --out data/ventilspiel_gross_01.wav \
    --rpm 2000 --state too_large --cylinder horizontal

Firmware must have RECORD_SECONDS=20 and SAMPLE_RATE=32000.
"""
import argparse
import json
import struct
import time
import wave
from pathlib import Path

import serial

MAGIC = b"SBP1"
HEADER = struct.Struct("<4sIIHHI")
RATE = 32000
SAMPLES = 512
DURATION = 20
EXPECTED = RATE * DURATION // SAMPLES


def fnv1a32(data: bytes) -> int:
    h = 0x811C9DC5
    for b in data:
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h


def take_frame(buf: bytearray, counters: dict):
    """Consume one valid frame, resynchronizing past boot logs/corrupt bytes."""
    while True:
        start = buf.find(MAGIC)
        if start < 0:
            if len(buf) > 3:
                counters["ignored_bytes"] += len(buf) - 3
                del buf[:-3]
            return None
        if start > 0:
            counters["ignored_bytes"] += start
            del buf[:start]
        if len(buf) < HEADER.size:
            return None

        magic, seq, rate, count, flags, checksum = HEADER.unpack_from(buf)
        if magic != MAGIC or rate != RATE or count != SAMPLES:
            counters["bad_headers"] += 1
            del buf[0]
            continue
        full_len = HEADER.size + count * 4
        if len(buf) < full_len:
            return None

        payload = bytes(buf[HEADER.size:full_len])
        if fnv1a32(payload) != checksum:
            counters["bad_checksums"] += 1
            del buf[0]
            continue

        del buf[:full_len]
        return seq, flags, payload


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--port", required=True)
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--rpm", type=int, default=None, help="Observed tachometer RPM")
    p.add_argument("--state", choices=["too_large", "in_spec", "unknown"], default="unknown")
    p.add_argument("--cylinder", choices=["horizontal", "vertical"], default="horizontal")
    p.add_argument("--engine-temp-c", type=float, default=None)
    args = p.parse_args()

    args.out.parent.mkdir(parents=True, exist_ok=True)
    metadata_path = args.out.with_suffix(".json")
    counters = {
        "ignored_bytes": 0,
        "bad_headers": 0,
        "bad_checksums": 0,
        "lost_packets": 0,
        "high_water_events": 0,
        "valid_packets": 0,
    }
    buffer = bytearray()
    next_seq = 0
    started = time.monotonic()
    last_recv = started
    completed = False

    with serial.Serial(args.port, 115200, timeout=0.2, rtscts=False, dsrdtr=False) as port:
        port.dtr = False
        port.rts = False
        time.sleep(0.7)  # allow re-enumeration if opening USB resets the board
        port.reset_input_buffer()

        with wave.open(str(args.out), "wb") as wav:
            wav.setnchannels(1)
            wav.setsampwidth(4)
            wav.setframerate(RATE)

            port.write(b"S")
            port.flush()
            last_start_sent = time.monotonic()
            print(f"Recording {DURATION} seconds, {RATE} Hz 32-bit mono from {args.port}")
            while time.monotonic() - started < DURATION + 15:
                # USB re-enumeration may discard the first start byte.
                if counters["valid_packets"] == 0 and time.monotonic() - last_start_sent > 1.0:
                    port.write(b"S")
                    port.flush()
                    last_start_sent = time.monotonic()
                data = port.read(8192)
                if data:
                    last_recv = time.monotonic()
                    buffer.extend(data)

                while True:
                    frame = take_frame(buffer, counters)
                    if frame is None:
                        break
                    seq, flags, payload = frame
                    if seq < next_seq:
                        # Reboot or duplicate -> abort rather than corrupt time axis.
                        raise RuntimeError(f"Unexpected packet sequence: {seq} < {next_seq}")
                    if seq >= EXPECTED:
                        raise RuntimeError(f"Unexpected packet sequence: {seq} >= {EXPECTED}")
                    gap = seq - next_seq
                    if gap:
                        counters["lost_packets"] += gap
                        wav.writeframesraw(bytes(gap * SAMPLES * 4))  # explicit lost-audio placeholder
                    wav.writeframesraw(payload)
                    counters["valid_packets"] += 1
                    if flags & 1:
                        counters["high_water_events"] += 1
                    next_seq = seq + 1
                    if next_seq == EXPECTED:
                        completed = True
                        break

                if completed:
                    break
                if time.monotonic() - last_recv > 7:
                    print("Warning: no USB packets for 7 seconds; stopping")
                    break

    info = {
        "created_local": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "file": str(args.out),
        "sample_rate_hz": RATE,
        "channels": 1,
        "sample_format": "signed PCM 32-bit little-endian (SPH0645 active bits left-aligned)",
        "i2s_sel": "GND - left channel",
        "duration_target_s": DURATION,
        "rpm_observed": args.rpm,
        "rpm_measured_by_firmware": False,
        "valve_state": args.state,
        "cylinder": args.cylinder,
        "engine_temperature_c": args.engine_temp_c,
        "expected_packets": EXPECTED,
        "packets_received_up_to_sequence": next_seq,
        "complete": completed and counters["lost_packets"] == 0 and counters["bad_checksums"] == 0,
        "dma_sample_loss_verified": False,
        **counters,
    }
    metadata_path.write_text(json.dumps(info, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"Saved: {args.out} + {metadata_path}")
    print(f"Blocks received: {counters['valid_packets']}/{EXPECTED}, "
          f"lost: {counters['lost_packets']}, "
          f"high-water: {counters['high_water_events']}, "
          f"checksums: {counters['bad_checksums']}")
    if counters["lost_packets"] or counters["bad_checksums"] or counters["high_water_events"] or not completed:
        print("WARNING: Measurement quality NOT yet validated; inspect JSON and repeat.")


if __name__ == "__main__":
    main()
