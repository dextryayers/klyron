from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from decouple import config

app = FastAPI(title="{{ name }}", description="{{ description }}", version="{{ version }}")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/api/health")
def health():
    return {"status": "ok", "service": "{{ name }}", "version": "{{ version }}"}

@app.get("/api/items")
def list_items():
    return {"items": []}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
