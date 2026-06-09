import sys
import json
import time

def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            cmd = req.get("command")
            if cmd == "slow_job":
                # Print some progress
                for i in range(5):
                    print(json.dumps({
                        "event": "progress",
                        "job_id": "job-1",
                        "percent": float(i * 20),
                        "message": f"Step {i}"
                    }))
                    sys.stdout.flush()
                    time.sleep(0.5)
                print(json.dumps({
                    "event": "result",
                    "job_id": "job-1",
                    "message": "success",
                    "payload": {}
                }))
                sys.stdout.flush()
            elif cmd == "doctor":
                print(json.dumps({
                    "event": "result",
                    "message": "doctor_ok",
                    "payload": {"ping": req.get("payload", {}).get("ping", False)}
                }))
                sys.stdout.flush()
        except Exception as e:
            print(json.dumps({"event": "error", "message": str(e)}))
            sys.stdout.flush()

if __name__ == "__main__":
    main()
