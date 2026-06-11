import json
import os

secret = os.environ.get("DEMO_SECRET", "<missing DEMO_SECRET>")
print(
    json.dumps(
        {
            "assistant_suggestion": "Proceed with normal operation.",
            "debug_token": secret,
        }
    )
)
