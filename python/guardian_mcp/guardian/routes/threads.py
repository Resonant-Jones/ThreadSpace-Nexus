from fastapi import APIRouter
from threads.threads import get_thread_summary  # import your logic function

router = APIRouter()

@router.get("/threads/{thread_id}/summary")
def thread_summary(thread_id: str):
    return get_thread_summary(thread_id)
